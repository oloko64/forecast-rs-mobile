//! Fallback provider: MET Norway Locationforecast 2.0 (free, no key, CC BY 4.0).
//! <https://api.met.no/weatherapi/locationforecast/2.0/documentation>

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, Offset, Utc};
use chrono_tz::Tz;
use serde::Deserialize;

use crate::{
    http::get_json,
    location::Location,
    openmeteo::decode_weather,
    types::{DayForecast, ForecastResponse, HourForecast, Provider},
};

const FORECAST_URL: &str = "https://api.met.no/weatherapi/locationforecast/2.0/compact";

pub(crate) async fn fetch_forecast(
    location: &Location,
    days: u32,
) -> anyhow::Result<ForecastResponse> {
    // MET asks for at most 4 decimals, so responses can be cached.
    let round = |n: f64| format!("{:.4}", n);
    let response: MetResponse = get_json(
        "MET Norway",
        FORECAST_URL,
        &[("lat", round(location.lat)), ("lon", round(location.lon))],
    )
    .await?;

    let tz = timezone_for(location);
    let today = Utc::now().with_timezone(&tz).date_naive();
    convert(response, tz, today, days)
}

/// The location's timezone. GPS locations without one use the device's, as the
/// device is there.
fn timezone_for(location: &Location) -> Tz {
    location
        .timezone
        .parse()
        .ok()
        .or_else(|| iana_time_zone::get_timezone().ok()?.parse().ok())
        .unwrap_or(Tz::UTC)
}

/// Turns MET's mixed 1h/6h UTC steps into an hourly local-time grid, `days` long
/// from `today`, plus daily summaries.
fn convert(
    response: MetResponse,
    tz: Tz,
    today: NaiveDate,
    days: u32,
) -> anyhow::Result<ForecastResponse> {
    let steps = response.properties.timeseries;
    let last_day = today + Duration::days(i64::from(days.max(1)) - 1);
    let mut hourly = Vec::new();

    for (i, step) in steps.iter().enumerate() {
        let next = steps.get(i + 1);
        let gap = next
            .map_or(1, |n| (n.time - step.time).num_hours())
            .clamp(1, 6);
        let period = if gap == 1 {
            step.data
                .next_1_hours
                .as_ref()
                .or(step.data.next_6_hours.as_ref())
        } else {
            step.data
                .next_6_hours
                .as_ref()
                .or(step.data.next_12_hours.as_ref())
        };
        // The last steps carry no forecast period, only an instant reading.
        let Some(period) = period else { continue };

        let weather = decode_weather(symbol_to_wmo(&period.summary.symbol_code));
        let span = if gap == 1 { 1.0 } else { 6.0 };
        let precipitation = period.details.precipitation_amount.unwrap_or(0.0) / span;
        let t0 = step.data.instant.details.air_temperature;
        let t1 = next.map_or(t0, |n| n.data.instant.details.air_temperature);

        for h in 0..gap {
            let time = (step.time + Duration::hours(h))
                .with_timezone(&tz)
                .naive_local();
            if time.date() < today || time.date() > last_day {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let temperature = t0 + (t1 - t0) * h as f64 / gap as f64;
            hourly.push(HourForecast {
                time,
                temperature: (temperature * 10.0).round() / 10.0,
                weather,
                precipitation: (precipitation * 100.0).round() / 100.0,
                precipitation_probability: None,
            });
        }
    }
    anyhow::ensure!(!hourly.is_empty(), "MET Norway returned no forecast data");

    let mut by_day: BTreeMap<NaiveDate, Vec<&HourForecast>> = BTreeMap::new();
    for hour in &hourly {
        by_day.entry(hour.time.date()).or_default().push(hour);
    }
    let daily = by_day
        .into_iter()
        .map(|(date, hours)| {
            let temps = hours.iter().map(|h| h.temperature);
            DayForecast {
                date,
                temperature_max: temps.clone().fold(f64::MIN, f64::max),
                temperature_min: temps.fold(f64::MAX, f64::min),
                // WMO codes grow with severity: show the day's worst weather.
                weather: hours
                    .iter()
                    .map(|h| h.weather)
                    .max_by_key(|w| wmo_rank(w.description))
                    .unwrap_or(hours[0].weather),
                precipitation: (hours.iter().map(|h| h.precipitation).sum::<f64>() * 10.0).round()
                    / 10.0,
            }
        })
        .collect();

    Ok(ForecastResponse {
        source: Provider::MetNorway,
        fallback: false,
        utc_offset_seconds: Utc::now()
            .with_timezone(&tz)
            .offset()
            .fix()
            .local_minus_utc(),
        timezone: tz.name().to_string(),
        daily,
        hourly,
    })
}

/// Maps a MET symbol code ("lightrainshowers_day") to the closest WMO code.
fn symbol_to_wmo(symbol: &str) -> u32 {
    let base = symbol.split('_').next().unwrap_or(symbol);
    if base.contains("thunder") {
        return if base.starts_with("heavy") { 97 } else { 95 };
    }
    match base {
        "clearsky" => 0,
        "fair" => 1,
        "partlycloudy" => 2,
        "cloudy" => 3,
        "fog" => 45,
        "lightrain" => 61,
        "rain" => 63,
        "heavyrain" => 65,
        "lightrainshowers" => 80,
        "rainshowers" => 81,
        "heavyrainshowers" => 82,
        "lightsleet" | "sleet" | "lightsleetshowers" | "sleetshowers" => 66,
        "heavysleet" | "heavysleetshowers" => 67,
        "lightsnow" => 71,
        "snow" => 73,
        "heavysnow" => 75,
        "lightsnowshowers" | "snowshowers" => 85,
        "heavysnowshowers" => 86,
        _ => u32::MAX,
    }
}

/// Severity order of the decoded descriptions, for the daily summary.
fn wmo_rank(description: &str) -> u32 {
    [
        "Clear sky",
        "Mainly clear",
        "Partly cloudy",
        "Overcast",
        "Fog",
        "Depositing rime fog",
        "Light drizzle",
        "Moderate drizzle",
        "Dense drizzle",
        "Slight rain",
        "Slight rain showers",
        "Moderate rain",
        "Moderate rain showers",
        "Snow grains",
        "Slight snow fall",
        "Slight snow showers",
        "Moderate snow fall",
        "Light freezing drizzle",
        "Dense freezing drizzle",
        "Light freezing rain",
        "Heavy rain",
        "Violent rain showers",
        "Heavy freezing rain",
        "Heavy snow fall",
        "Heavy snow showers",
        "Thunderstorm",
        "Heavy thunderstorm",
        "Thunderstorm with slight hail",
        "Thunderstorm with heavy hail",
    ]
    .iter()
    .position(|d| *d == description)
    .map_or(0, |p| u32::try_from(p).unwrap_or(0) + 1)
}

#[derive(Debug, Deserialize)]
struct MetResponse {
    properties: Properties,
}

#[derive(Debug, Deserialize)]
struct Properties {
    timeseries: Vec<Step>,
}

#[derive(Debug, Deserialize)]
struct Step {
    time: DateTime<Utc>,
    data: StepData,
}

#[derive(Debug, Deserialize)]
struct StepData {
    instant: Instant,
    next_1_hours: Option<Period>,
    next_6_hours: Option<Period>,
    next_12_hours: Option<Period>,
}

#[derive(Debug, Deserialize)]
struct Instant {
    details: InstantDetails,
}

#[derive(Debug, Deserialize)]
struct InstantDetails {
    air_temperature: f64,
}

#[derive(Debug, Deserialize)]
struct Period {
    summary: Summary,
    #[serde(default)]
    details: PeriodDetails,
}

#[derive(Debug, Deserialize)]
struct Summary {
    symbol_code: String,
}

#[derive(Debug, Default, Deserialize)]
struct PeriodDetails {
    precipitation_amount: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WeatherKind;
    use chrono::Timelike;

    fn step(time: &str, temp: f64, one: Option<(&str, f64)>, six: Option<(&str, f64)>) -> String {
        let period = |p: Option<(&str, f64)>, key: &str| {
            p.map_or(String::new(), |(s, a)| {
                format!(r#","{key}":{{"summary":{{"symbol_code":"{s}"}},"details":{{"precipitation_amount":{a}}}}}"#)
            })
        };
        format!(
            r#"{{"time":"{time}","data":{{"instant":{{"details":{{"air_temperature":{temp}}}}}{}{}}}}}"#,
            period(one, "next_1_hours"),
            period(six, "next_6_hours"),
        )
    }

    fn response(steps: &[String]) -> MetResponse {
        let json = format!(
            r#"{{"type":"Feature","properties":{{"timeseries":[{}]}}}}"#,
            steps.join(",")
        );
        serde_json::from_str(&json).unwrap()
    }

    fn sao_paulo() -> Tz {
        "America/Sao_Paulo".parse().unwrap()
    }

    #[test]
    fn converts_hourly_steps_to_local_time() {
        let r = response(&[
            step(
                "2026-11-26T15:00:00Z",
                24.0,
                Some(("rain", 1.0)),
                Some(("lightrain", 1.1)),
            ),
            step("2026-11-26T16:00:00Z", 25.0, Some(("cloudy", 0.0)), None),
            step("2026-11-26T17:00:00Z", 25.5, None, None),
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
        let f = convert(r, sao_paulo(), today, 7).unwrap();

        assert_eq!(f.source, Provider::MetNorway);
        assert_eq!(f.timezone, "America/Sao_Paulo");
        assert_eq!(f.hourly.len(), 2, "the last step has no period");
        assert_eq!(f.hourly[0].time.hour(), 12, "UTC-3");
        assert_eq!(f.hourly[0].weather.description, "Moderate rain");
        assert!((f.hourly[0].precipitation - 1.0).abs() < 1e-9);
        assert_eq!(f.daily.len(), 1);
        assert!((f.daily[0].temperature_max - 25.0).abs() < 1e-9);
        assert_eq!(
            f.daily[0].weather.description, "Moderate rain",
            "worst of the day"
        );
    }

    #[test]
    fn six_hour_steps_are_spread_over_hours() {
        let r = response(&[
            step("2026-11-27T03:00:00Z", 20.0, None, Some(("heavyrain", 6.0))),
            step(
                "2026-11-27T09:00:00Z",
                26.0,
                None,
                Some(("clearsky_day", 0.0)),
            ),
            step("2026-11-27T15:00:00Z", 30.0, None, None),
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        let f = convert(r, sao_paulo(), today, 7).unwrap();

        assert_eq!(f.hourly.len(), 12);
        assert_eq!(f.hourly[0].time.hour(), 0);
        assert!(
            (f.hourly[1].temperature - 21.0).abs() < 1e-9,
            "interpolated"
        );
        assert!(
            (f.hourly[0].precipitation - 1.0).abs() < 1e-9,
            "6 mm over 6 h"
        );
        assert!((f.daily[0].precipitation - 6.0).abs() < 1e-9);
        assert_eq!(f.hourly[6].weather.kind, WeatherKind::Clear);
    }

    #[test]
    fn days_limit_the_range() {
        let r = response(&[
            step("2026-11-26T12:00:00Z", 20.0, None, Some(("fair_day", 0.0))),
            step("2026-11-27T12:00:00Z", 20.0, None, Some(("fair_day", 0.0))),
            step("2026-11-28T12:00:00Z", 20.0, None, None),
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        let f = convert(r, Tz::UTC, today, 1).unwrap();
        assert!(f.hourly.iter().all(|h| h.time.date() == today));
        assert_eq!(f.daily.len(), 1);
    }

    #[test]
    fn empty_response_is_an_error() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        assert!(convert(response(&[]), Tz::UTC, today, 7).is_err());
    }

    #[test]
    fn maps_symbol_codes() {
        assert_eq!(symbol_to_wmo("clearsky_night"), 0);
        assert_eq!(symbol_to_wmo("partlycloudy_polartwilight"), 2);
        assert_eq!(symbol_to_wmo("heavyrainshowersandthunder_day"), 97);
        assert_eq!(symbol_to_wmo("lightrainandthunder"), 95);
        assert_eq!(symbol_to_wmo("lightsnowshowers_day"), 85);
        let known = [
            "clearsky",
            "fair",
            "partlycloudy",
            "cloudy",
            "fog",
            "lightrain",
            "rain",
            "heavyrain",
            "lightrainshowers",
            "rainshowers",
            "heavyrainshowers",
            "lightsleet",
            "sleet",
            "heavysleet",
            "lightsnow",
            "snow",
            "heavysnow",
            "snowshowers",
            "rainandthunder",
        ];
        for s in known {
            assert_ne!(
                decode_weather(symbol_to_wmo(s)).kind,
                WeatherKind::Unknown,
                "{s}"
            );
        }
        assert_eq!(
            decode_weather(symbol_to_wmo("martian_dust")).kind,
            WeatherKind::Unknown
        );
    }

    #[test]
    fn every_description_has_a_rank() {
        for code in 0..=99 {
            let w = decode_weather(code);
            if w.kind != WeatherKind::Unknown {
                assert!(wmo_rank(w.description) > 0, "{}", w.description);
            }
        }
    }

    #[test]
    fn auto_timezone_uses_the_device() {
        let mut l = crate::location::place("Here", 0.0, 0.0);
        l.timezone = "auto".into();
        let _ = timezone_for(&l); // never panics; UTC when the device has none
        l.timezone = "Europe/Lisbon".into();
        assert_eq!(timezone_for(&l).name(), "Europe/Lisbon");
    }
}
