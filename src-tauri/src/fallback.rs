//! Retries the preferred provider and switches to the other one when it keeps failing.

use std::{future::Future, sync::Mutex, time::Duration};

use tokio::time::{Instant, sleep};

use crate::types::{ForecastResponse, Provider};

/// Consecutive failures of the preferred provider before using the other one.
pub(crate) const ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(500);
/// After switching, keep using the fallback for this long before trying the
/// preferred provider again, so every refresh doesn't wait for 3 failures.
const COOLDOWN: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
pub(crate) struct Fallback {
    /// The provider that failed and until when to skip it.
    down: Mutex<Option<(Provider, Instant)>>,
}

impl Fallback {
    /// Forget past failures, e.g. after the user picks another provider.
    pub(crate) fn reset(&self) {
        *self.down.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    fn is_down(&self, provider: Provider) -> bool {
        matches!(*self.down.lock().unwrap_or_else(|e| e.into_inner()),
            Some((p, until)) if p == provider && Instant::now() < until)
    }

    fn set_down(&self, provider: Option<Provider>) {
        *self.down.lock().unwrap_or_else(|e| e.into_inner()) =
            provider.map(|p| (p, Instant::now() + COOLDOWN));
    }

    pub(crate) async fn fetch<F, Fut>(
        &self,
        preferred: Provider,
        fetch: F,
    ) -> anyhow::Result<ForecastResponse>
    where
        F: Fn(Provider) -> Fut,
        Fut: Future<Output = anyhow::Result<ForecastResponse>>,
    {
        let other = preferred.other();
        let served_by_fallback = |mut f: ForecastResponse| {
            f.fallback = true;
            f
        };

        let mut other_error = None;
        if self.is_down(preferred) {
            match fetch(other).await {
                Ok(f) => return Ok(served_by_fallback(f)),
                // The fallback is failing too: give the preferred one another go.
                Err(e) => other_error = Some(e),
            }
        }

        let mut last_error = None;
        for attempt in 1..=ATTEMPTS {
            match fetch(preferred).await {
                Ok(f) => {
                    self.set_down(None);
                    return Ok(f);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < ATTEMPTS {
                        sleep(RETRY_DELAY * attempt).await;
                    }
                }
            }
        }
        self.set_down(Some(preferred));
        let last_error = last_error.expect("ATTEMPTS > 0");

        let other_error = match other_error {
            Some(e) => e,
            None => match fetch(other).await {
                Ok(f) => return Ok(served_by_fallback(f)),
                Err(e) => e,
            },
        };
        anyhow::bail!(
            "{preferred} failed {ATTEMPTS} times ({last_error:#}), and {other} failed too ({other_error:#})"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn forecast(source: Provider) -> ForecastResponse {
        ForecastResponse {
            source,
            fallback: false,
            utc_offset_seconds: 0,
            timezone: "UTC".into(),
            daily: vec![],
            hourly: vec![],
        }
    }

    /// A fake fetcher: `failing` providers error; counts calls per provider.
    struct Fake {
        failing: Vec<Provider>,
        open_meteo_calls: AtomicU32,
        met_calls: AtomicU32,
    }

    impl Fake {
        fn new(failing: &[Provider]) -> Self {
            Self {
                failing: failing.to_vec(),
                open_meteo_calls: AtomicU32::new(0),
                met_calls: AtomicU32::new(0),
            }
        }

        async fn fetch(&self, p: Provider) -> anyhow::Result<ForecastResponse> {
            match p {
                Provider::OpenMeteo => &self.open_meteo_calls,
                Provider::MetNorway => &self.met_calls,
            }
            .fetch_add(1, Ordering::SeqCst);
            if self.failing.contains(&p) {
                anyhow::bail!("{p} down")
            }
            Ok(forecast(p))
        }

        fn calls(&self) -> (u32, u32) {
            (
                self.open_meteo_calls.load(Ordering::SeqCst),
                self.met_calls.load(Ordering::SeqCst),
            )
        }
    }

    #[tokio::test(start_paused = true)]
    async fn healthy_preferred_is_used() {
        let fb = Fallback::default();
        let fake = Fake::new(&[]);
        let f = fb
            .fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap();
        assert_eq!((f.source, f.fallback), (Provider::OpenMeteo, false));
        assert_eq!(fake.calls(), (1, 0));
    }

    #[tokio::test(start_paused = true)]
    async fn switches_after_three_failures_and_stays_switched() {
        let fb = Fallback::default();
        let fake = Fake::new(&[Provider::OpenMeteo]);
        let f = fb
            .fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap();
        assert_eq!((f.source, f.fallback), (Provider::MetNorway, true));
        assert_eq!(fake.calls(), (3, 1));

        // Next refresh goes straight to the fallback.
        fb.fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap();
        assert_eq!(fake.calls(), (3, 2));

        // After the cooldown the preferred provider is tried again.
        tokio::time::advance(COOLDOWN + Duration::from_secs(1)).await;
        fb.fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap();
        assert_eq!(fake.calls(), (6, 3));
    }

    #[tokio::test(start_paused = true)]
    async fn works_the_other_way_round() {
        let fb = Fallback::default();
        let fake = Fake::new(&[Provider::MetNorway]);
        let f = fb
            .fetch(Provider::MetNorway, |p| fake.fetch(p))
            .await
            .unwrap();
        assert_eq!((f.source, f.fallback), (Provider::OpenMeteo, true));
    }

    #[tokio::test(start_paused = true)]
    async fn both_down_is_an_error_naming_both() {
        let fb = Fallback::default();
        let fake = Fake::new(&[Provider::OpenMeteo, Provider::MetNorway]);
        let err = fb
            .fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Open-Meteo failed 3 times") && msg.contains("MET Norway"),
            "{msg}"
        );

        // While switched, a failing fallback still gives the preferred one a chance,
        // and the fallback is not called twice in one fetch.
        fb.fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap_err();
        assert_eq!(fake.calls(), (6, 2));
    }

    #[tokio::test(start_paused = true)]
    async fn reset_forgets_failures() {
        let fb = Fallback::default();
        let fake = Fake::new(&[Provider::OpenMeteo]);
        fb.fetch(Provider::OpenMeteo, |p| fake.fetch(p))
            .await
            .unwrap();
        fb.reset();
        assert!(!fb.is_down(Provider::OpenMeteo));
    }
}
