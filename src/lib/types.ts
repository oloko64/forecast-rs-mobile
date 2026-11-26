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

export interface Config {
  last_location: Location | null;
  favorites: Location[];
  days: number;
}

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
  utc_offset_seconds: number;
  timezone: string;
  daily: DayForecast[];
  hourly: HourForecast[];
}
