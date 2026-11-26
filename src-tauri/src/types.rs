use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Where forecasts come from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Provider {
    #[default]
    OpenMeteo,
    MetNorway,
}

impl Provider {
    pub(crate) fn other(self) -> Self {
        match self {
            Self::OpenMeteo => Self::MetNorway,
            Self::MetNorway => Self::OpenMeteo,
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OpenMeteo => "Open-Meteo",
            Self::MetNorway => "MET Norway",
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ForecastResponse {
    /// The provider that answered.
    pub source: Provider,
    /// Whether `source` is the fallback because the preferred provider kept failing.
    pub fallback: bool,
    /// Offset of the forecast location's local time from UTC, in seconds.
    pub utc_offset_seconds: i32,
    /// IANA timezone Open-Meteo resolved for the location.
    pub timezone: String,
    pub daily: Vec<DayForecast>,
    pub hourly: Vec<HourForecast>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HourForecast {
    /// Local time at the forecast location, serialized as "YYYY-MM-DDTHH:MM:SS".
    pub time: NaiveDateTime,
    pub temperature: f64,
    pub weather: Weather,
    pub precipitation: f64,
    /// Probability of precipitation in percent; missing for some models.
    pub precipitation_probability: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DayForecast {
    pub date: NaiveDate,
    pub temperature_max: f64,
    pub temperature_min: f64,
    pub weather: Weather,
    pub precipitation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct Weather {
    pub description: &'static str,
    pub icon: &'static str,
    pub kind: WeatherKind,
}

/// Broad weather category, used to pick colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WeatherKind {
    Clear,
    Cloudy,
    Fog,
    Rain,
    Snow,
    Storm,
    Unknown,
}
