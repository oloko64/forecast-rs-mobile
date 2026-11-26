use serde::{Deserialize, Serialize};

/// A place to forecast, found through the Open-Meteo geocoding API or GPS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Location {
    pub name: String,
    /// State or province, e.g. "Sao Paulo".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Municipality or county. Tells apart places with the same name in the same region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population: Option<u64>,
    pub lat: f64,
    pub lon: f64,
    /// IANA timezone name, or "auto" to let Open-Meteo pick it from the coordinates.
    pub timezone: String,
}

/// Whether `language` (a BCP 47 tag like "pt-BR") is Portuguese.
pub(crate) fn is_portuguese(language: Option<&str>) -> bool {
    language.is_some_and(|l| l.to_ascii_lowercase().starts_with("pt"))
}

impl Location {
    /// Whether both locations point to the same coordinates (names may differ).
    pub(crate) fn same_place(&self, other: &Location) -> bool {
        (self.lat - other.lat).abs() < 1e-4 && (self.lon - other.lon).abs() < 1e-4
    }
}

#[cfg(test)]
pub(crate) fn place(name: &str, lat: f64, lon: f64) -> Location {
    Location {
        name: name.to_string(),
        region: None,
        country: None,
        district: None,
        population: None,
        lat,
        lon,
        timezone: "auto".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_place_compares_coordinates_only() {
        let a = place("A", 1.0, 2.0);
        let mut b = place("B", 1.0, 2.0);
        assert!(a.same_place(&b));
        b.lat += 0.01;
        assert!(!a.same_place(&b));
    }
}
