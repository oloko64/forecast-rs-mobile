# forecast-mobile

Android hourly weather forecast built with Tauri 2 + SvelteKit, powered by [Open-Meteo](https://open-meteo.com/) (no API key).
Mobile sibling of [cli-forecast-rs](../forecast-rs): the Rust networking/config code is ported from it.

## Features (mk1)

- Day strip, hourly temperature + rain chart (tap/drag to pick an hour), and an hourly list with its own scroll
  region that brings the selected hour to the top
- City search (Open-Meteo geocoding), favorites, precise GPS location
- Map picker: open any result on a map and move it under the pin to place the location exactly; the spot is
  named via OpenStreetMap Nominatim and its timezone resolved via Open-Meteo. Maps by OpenFreeMap (MapLibre,
  no API key)
- 3 / 7 / 10 / 16 day forecasts, pull to refresh, auto refresh while open (10 min to 2 h, default 15 min) and on
  coming back when older than a set age (15 min to 2 h, default 30 min)
- Settings: appearance (system / light / dark), color theme (Ocean, Forest, Sunset, each customizable),
  °C/°F, days, refresh intervals, weather provider
- Two providers: Open-Meteo (default) and MET Norway. After 3 consecutive failures the other one is used
  for 10 minutes, and the app says so
- English and Brazilian Portuguese (follows the phone by default, or pick one in settings); place names, dates,
  decimals and coordinates are localized too
- Remembers the last location, favorites and settings (`config.json` in the app data dir)

## Stack

- `src-tauri/` — Rust: Open-Meteo, MET Norway (converted to an hourly local-time grid) and geocoding over **reqwest + rustls (ring)** with bundled Mozilla roots (`webpki-roots`).
  No OpenSSL / aws-lc / platform verifier, so Android cross-builds need nothing beyond the NDK.
- `src/` — SvelteKit (static SPA, Svelte 5). Weather and place lookups go through Rust; the webview CSP only allows
  map tiles/styles from `tiles.openfreemap.org`.

## Develop

Requires Rust, bun, Android SDK + NDK (`ANDROID_HOME`, `NDK_HOME`) and a JDK Gradle supports — use Android Studio's bundled one (`/opt/android-studio/jbr`, JDK 25). After switching JDKs, run `./gradlew --stop` in `src-tauri/gen/android` once.
Needs Node ≥ 20 on `PATH` (`.nvmrc` pins 26: `nvm use`) — Vite 8 does not run on Node 16.

```sh
bun install
cargo tauri dev                                       # desktop preview (GPS unavailable)
JAVA_HOME=/opt/android-studio/jbr cargo tauri android dev
JAVA_HOME=/opt/android-studio/jbr cargo tauri android build --debug --apk --target aarch64
cd src-tauri && cargo test                            # unit tests
cd src-tauri && cargo test -- --ignored               # live API test over rustls
```
