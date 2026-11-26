mod config;
mod geocoding;
mod http;
mod location;
mod openmeteo;
mod types;

use std::{path::PathBuf, sync::Mutex};

use tauri::{Manager, State};

use crate::{config::Config, location::Location, types::ForecastResponse};

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

#[tauri::command]
async fn get_forecast(location: Location, days: u32) -> Result<ForecastResponse, String> {
    openmeteo::fetch_forecast(&location, days).await.map_err(err)
}

#[tauri::command]
async fn search_locations(query: String) -> Result<Vec<Location>, String> {
    if query.trim().len() < 2 {
        return Ok(Vec::new());
    }
    geocoding::search(&query).await.map_err(err)
}

#[tauri::command]
fn load_config(store: State<'_, ConfigStore>) -> Result<Config, String> {
    store.config.lock().map(|c| c.clone()).map_err(|e| e.to_string())
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
fn set_days(store: State<'_, ConfigStore>, days: u32) -> Result<Config, String> {
    store.update(|c| c.set_days(days))
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
            app.manage(ConfigStore {
                path,
                config: Mutex::new(config),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_forecast,
            search_locations,
            load_config,
            set_last_location,
            toggle_favorite,
            set_days,
            day_choices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
