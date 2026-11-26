use anyhow::{Context, ensure};
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::{
    http::get_json,
    location::Location,
    types::{DayForecast, ForecastResponse, HourForecast, Provider, Weather, WeatherKind},
};

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
/// The most days Open-Meteo forecasts, today included.
pub(crate) const MAX_FORECAST_DAYS: u32 = 16;

/// Fetches the daily and hourly forecast for `days` days, today included.
pub(crate) async fn fetch_forecast(
    location: &Location,
    days: u32,
) -> anyhow::Result<ForecastResponse> {
    let response: OpenMeteoResponse = get_json(
        "Open-Meteo",
        FORECAST_URL,
        &[
            ("latitude", location.lat.to_string()),
            ("longitude", location.lon.to_string()),
            ("timezone", location.timezone.clone()),
            (
                "daily",
                "temperature_2m_max,temperature_2m_min,weathercode,precipitation_sum".to_string(),
            ),
            (
                "hourly",
                "temperature_2m,weathercode,precipitation,precipitation_probability".to_string(),
            ),
            (
                "forecast_days",
                days.clamp(1, MAX_FORECAST_DAYS).to_string(),
            ),
        ],
    )
    .await?;

    response.try_into()
}

impl TryFrom<OpenMeteoResponse> for ForecastResponse {
    type Error = anyhow::Error;

    fn try_from(value: OpenMeteoResponse) -> Result<Self, Self::Error> {
        let OpenMeteoResponse {
            utc_offset_seconds,
            timezone,
            daily,
            mut hourly,
        } = value;

        ensure!(
            [
                daily.temperature_2m_max.len(),
                daily.temperature_2m_min.len(),
                daily.weathercode.len(),
                daily.precipitation_sum.len()
            ]
            .iter()
            .all(|&len| len == daily.time.len()),
            "Open-Meteo returned daily data with mismatched lengths"
        );
        ensure!(
            [
                hourly.temperature_2m.len(),
                hourly.weathercode.len(),
                hourly.precipitation.len()
            ]
            .iter()
            .all(|&len| len == hourly.time.len()),
            "Open-Meteo returned hourly data with mismatched lengths"
        );
        // Optional series: pad or trim so a missing one never fails the forecast.
        hourly
            .precipitation_probability
            .resize(hourly.time.len(), None);

        let daily = daily
            .time
            .iter()
            .zip(daily.temperature_2m_max)
            .zip(daily.temperature_2m_min)
            .zip(daily.weathercode)
            .zip(daily.precipitation_sum)
            .map(
                |((((date, temperature_max), temperature_min), code), precipitation)| {
                    Ok(DayForecast {
                        date: NaiveDate::parse_from_str(date, "%Y-%m-%d")
                            .with_context(|| format!("Invalid date from Open-Meteo: {date}"))?,
                        temperature_max,
                        temperature_min,
                        weather: decode_weather(code),
                        precipitation,
                    })
                },
            )
            .collect::<anyhow::Result<_>>()?;

        let hourly = hourly
            .time
            .iter()
            .zip(hourly.temperature_2m)
            .zip(hourly.weathercode)
            .zip(hourly.precipitation)
            .zip(hourly.precipitation_probability)
            .map(
                |((((time, temperature), code), precipitation), precipitation_probability)| {
                    Ok(HourForecast {
                        time: NaiveDateTime::parse_from_str(time, "%Y-%m-%dT%H:%M")
                            .with_context(|| format!("Invalid time from Open-Meteo: {time}"))?,
                        temperature,
                        weather: decode_weather(code),
                        precipitation,
                        precipitation_probability,
                    })
                },
            )
            .collect::<anyhow::Result<_>>()?;

        Ok(ForecastResponse {
            source: Provider::OpenMeteo,
            fallback: false,
            utc_offset_seconds,
            timezone,
            daily,
            hourly,
        })
    }
}

/// Decodes a WMO weather interpretation code, as returned by Open-Meteo.
pub(crate) fn decode_weather(code: u32) -> Weather {
    use WeatherKind::{Clear, Cloudy, Fog, Rain, Snow, Storm, Unknown};

    let (description, icon, kind) = match code {
        0 => ("Clear sky", "☀️", Clear),
        1 => ("Mainly clear", "🌤️", Clear),
        2 => ("Partly cloudy", "⛅", Cloudy),
        3 => ("Overcast", "☁️", Cloudy),
        45 | 48 => ("Fog", "🌫️", Fog),
        51 | 53 | 55 => ("Drizzle", "🌦️", Rain),
        56 | 57 => ("Freezing drizzle", "🥶", Rain),
        61 => ("Slight rain", "🌧️", Rain),
        63 => ("Moderate rain", "🌧️", Rain),
        65 => ("Heavy rain", "🌧️", Rain),
        66 | 67 => ("Freezing rain", "🧊", Rain),
        71 => ("Slight snow fall", "🌨️", Snow),
        73 => ("Moderate snow fall", "🌨️", Snow),
        75 => ("Heavy snow fall", "❄️", Snow),
        77 => ("Snow grains", "🌨️", Snow),
        80..=82 => ("Rain showers", "🌦️", Rain),
        85 | 86 => ("Snow showers", "🌨️", Snow),
        95 => ("Thunderstorm", "⛈️", Storm),
        96 | 99 => ("Thunderstorm with hail", "⛈️", Storm),
        _ => ("Unknown", "❔", Unknown),
    };

    Weather {
        description,
        icon,
        kind,
    }
}

/// The subset of the Open-Meteo forecast response we use; other fields are ignored.
#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    utc_offset_seconds: i32,
    timezone: String,
    daily: DailyData,
    hourly: HourlyData,
}

#[derive(Debug, Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    weathercode: Vec<u32>,
    precipitation: Vec<f64>,
    #[serde(default)]
    precipitation_probability: Vec<Option<f64>>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    weathercode: Vec<u32>,
    precipitation_sum: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    const FIXTURE: &str = r#"{
        "latitude": -21.75, "longitude": -48.18, "generationtime_ms": 0.3,
        "utc_offset_seconds": -10800, "timezone": "America/Sao_Paulo",
        "timezone_abbreviation": "GMT-3", "elevation": 685.0,
        "daily_units": {}, "hourly_units": {},
        "daily": {
            "time": ["2026-11-11"],
            "temperature_2m_max": [27.3],
            "temperature_2m_min": [17.1],
            "weathercode": [2],
            "precipitation_sum": [1.5]
        },
        "hourly": {
            "time": ["2026-11-11T00:00", "2026-11-11T01:00"],
            "temperature_2m": [18.2, 17.9],
            "weathercode": [0, 1234],
            "precipitation": [0.0, 0.4],
            "precipitation_probability": [5, null]
        }
    }"#;

    fn fixture() -> OpenMeteoResponse {
        serde_json::from_str(FIXTURE).unwrap()
    }

    #[test]
    fn converts_api_response() {
        let forecast = ForecastResponse::try_from(fixture()).unwrap();

        assert_eq!(forecast.utc_offset_seconds, -10800);
        assert_eq!(forecast.timezone, "America/Sao_Paulo");
        assert_eq!(forecast.daily.len(), 1);
        assert_eq!(forecast.daily[0].date.to_string(), "2026-11-11");
        assert_eq!(forecast.daily[0].weather.description, "Partly cloudy");

        assert_eq!(forecast.hourly.len(), 2);
        assert_eq!(forecast.hourly[1].time.hour(), 1);
        assert_eq!(forecast.hourly[0].weather.kind, WeatherKind::Clear);
        assert_eq!(forecast.hourly[0].precipitation_probability, Some(5.0));
        assert_eq!(forecast.hourly[1].precipitation_probability, None);
    }

    #[test]
    fn serializes_for_the_frontend() {
        let forecast = ForecastResponse::try_from(fixture()).unwrap();
        let json = serde_json::to_value(&forecast).unwrap();
        assert_eq!(json["hourly"][1]["time"], "2026-11-11T01:00:00");
        assert_eq!(json["daily"][0]["date"], "2026-11-11");
        assert_eq!(json["daily"][0]["weather"]["kind"], "cloudy");
    }

    #[test]
    fn missing_probability_series_is_fine() {
        let json = FIXTURE.replace(
            r#""precipitation_probability": [5, null]"#,
            r#""unused": []"#,
        );
        assert!(!json.contains("probability"));
        let forecast =
            ForecastResponse::try_from(serde_json::from_str::<OpenMeteoResponse>(&json).unwrap())
                .unwrap();
        assert!(
            forecast
                .hourly
                .iter()
                .all(|h| h.precipitation_probability.is_none())
        );
    }

    /// Hits the real API over rustls: `cargo test -- --ignored`.
    #[tokio::test]
    #[ignore = "needs network"]
    async fn live_forecast_and_search() {
        let places = crate::geocoding::search("Araraquara", None).await.unwrap();
        assert!(!places.is_empty());
        let forecast = fetch_forecast(&places[0], 3).await.unwrap();
        assert_eq!(forecast.daily.len(), 3);
        assert_eq!(forecast.hourly.len(), 72);

        let met = crate::metno::fetch_forecast(&places[0], 3).await.unwrap();
        assert_eq!(met.timezone, "America/Sao_Paulo");
        assert!(!met.hourly.is_empty() && met.daily.len() <= 3);
    }

    #[test]
    fn unknown_weather_code_does_not_fail() {
        let forecast = ForecastResponse::try_from(fixture()).unwrap();
        assert_eq!(forecast.hourly[1].weather.kind, WeatherKind::Unknown);
    }

    #[test]
    fn mismatched_lengths_are_an_error() {
        let mut response = fixture();
        response.hourly.precipitation.pop();
        let err = ForecastResponse::try_from(response).unwrap_err();
        assert!(err.to_string().contains("hourly"));

        let mut response = fixture();
        response.daily.weathercode.push(0);
        let err = ForecastResponse::try_from(response).unwrap_err();
        assert!(err.to_string().contains("daily"));
    }

    #[test]
    fn invalid_dates_are_an_error() {
        let mut response = fixture();
        response.daily.time[0] = "11/11/2026".to_string();
        let err = ForecastResponse::try_from(response).unwrap_err();
        assert!(err.to_string().contains("11/11/2026"));
    }

    #[test]
    fn every_documented_wmo_code_is_known() {
        let documented = [
            0, 1, 2, 3, 45, 48, 51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 71, 73, 75, 77, 80, 81, 82,
            85, 86, 95, 96, 99,
        ];
        for code in documented {
            assert_ne!(
                decode_weather(code).kind,
                WeatherKind::Unknown,
                "code {code}"
            );
        }
        assert_eq!(decode_weather(4).kind, WeatherKind::Unknown);
    }
}
