use std::{sync::OnceLock, time::Duration};

use anyhow::{Context, bail};
use serde::{Deserialize, de::DeserializeOwned};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// One shared client, trusting the bundled Mozilla root certificates.
fn client() -> anyhow::Result<&'static reqwest::Client> {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }

    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let tls = rustls::ClientConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into(),
    )
    .with_safe_default_protocol_versions()
    .context("Failed to set up TLS")?
    .with_root_certificates(roots)
    .with_no_client_auth();

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .tls_backend_preconfigured(tls)
        .build()
        .context("Failed to build HTTP client")?;

    Ok(CLIENT.get_or_init(|| client))
}

/// Sends a GET request to an Open-Meteo API and decodes the JSON response.
pub(crate) async fn get_json<T: DeserializeOwned>(
    url: &str,
    query: &[(&str, String)],
) -> anyhow::Result<T> {
    let response = client()?
        .get(url)
        .query(query)
        .send()
        .await
        .context("Failed to reach Open-Meteo")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        // Open-Meteo explains errors as {"error": true, "reason": "..."}
        let reason = serde_json::from_str::<ApiError>(&body).map_or(body, |e| e.reason);
        bail!("Open-Meteo request failed: HTTP {status}: {reason}");
    }

    response
        .json()
        .await
        .context("Failed to decode Open-Meteo response")
}

#[derive(Debug, Deserialize)]
struct ApiError {
    reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_error_reason() {
        let body = r#"{"error":true,"reason":"Forecast days is invalid. Allowed range 0 to 16."}"#;
        let error: ApiError = serde_json::from_str(body).unwrap();
        assert!(error.reason.starts_with("Forecast days is invalid"));
    }

    #[test]
    fn client_builds() {
        client().unwrap();
    }
}
