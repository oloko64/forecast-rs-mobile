# forecast-mobile

Android hourly weather forecast built with Tauri 2 + SvelteKit, powered by [Open-Meteo](https://open-meteo.com/) (no API key).
Mobile sibling of [cli-forecast-rs](../forecast-rs): the Rust networking/config code is ported from it.

## Features (mk1)

- Day strip, hourly temperature + rain chart (tap/drag to pick an hour), hourly list with rain probability
- City search (Open-Meteo geocoding), favorites, precise GPS location
- 3 / 7 / 10 / 16 day forecasts, pull to refresh, auto refresh after 30 min in background
- Remembers the last location, favorites and day count (`config.json` in the app data dir)

## Stack

- `src-tauri/` — Rust: Open-Meteo + geocoding over **reqwest + rustls (ring)** with bundled Mozilla roots (`webpki-roots`).
  No OpenSSL / aws-lc / platform verifier, so Android cross-builds need nothing beyond the NDK.
- `src/` — SvelteKit (static SPA, Svelte 5). All HTTP goes through Rust; the webview CSP blocks network access.

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
