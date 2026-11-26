//! Shares the latest forecast with the Android home-screen widget.
//!
//! The widget (Kotlin, `gen/android/.../widget`) reads `widget.json` next to
//! `config.json`, and its own background refresh writes the same format.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{location::Location, types::ForecastResponse};

pub(crate) const FILE_NAME: &str = "widget.json";

#[derive(Serialize)]
struct WidgetData<'a> {
    location: &'a Location,
    forecast: &'a ForecastResponse,
    /// Unix time in milliseconds.
    fetched_at: u128,
}

/// Writes the forecast for the widget, atomically.
pub(crate) fn save(dir: &Path, location: &Location, forecast: &ForecastResponse) -> Result<()> {
    let fetched_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let json = serde_json::to_string(&WidgetData {
        location,
        forecast,
        fetched_at,
    })?;
    let path = dir.join(FILE_NAME);
    let tmp = path.with_extension("json.tmp");
    std::fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    std::fs::write(&tmp, json).with_context(|| format!("Failed to write {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| format!("Failed to replace {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{location::place, types::Provider};

    #[test]
    fn writes_the_shape_the_widget_reads() {
        let dir = std::env::temp_dir().join(format!("forecast-widget-test-{}", std::process::id()));
        let forecast = ForecastResponse {
            source: Provider::OpenMeteo,
            fallback: false,
            utc_offset_seconds: -10800,
            timezone: "America/Sao_Paulo".into(),
            daily: vec![],
            hourly: vec![],
        };
        save(&dir, &place("Rio", -22.9, -43.2), &forecast).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join(FILE_NAME)).unwrap()).unwrap();
        assert_eq!(json["location"]["name"], "Rio");
        assert_eq!(json["forecast"]["utc_offset_seconds"], -10800);
        assert!(json["forecast"]["hourly"].is_array());
        assert!(json["fetched_at"].as_u64().unwrap() > 1_700_000_000_000);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
