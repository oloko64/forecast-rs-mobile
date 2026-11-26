mod config;
mod fallback;
mod geocoding;
mod http;
mod location;
mod metno;
mod openmeteo;
mod spot;
mod types;
mod widget;

use std::{path::PathBuf, sync::Mutex};

use tauri::{Manager, State};

use crate::{
    config::{Config, SettingsPatch},
    fallback::Fallback,
    location::Location,
    types::{ForecastResponse, Provider},
};

/// The config and where it lives. Every mutation is saved right away.
struct ConfigStore {
    path: PathBuf,
    config: Mutex<Config>,
}

impl ConfigStore {
    fn update(&self, change: impl FnOnce(&mut Config)) -> Result<Config, String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        change(&mut config);
        config.save_to(&self.path).map_err(|e| format!("{e:#}"))?;
        Ok(config.clone())
    }
}

/// Commands return `String` errors so the frontend can show them as-is.
fn err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

async fn fetch_from(
    provider: Provider,
    location: &Location,
    days: u32,
) -> anyhow::Result<ForecastResponse> {
    match provider {
        Provider::OpenMeteo => openmeteo::fetch_forecast(location, days).await,
        Provider::MetNorway => metno::fetch_forecast(location, days).await,
    }
}

/// Uses the provider chosen in the settings, falling back to the other one
/// after 3 consecutive failures.
#[tauri::command]
async fn get_forecast(
    store: State<'_, ConfigStore>,
    fallback: State<'_, Fallback>,
    location: Location,
    days: u32,
) -> Result<ForecastResponse, String> {
    let preferred = store.config.lock().map_err(|e| e.to_string())?.provider;
    let forecast = fallback
        .fetch(preferred, |p| fetch_from(p, &location, days))
        .await
        .map_err(err)?;
    // Keep the home-screen widget in sync; a failure here must not hide the forecast.
    if let Some(dir) = store.path.parent()
        && let Err(e) = widget::save(dir, &location, &forecast)
    {
        eprintln!("Widget data not saved: {e:#}");
    }
    Ok(forecast)
}

#[tauri::command]
async fn search_locations(
    query: String,
    language: Option<String>,
) -> Result<Vec<Location>, String> {
    if query.trim().len() < 2 {
        return Ok(Vec::new());
    }
    geocoding::search(&query, language.as_deref())
        .await
        .map_err(err)
}

/// Names a point picked on the map and resolves its timezone.
#[tauri::command]
async fn describe_spot(
    lat: f64,
    lon: f64,
    fallback_timezone: Option<String>,
    language: Option<String>,
) -> Location {
    spot::describe(lat, lon, fallback_timezone, language.as_deref()).await
}

#[tauri::command]
fn load_config(store: State<'_, ConfigStore>) -> Result<Config, String> {
    store
        .config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_last_location(store: State<'_, ConfigStore>, location: Location) -> Result<Config, String> {
    store.update(|c| c.last_location = Some(location))
}

#[tauri::command]
fn toggle_favorite(store: State<'_, ConfigStore>, location: Location) -> Result<Config, String> {
    store.update(|c| c.toggle_favorite(&location))
}

#[tauri::command]
fn update_settings(
    store: State<'_, ConfigStore>,
    fallback: State<'_, Fallback>,
    patch: SettingsPatch,
) -> Result<Config, String> {
    if patch.provider.is_some() {
        fallback.reset();
    }
    store.update(|c| c.apply(patch))
}

#[tauri::command]
fn day_choices() -> Vec<u32> {
    config::DAY_CHOICES.to_vec()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_geolocation::init());

    builder
        .setup(|app| {
            let path = app.path().app_data_dir()?.join(config::FILE_NAME);
            // A broken config file should not brick the app: start fresh instead.
            let config = Config::load_from(&path).unwrap_or_else(|e| {
                eprintln!("Ignoring config: {e:#}");
                Config::default()
            });
            app.manage(Fallback::default());
            app.manage(ConfigStore {
                path,
                config: Mutex::new(config),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_forecast,
            search_locations,
            describe_spot,
            load_config,
            set_last_location,
            toggle_favorite,
            update_settings,
            day_choices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
