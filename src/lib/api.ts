import { invoke } from "@tauri-apps/api/core";
import type { Config, Forecast, Location, SettingsPatch } from "./types";
import { i18n, t } from "./i18n.svelte";

export const getForecast = (location: Location, days: number) =>
  invoke<Forecast>("get_forecast", { location, days });

export const searchLocations = (query: string) =>
  invoke<Location[]>("search_locations", { query, language: i18n.lang });

export const loadConfig = () => invoke<Config>("load_config");
export const setLastLocation = (location: Location) =>
  invoke<Config>("set_last_location", { location });
export const toggleFavorite = (location: Location) =>
  invoke<Config>("toggle_favorite", { location });
export const updateSettings = (patch: SettingsPatch) =>
  invoke<Config>("update_settings", { patch });
export const dayChoices = () => invoke<number[]>("day_choices");

/** Current position as a Location. Only works on Android/iOS. */
export async function gpsLocation(): Promise<Location> {
  let geo: typeof import("@tauri-apps/plugin-geolocation");
  try {
    geo = await import("@tauri-apps/plugin-geolocation");
  } catch {
    throw new Error(t("gpsUnavailable"));
  }

  let perms = await geo.checkPermissions().catch(() => {
    throw new Error(t("gpsPhoneOnly"));
  });
  // Ask for precise (fine) location. Android also lets the user pick "approximate";
  // we still accept that, it just comes back less accurate.
  if (perms.location !== "granted") {
    perms = await geo.requestPermissions(["location"]);
    if (perms.location !== "granted" && perms.coarseLocation !== "granted") {
      throw new Error(t("permissionDenied"));
    }
  }

  const pos = await geo.getCurrentPosition({
    enableHighAccuracy: true,
    timeout: 20000,
    maximumAge: 60 * 1000,
  });
  return {
    name: t("myLocation"),
    lat: round(pos.coords.latitude),
    lon: round(pos.coords.longitude),
    // The phone is there, so its timezone is the location's.
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "auto",
  };
}

/** 4 decimals ≈ 11 m: as precise as GPS gets, and matches samePlace()'s tolerance. */
const round = (n: number) => Math.round(n * 1e4) / 1e4;

/** Names a point picked on the map and resolves its timezone (never fails). */
export const describeSpot = (lat: number, lon: number, fallbackTimezone?: string) =>
  invoke<Location>("describe_spot", {
    lat,
    lon,
    fallbackTimezone: fallbackTimezone ?? null,
    language: i18n.lang,
  });
