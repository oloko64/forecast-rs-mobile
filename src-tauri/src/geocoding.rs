use serde::Deserialize;

use crate::{http::get_json, location::Location};

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const MAX_RESULTS: u32 = 10;

/// Finds places whose name matches `query`, best matches first.
pub(crate) async fn search(query: &str) -> anyhow::Result<Vec<Location>> {
    let response: SearchResponse = get_json(
        GEOCODING_URL,
        &[
            ("name", query.trim().to_string()),
            ("count", MAX_RESULTS.to_string()),
            ("language", "en".to_string()),
            ("format", "json".to_string()),
        ],
    )
    .await?;

    Ok(response.into_locations())
}

/// The subset of the geocoding response we use; other fields are ignored.
#[derive(Debug, Deserialize)]
struct SearchResponse {
    /// Missing when nothing matches.
    #[serde(default)]
    results: Vec<Place>,
}

#[derive(Debug, Deserialize)]
struct Place {
    name: String,
    latitude: f64,
    longitude: f64,
    timezone: Option<String>,
    admin1: Option<String>,
    admin2: Option<String>,
    country: Option<String>,
    population: Option<u64>,
}

impl SearchResponse {
    fn into_locations(self) -> Vec<Location> {
        self.results
            .into_iter()
            .map(|place| Location {
                name: place.name,
                region: place.admin1,
                country: place.country,
                district: place.admin2,
                population: place.population,
                lat: place.latitude,
                lon: place.longitude,
                timezone: place.timezone.unwrap_or_else(|| "auto".to_string()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_results() {
        let json = r#"{
            "results": [
                {
                    "id": 3471872, "name": "Araraquara", "latitude": -21.79444,
                    "longitude": -48.17556, "elevation": 664.0, "feature_code": "PPLA2",
                    "country_code": "BR", "timezone": "America/Sao_Paulo",
                    "population": 208662, "country_id": 3469034,
                    "country": "Brazil", "admin1": "Sao Paulo", "admin2": "Araraquara"
                },
                { "id": 1, "name": "Somewhere", "latitude": 1.5, "longitude": 2.5 }
            ],
            "generationtime_ms": 0.5
        }"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        let locations = response.into_locations();

        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].region.as_deref(), Some("Sao Paulo"));
        assert_eq!(locations[0].timezone, "America/Sao_Paulo");
        assert_eq!(locations[0].population, Some(208_662));
        assert_eq!(locations[0].district.as_deref(), Some("Araraquara"));
        assert_eq!(locations[1].name, "Somewhere");
        assert_eq!(locations[1].timezone, "auto");
    }

    #[test]
    fn no_results_is_empty() {
        let response: SearchResponse =
            serde_json::from_str(r#"{"generationtime_ms": 0.2}"#).unwrap();
        assert_eq!(response.into_locations(), vec![]);
    }
}
