use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{location::Location, openmeteo::MAX_FORECAST_DAYS, types::Provider};

pub(crate) const FILE_NAME: &str = "config.json";

/// Day counts offered in the settings, today included.
pub(crate) const DAY_CHOICES: [u32; 4] = [3, 7, 10, 16];
const DEFAULT_DAYS: u32 = 7;
/// Auto-refresh interval while the app is open, in minutes (10 min to 2 h).
const DEFAULT_REFRESH_MINUTES: u32 = 15;
const REFRESH_MINUTES_RANGE: std::ops::RangeInclusive<u32> = 10..=120;
/// On coming back to the app, refresh if the forecast is older than this (15 min to 2 h).
const DEFAULT_RESUME_MINUTES: u32 = 30;
const RESUME_MINUTES_RANGE: std::ops::RangeInclusive<u32> = 15..=120;
/// Home-screen widget background refresh, in minutes. Android runs periodic
/// work at most every 15 minutes.
const DEFAULT_WIDGET_REFRESH_MINUTES: u32 = 30;
const WIDGET_REFRESH_MINUTES_RANGE: std::ops::RangeInclusive<u32> = 15..=240;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub last_location: Option<Location>,
    #[serde(default)]
    pub favorites: Vec<Location>,
    /// How many days to forecast, today included.
    #[serde(default = "default_days")]
    pub days: u32,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub palette: Palette,
    /// Preferred forecast provider; the other one is the fallback.
    #[serde(default)]
    pub provider: Provider,
    #[serde(default)]
    pub units: Units,
    /// User changes to the preset colors, per palette.
    #[serde(default)]
    pub custom_colors: BTreeMap<Palette, PaletteColors>,
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u32,
    #[serde(default = "default_resume_minutes")]
    pub resume_minutes: u32,
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_widget_refresh_minutes")]
    pub widget_refresh_minutes: u32,
    #[serde(default)]
    pub widget: WidgetOptions,
    /// Which `WIDGET_DEFAULTS_VERSION` this config has been migrated to.
    #[serde(default)]
    pub widget_defaults: u32,
}

fn default_widget_refresh_minutes() -> u32 {
    DEFAULT_WIDGET_REFRESH_MINUTES
}

/// What the home-screen widget shows and how it looks.
/// Read by the widget's Kotlin code from config.json.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct WidgetOptions {
    pub location: bool,
    pub now: bool,
    pub summary: bool,
    pub day_picker: bool,
    pub refresh_button: bool,
    pub rain: bool,
    pub clock: ClockStyle,
    /// Background opacity in percent; 0 is fully transparent.
    pub background: u8,
}

impl Default for WidgetOptions {
    fn default() -> Self {
        Self {
            location: false,
            now: true,
            summary: false,
            day_picker: true,
            refresh_button: true,
            rain: true,
            clock: ClockStyle::Big,
            background: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ClockStyle {
    Off,
    Small,
    #[default]
    Big,
}

/// Bumped when the widget's defaults change for everyone (see `migrate`).
const WIDGET_DEFAULTS_VERSION: u32 = 2;

fn default_refresh_minutes() -> u32 {
    DEFAULT_REFRESH_MINUTES
}

fn default_resume_minutes() -> u32 {
    DEFAULT_RESUME_MINUTES
}

fn clamp_to(minutes: u32, range: &std::ops::RangeInclusive<u32>) -> u32 {
    minutes.clamp(*range.start(), *range.end())
}

/// Interface language; `System` follows the phone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "pt-BR")]
    PtBr,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Units {
    #[default]
    Celsius,
    Fahrenheit,
}

/// Overrides for a palette's colors, as "#rrggbb". Missing ones keep the preset.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PaletteColors {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rain: Option<String>,
}

impl PaletteColors {
    fn is_empty(&self) -> bool {
        self.accent.is_none() && self.temp.is_none() && self.rain.is_none()
    }

    /// Keeps only valid "#rrggbb" colors, lowercased. They end up in CSS.
    fn sanitized(self) -> Self {
        let clean = |c: Option<String>| {
            c.filter(|c| {
                c.len() == 7
                    && c.starts_with('#')
                    && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
            })
            .map(|c| c.to_ascii_lowercase())
        };
        Self {
            accent: clean(self.accent),
            temp: clean(self.temp),
            rain: clean(self.rain),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Palette {
    #[default]
    Ocean,
    Forest,
    Sunset,
}

/// Settings changed from the settings sheet; missing fields stay as they are.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct SettingsPatch {
    pub days: Option<u32>,
    pub theme_mode: Option<ThemeMode>,
    pub palette: Option<Palette>,
    pub provider: Option<Provider>,
    pub units: Option<Units>,
    /// Replaces a palette's custom colors; all fields empty resets it to the preset.
    pub custom_colors: Option<(Palette, PaletteColors)>,
    pub refresh_minutes: Option<u32>,
    pub resume_minutes: Option<u32>,
    pub language: Option<Language>,
    pub widget_refresh_minutes: Option<u32>,
    pub widget: Option<WidgetOptions>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_location: None,
            favorites: Vec::new(),
            days: DEFAULT_DAYS,
            theme_mode: ThemeMode::default(),
            palette: Palette::default(),
            provider: Provider::default(),
            units: Units::default(),
            custom_colors: BTreeMap::new(),
            refresh_minutes: DEFAULT_REFRESH_MINUTES,
            resume_minutes: DEFAULT_RESUME_MINUTES,
            language: Language::default(),
            widget_refresh_minutes: DEFAULT_WIDGET_REFRESH_MINUTES,
            widget: WidgetOptions::default(),
            widget_defaults: WIDGET_DEFAULTS_VERSION,
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
        config.refresh_minutes = clamp_to(config.refresh_minutes, &REFRESH_MINUTES_RANGE);
        config.resume_minutes = clamp_to(config.resume_minutes, &RESUME_MINUTES_RANGE);
        config.widget_refresh_minutes =
            clamp_to(config.widget_refresh_minutes, &WIDGET_REFRESH_MINUTES_RANGE);
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
        std::fs::rename(&tmp, path).with_context(|| format!("Failed to replace {}", path.display()))
    }

    pub(crate) fn apply(&mut self, patch: SettingsPatch) {
        if let Some(days) = patch.days {
            self.days = days.clamp(1, MAX_FORECAST_DAYS);
        }
        if let Some(mode) = patch.theme_mode {
            self.theme_mode = mode;
        }
        if let Some(palette) = patch.palette {
            self.palette = palette;
        }
        if let Some(provider) = patch.provider {
            self.provider = provider;
        }
        if let Some(units) = patch.units {
            self.units = units;
        }
        if let Some(minutes) = patch.refresh_minutes {
            self.refresh_minutes = clamp_to(minutes, &REFRESH_MINUTES_RANGE);
        }
        if let Some(language) = patch.language {
            self.language = language;
        }
        if let Some(minutes) = patch.widget_refresh_minutes {
            self.widget_refresh_minutes = clamp_to(minutes, &WIDGET_REFRESH_MINUTES_RANGE);
        }
        if let Some(mut widget) = patch.widget {
            widget.background = widget.background.min(100);
            self.widget = widget;
        }
        if let Some(minutes) = patch.resume_minutes {
            self.resume_minutes = clamp_to(minutes, &RESUME_MINUTES_RANGE);
        }
        if let Some((palette, colors)) = patch.custom_colors {
            let colors = colors.sanitized();
            if colors.is_empty() {
                self.custom_colors.remove(&palette);
            } else {
                self.custom_colors.insert(palette, colors);
            }
        }
    }

    /// One-time changes for configs saved by older versions. Returns whether
    /// anything changed (the caller saves it, so the widget sees it too).
    pub(crate) fn migrate(&mut self) -> bool {
        if self.widget_defaults >= WIDGET_DEFAULTS_VERSION {
            return false;
        }
        // v2: the location name and the day summary are off by default.
        self.widget.location = false;
        self.widget.summary = false;
        self.widget_defaults = WIDGET_DEFAULTS_VERSION;
        true
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
            let dir = std::env::temp_dir().join(format!(
                "forecast-mobile-test-{name}-{}",
                std::process::id()
            ));
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
            theme_mode: ThemeMode::Dark,
            palette: Palette::Sunset,
            provider: Provider::MetNorway,
            units: Units::Fahrenheit,
            custom_colors: BTreeMap::from([(
                Palette::Forest,
                PaletteColors {
                    accent: Some("#112233".into()),
                    ..PaletteColors::default()
                },
            )]),
            refresh_minutes: 30,
            resume_minutes: 60,
            language: Language::PtBr,
            widget_refresh_minutes: 60,
            widget: WidgetOptions {
                rain: false,
                clock: ClockStyle::Big,
                background: 25,
                ..WidgetOptions::default()
            },
            widget_defaults: WIDGET_DEFAULTS_VERSION,
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
        config.apply(SettingsPatch {
            days: Some(0),
            ..SettingsPatch::default()
        });
        assert_eq!(config.days, 1);
    }

    #[test]
    fn older_configs_get_default_settings() {
        let dir = TempDir::new("older");
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.file(), r#"{"favorites":[],"days":3}"#).unwrap();
        let config = Config::load_from(&dir.file()).unwrap();
        assert_eq!(config.theme_mode, ThemeMode::System);
        assert_eq!(config.palette, Palette::Ocean);
        assert_eq!(config.provider, Provider::OpenMeteo);
        assert_eq!(config.units, Units::Celsius);
        assert_eq!(config.refresh_minutes, 15);
        assert_eq!(config.resume_minutes, 30);
        assert_eq!(config.language, Language::System);
        assert_eq!(config.widget_refresh_minutes, 30);
        assert_eq!(config.widget, WidgetOptions::default());
        assert!(config.custom_colors.is_empty());
    }

    #[test]
    fn widget_settings_are_clamped_and_partial_options_default_to_shown() {
        let mut config = Config::default();
        let patch: SettingsPatch =
            serde_json::from_str(r#"{"widget_refresh_minutes":5,"widget":{"rain":false}}"#)
                .unwrap();
        config.apply(patch);
        assert_eq!(config.widget_refresh_minutes, 15, "Android's minimum");
        assert!(!config.widget.rain);
        assert!(config.widget.day_picker && config.widget.now);
        assert!(
            !config.widget.location && !config.widget.summary,
            "off by default"
        );
        assert_eq!(config.widget.clock, ClockStyle::Big, "big clock by default");
        assert_eq!(config.widget.background, 100);
        config.apply(SettingsPatch {
            widget_refresh_minutes: Some(999),
            ..SettingsPatch::default()
        });
        assert_eq!(config.widget_refresh_minutes, 240);
    }

    #[test]
    fn widget_background_is_capped_at_100() {
        let mut config = Config::default();
        let patch: SettingsPatch =
            serde_json::from_str(r#"{"widget":{"background":250,"clock":"small"}}"#).unwrap();
        config.apply(patch);
        assert_eq!(config.widget.background, 100);
        assert_eq!(config.widget.clock, ClockStyle::Small);
    }

    #[test]
    fn older_widget_settings_get_the_new_defaults_once() {
        let dir = TempDir::new("migrate");
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(
            dir.file(),
            r#"{"widget":{"location":true,"summary":true,"rain":false}}"#,
        )
        .unwrap();
        let mut config = Config::load_from(&dir.file()).unwrap();
        assert!(config.migrate());
        assert!(!config.widget.location && !config.widget.summary);
        assert!(!config.widget.rain, "other choices are kept");
        assert_eq!(config.widget.clock, ClockStyle::Big);

        // Turned back on after the migration: stays on.
        config.widget.location = true;
        assert!(!config.migrate());
        assert!(config.widget.location);
        // A brand-new config is already current.
        assert!(!Config::default().migrate());
    }

    #[test]
    fn language_uses_bcp47_tags() {
        let patch: SettingsPatch = serde_json::from_str(r#"{"language":"pt-BR"}"#).unwrap();
        let mut config = Config::default();
        config.apply(patch);
        assert_eq!(config.language, Language::PtBr);
        assert_eq!(serde_json::to_value(config.language).unwrap(), "pt-BR");
    }

    #[test]
    fn refresh_intervals_are_clamped() {
        let mut config = Config::default();
        for (asked, kept) in [(5, 10), (10, 10), (15, 15), (120, 120), (600, 120)] {
            config.apply(SettingsPatch {
                refresh_minutes: Some(asked),
                ..SettingsPatch::default()
            });
            assert_eq!(config.refresh_minutes, kept);
        }
        for (asked, kept) in [(10, 15), (15, 15), (30, 30), (120, 120), (600, 120)] {
            config.apply(SettingsPatch {
                resume_minutes: Some(asked),
                ..SettingsPatch::default()
            });
            assert_eq!(config.resume_minutes, kept);
        }
    }

    #[test]
    fn saved_out_of_range_intervals_are_clamped_on_load() {
        let dir = TempDir::new("intervals");
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.file(), r#"{"refresh_minutes":2,"resume_minutes":500}"#).unwrap();
        let config = Config::load_from(&dir.file()).unwrap();
        assert_eq!((config.refresh_minutes, config.resume_minutes), (10, 120));
    }

    #[test]
    fn custom_colors_are_validated_and_resettable() {
        let mut config = Config::default();
        let patch: SettingsPatch = serde_json::from_str(
            r##"{"custom_colors":["sunset",{"accent":"#AABBCC","temp":"red;}","rain":"#12345"}]}"##,
        )
        .unwrap();
        config.apply(patch);
        let colors = &config.custom_colors[&Palette::Sunset];
        assert_eq!(colors.accent.as_deref(), Some("#aabbcc"));
        assert_eq!(colors.temp, None, "not a hex color");
        assert_eq!(colors.rain, None, "too short");

        config.apply(SettingsPatch {
            custom_colors: Some((Palette::Sunset, PaletteColors::default())),
            ..SettingsPatch::default()
        });
        assert!(config.custom_colors.is_empty());
    }

    #[test]
    fn patch_changes_only_given_fields() {
        let mut config = Config::default();
        let patch: SettingsPatch =
            serde_json::from_str(r#"{"theme_mode":"dark","provider":"met_norway"}"#).unwrap();
        config.apply(patch);
        assert_eq!(config.theme_mode, ThemeMode::Dark);
        assert_eq!(config.provider, Provider::MetNorway);
        assert_eq!(config.palette, Palette::Ocean);
        assert_eq!(config.days, 7);
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
