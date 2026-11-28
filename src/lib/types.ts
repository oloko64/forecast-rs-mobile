// Mirrors the Rust types in src-tauri/src/{location,config,types}.rs.

export interface Location {
  name: string;
  region?: string;
  country?: string;
  district?: string;
  population?: number;
  lat: number;
  lon: number;
  /** IANA name, or "auto" to let Open-Meteo resolve it from the coordinates. */
  timezone: string;
}

export type ThemeMode = "system" | "light" | "dark";
export type Palette = "ocean" | "forest" | "sunset";
export type Provider = "open_meteo" | "met_norway";
export type Units = "celsius" | "fahrenheit";
/** "system" follows the phone. */
export type Language = "system" | "en" | "pt-BR";

/** Overrides for a palette's colors ("#rrggbb"); missing ones keep the preset. */
export interface PaletteColors {
  accent?: string;
  temp?: string;
  rain?: string;
}

export const PROVIDER_NAMES: Record<Provider, string> = {
  open_meteo: "Open-Meteo",
  met_norway: "MET Norway",
};

export interface Config {
  last_location: Location | null;
  favorites: Location[];
  days: number;
  theme_mode: ThemeMode;
  palette: Palette;
  /** Preferred forecast provider; the other one is the fallback. */
  provider: Provider;
  units: Units;
  custom_colors: Partial<Record<Palette, PaletteColors>>;
  /** Auto-refresh interval while the app is open (10 to 120). */
  refresh_minutes: number;
  /** On coming back, refresh if the forecast is older than this (15 to 120). */
  resume_minutes: number;
  language: Language;
  /** Home-screen widget background refresh (15 to 240 minutes). */
  widget_refresh_minutes: number;
  /** What the home-screen widget shows besides the temperature line. */
  widget: WidgetOptions;
}

export interface WidgetOptions {
  location: boolean;
  now: boolean;
  summary: boolean;
  day_picker: boolean;
  /** A day picked with the arrows goes back to today after `return_after_seconds`. */
  return_to_today: boolean;
  /** 30 s to 20 min. */
  return_after_seconds: number;
  refresh_button: boolean;
  rain: boolean;
  clock: "off" | "small" | "big";
  /** Background opacity in percent; 0 is fully transparent. */
  background: number;
}

export type SettingsPatch = Partial<Pick<Config, "days" | "theme_mode" | "palette" | "provider" | "units" | "refresh_minutes" | "resume_minutes" | "language" | "widget_refresh_minutes" | "widget">> & {
  /** Replaces a palette's custom colors; empty resets it to the preset. */
  custom_colors?: [Palette, PaletteColors];
};

export type WeatherKind = "clear" | "cloudy" | "fog" | "rain" | "snow" | "storm" | "unknown";

export interface Weather {
  description: string;
  icon: string;
  kind: WeatherKind;
}

export interface HourForecast {
  /** Local time at the location, "YYYY-MM-DDTHH:MM:SS". */
  time: string;
  temperature: number;
  weather: Weather;
  precipitation: number;
  precipitation_probability: number | null;
}

export interface DayForecast {
  /** "YYYY-MM-DD" */
  date: string;
  temperature_max: number;
  temperature_min: number;
  weather: Weather;
  precipitation: number;
}

export interface Forecast {
  /** The provider that answered. */
  source: Provider;
  /** True when `source` is the fallback because the preferred provider kept failing. */
  fallback: boolean;
  utc_offset_seconds: number;
  timezone: string;
  daily: DayForecast[];
  hourly: HourForecast[];
}
