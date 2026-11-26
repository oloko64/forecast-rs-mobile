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
}

export type SettingsPatch = Partial<Pick<Config, "days" | "theme_mode" | "palette" | "provider">>;

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
