import type { HourForecast, Location } from "./types";

/** "Araraquara, Sao Paulo, Brazil" */
export function label(l: Location): string {
  return [l.name, l.region, l.country].filter((p) => p && p.length > 0).join(", ");
}

/** "21.79°S 48.18°W · Araraquara · pop 168k" */
export function details(l: Location): string {
  const lat = `${Math.abs(l.lat).toFixed(2)}°${l.lat < 0 ? "S" : "N"}`;
  const lon = `${Math.abs(l.lon).toFixed(2)}°${l.lon < 0 ? "W" : "E"}`;
  const parts = [`${lat} ${lon}`];
  if (l.district) parts.push(l.district);
  if (l.population && l.population > 0) parts.push(population(l.population));
  return parts.join(" · ");
}

function population(p: number): string {
  if (p < 1_000) return `pop ${p}`;
  if (p < 1_000_000) return `pop ${(p / 1e3).toFixed(0)}k`;
  return `pop ${(p / 1e6).toFixed(1)}M`;
}

export const samePlace = (a: Location, b: Location) =>
  Math.abs(a.lat - b.lat) < 1e-4 && Math.abs(a.lon - b.lon) < 1e-4;

/** "YYYY-MM-DDTHH" of the current hour at a location with the given UTC offset. */
export function nowHourKey(utcOffsetSeconds: number, now = Date.now()): string {
  return new Date(now + utcOffsetSeconds * 1000).toISOString().slice(0, 13);
}

export const hourKey = (h: HourForecast) => h.time.slice(0, 13);
export const dateOf = (h: HourForecast) => h.time.slice(0, 10);
export const hourOf = (h: HourForecast) => h.time.slice(11, 16);

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** Parses "YYYY-MM-DD" without timezone shifts. */
function parseDate(date: string): Date {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(Date.UTC(y, m - 1, d));
}

export const weekday = (date: string) => WEEKDAYS[parseDate(date).getUTCDay()];
export const dayOfMonth = (date: string) => parseDate(date).getUTCDate();
export function longDate(date: string): string {
  const d = parseDate(date);
  return `${WEEKDAYS[d.getUTCDay()]}, ${MONTHS[d.getUTCMonth()]} ${d.getUTCDate()}`;
}

export const temp = (t: number) => `${Math.round(t)}°`;
export const mm = (p: number) => (p >= 10 ? p.toFixed(0) : p.toFixed(1));
