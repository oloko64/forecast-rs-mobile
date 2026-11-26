//! Describes a point picked on the map: a place name (OpenStreetMap Nominatim)
//! and its timezone (Open-Meteo), fetched concurrently.

use serde::Deserialize;

use crate::{
    http::get_json,
    location::{Location, is_portuguese},
};

const REVERSE_URL: &str = "https://nominatim.openstreetmap.org/reverse";
const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

/// A `Location` for the point. Lookups that fail fall back to a generic name
/// and the given timezone (or "auto"), so a pin can always be used.
pub(crate) async fn describe(
    lat: f64,
    lon: f64,
    fallback_timezone: Option<String>,
    language: Option<&str>,
) -> Location {
    let portuguese = is_portuguese(language);
    let lat = round(lat);
    // Panning around the world yields longitudes past ±180.
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    let lon = round(lon);
    let (place, timezone) = tokio::join!(reverse(lat, lon, portuguese), timezone(lat, lon));

    let mut location = place.ok().flatten().unwrap_or_else(|| Location {
        name: if portuguese {
            "Local marcado"
        } else {
            "Pinned location"
        }
        .into(),
        region: None,
        country: None,
        district: None,
        population: None,
        lat,
        lon,
        timezone: String::new(),
    });
    location.lat = lat;
    location.lon = lon;
    location.timezone = timezone
        .ok()
        .or(fallback_timezone)
        .unwrap_or_else(|| "auto".into());
    location
}

/// ~11 m, finer than a forecast grid and matching `same_place`'s tolerance.
fn round(n: f64) -> f64 {
    (n * 1e4).round() / 1e4
}

async fn reverse(lat: f64, lon: f64, portuguese: bool) -> anyhow::Result<Option<Location>> {
    let response: ReverseResponse = get_json(
        "OpenStreetMap",
        REVERSE_URL,
        &[
            ("format", "jsonv2".into()),
            ("lat", lat.to_string()),
            ("lon", lon.to_string()),
            ("zoom", "14".into()),
            ("addressdetails", "1".into()),
            (
                "accept-language",
                if portuguese { "pt-BR,pt" } else { "en" }.into(),
            ),
        ],
    )
    .await?;
    Ok(response.into_location(lat, lon))
}

async fn timezone(lat: f64, lon: f64) -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct Tz {
        timezone: String,
    }
    let tz: Tz = get_json(
        "Open-Meteo",
        FORECAST_URL,
        &[
            ("latitude", lat.to_string()),
            ("longitude", lon.to_string()),
            ("timezone", "auto".into()),
            ("forecast_days", "1".into()),
        ],
    )
    .await?;
    Ok(tz.timezone)
}

#[derive(Debug, Deserialize)]
struct ReverseResponse {
    /// Set instead of an address for points without one (e.g. the ocean).
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    address: Address,
}

#[derive(Debug, Default, Deserialize)]
struct Address {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    hamlet: Option<String>,
    county: Option<String>,
    suburb: Option<String>,
    neighbourhood: Option<String>,
    city_district: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

impl ReverseResponse {
    fn into_location(self, lat: f64, lon: f64) -> Option<Location> {
        if self.error.is_some() {
            return None;
        }
        let a = self.address;
        let nonempty = |s: Option<String>| s.filter(|s| !s.trim().is_empty());
        let settlement = nonempty(a.city)
            .or(nonempty(a.town))
            .or(nonempty(a.village))
            .or(nonempty(a.municipality))
            .or(nonempty(a.hamlet));
        let district = nonempty(a.suburb)
            .or(nonempty(a.neighbourhood))
            .or(nonempty(a.city_district));
        // Name the town; the neighbourhood goes in `district`. Outside towns, use
        // whatever OSM calls the spot.
        let name = settlement
            .clone()
            .or_else(|| nonempty(self.name.clone()))
            .or(nonempty(a.county))?;
        Some(Location {
            district: district.filter(|d| Some(d) != settlement.as_ref()),
            name,
            region: nonempty(a.state),
            country: nonempty(a.country),
            population: None,
            lat,
            lon,
            timezone: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Option<Location> {
        serde_json::from_str::<ReverseResponse>(json)
            .unwrap()
            .into_location(1.0, 2.0)
    }

    #[test]
    fn names_the_town_and_keeps_the_suburb() {
        let l = parse(
            r#"{"name":"Vila Melhado","address":{"suburb":"Vila Melhado","city":"Araraquara",
            "state":"São Paulo","country":"Brazil","country_code":"br"}}"#,
        )
        .unwrap();
        assert_eq!(l.name, "Araraquara");
        assert_eq!(l.district.as_deref(), Some("Vila Melhado"));
        assert_eq!(l.region.as_deref(), Some("São Paulo"));
        assert_eq!(l.country.as_deref(), Some("Brazil"));
    }

    #[test]
    fn falls_back_to_the_osm_name_then_county() {
        let l =
            parse(r#"{"name":"Serra da Canastra","address":{"state":"Minas Gerais"}}"#).unwrap();
        assert_eq!(l.name, "Serra da Canastra");
        let l = parse(r#"{"name":"","address":{"county":"Somewhere County"}}"#).unwrap();
        assert_eq!(l.name, "Somewhere County");
    }

    #[test]
    fn ocean_has_no_place() {
        assert!(parse(r#"{"error":"Unable to geocode"}"#).is_none());
    }

    #[test]
    fn rounds_coordinates() {
        assert!((round(-21.794_449) - -21.7944).abs() < 1e-9);
    }

    #[tokio::test]
    #[ignore = "needs network"]
    async fn live_describe() {
        let l = describe(-21.7944, -48.1756, None, None).await;
        assert_eq!(l.name, "Araraquara");
        assert_eq!(l.country.as_deref(), Some("Brazil"));
        let l = describe(-21.7944, -48.1756, None, Some("pt-BR")).await;
        assert_eq!(l.country.as_deref(), Some("Brasil"));
        assert_eq!(l.timezone, "America/Sao_Paulo");
        // Wrapped longitudes from a panned-around map are normalized.
        let l = describe(-21.7944, -48.1756 + 360.0, None, None).await;
        assert!((l.lon - -48.1756).abs() < 1e-6);
        let l = describe(0.0, -30.0, Some("UTC".into()), None).await;
        assert_eq!(l.name, "Pinned location");
        let l = describe(0.0, -30.0, None, Some("pt-BR")).await;
        assert_eq!(l.name, "Local marcado");
    }
}
