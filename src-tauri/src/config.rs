use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{location::Location, openmeteo::MAX_FORECAST_DAYS};

pub(crate) const FILE_NAME: &str = "config.json";

/// Day counts offered in the settings, today included.
pub(crate) const DAY_CHOICES: [u32; 4] = [3, 7, 10, 16];
const DEFAULT_DAYS: u32 = 7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub last_location: Option<Location>,
    #[serde(default)]
    pub favorites: Vec<Location>,
    /// How many days to forecast, today included.
    #[serde(default = "default_days")]
    pub days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_location: None,
            favorites: Vec::new(),
            days: DEFAULT_DAYS,
        }
    }
}

fn default_days() -> u32 {
    DEFAULT_DAYS
}

impl Config {
    /// Loads the config file, or an empty config if nothing has been saved yet.
    pub(crate) fn load_from(path: &Path) -> Result<Self> {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
        };

        let mut config: Self = serde_json::from_str(&content)
            .with_context(|| format!("Invalid config file {}", path.display()))?;
        config.days = config.days.clamp(1, MAX_FORECAST_DAYS);
        Ok(config)
    }

    /// Writes the config atomically: a crash mid-write never leaves a broken file.
    pub(crate) fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
        }
        let tmp: PathBuf = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("Failed to write {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Failed to replace {}", path.display()))
    }

    pub(crate) fn set_days(&mut self, days: u32) {
        self.days = days.clamp(1, MAX_FORECAST_DAYS);
    }

    pub(crate) fn is_favorite(&self, location: &Location) -> bool {
        self.favorites.iter().any(|fav| fav.same_place(location))
    }

    /// Adds the location to the favorites, or removes it if already there.
    pub(crate) fn toggle_favorite(&mut self, location: &Location) {
        if self.is_favorite(location) {
            self.favorites.retain(|fav| !fav.same_place(location));
        } else {
            self.favorites.push(location.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::place;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("forecast-mobile-test-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }

        fn file(&self) -> PathBuf {
            self.0.join(FILE_NAME)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_file_is_empty_config() {
        let dir = TempDir::new("missing");
        assert_eq!(Config::load_from(&dir.file()).unwrap(), Config::default());
    }

    #[test]
    fn save_creates_the_directory_and_round_trips() {
        let dir = TempDir::new("roundtrip");
        let config = Config {
            last_location: Some(place("Lisbon", 38.7, -9.1)),
            favorites: vec![place("Lisbon", 38.7, -9.1)],
            days: 10,
        };
        config.save_to(&dir.file()).unwrap();
        assert_eq!(Config::load_from(&dir.file()).unwrap(), config);
        assert!(!dir.0.join("config.json.tmp").exists());
    }

    #[test]
    fn days_default_to_a_week_and_are_clamped() {
        let dir = TempDir::new("days");
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.file(), r#"{"favorites":[]}"#).unwrap();
        assert_eq!(Config::load_from(&dir.file()).unwrap().days, 7);

        std::fs::write(dir.file(), r#"{"days":99}"#).unwrap();
        assert_eq!(Config::load_from(&dir.file()).unwrap().days, 16);

        let mut config = Config::default();
        config.set_days(0);
        assert_eq!(config.days, 1);
    }

    #[test]
    fn invalid_json_is_an_error() {
        let dir = TempDir::new("badjson");
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.file(), "{bad").unwrap();
        let err = Config::load_from(&dir.file()).unwrap_err();
        assert!(format!("{err:#}").contains("Invalid config file"));
    }

    #[test]
    fn toggle_favorite_adds_and_removes() {
        let mut config = Config::default();
        let lisbon = place("Lisbon", 38.7, -9.1);
        config.toggle_favorite(&lisbon);
        assert!(config.is_favorite(&lisbon));
        config.toggle_favorite(&lisbon);
        assert_eq!(config.favorites, vec![]);
    }
}
