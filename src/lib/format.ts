import type { HourForecast, Location } from "./types";
import { prefs } from "./prefs.svelte";
import { i18n } from "./i18n.svelte";

/** Decimal comma in Portuguese: "8,4". */
const decimal = (text: string) => (i18n.lang === "pt-BR" ? text.replace(".", ",") : text);

/** "Araraquara, Sao Paulo, Brazil" */
export function label(l: Location): string {
  return [l.name, l.region, l.country].filter((p) => p && p.length > 0).join(", ");
}

/** "21.79°S 48.18°W · Araraquara · pop 168k" */
export function details(l: Location): string {
  // Portuguese: decimal comma, O(este)/L(este) for west/east.
  const pt = i18n.lang === "pt-BR";
  const lat = `${decimal(Math.abs(l.lat).toFixed(2))}°${l.lat < 0 ? "S" : "N"}`;
  const lon = `${decimal(Math.abs(l.lon).toFixed(2))}°${l.lon < 0 ? (pt ? "O" : "W") : pt ? "L" : "E"}`;
  const parts = [`${lat} ${lon}`];
  if (l.district) parts.push(l.district);
  if (l.population && l.population > 0) parts.push(population(l.population));
  return parts.join(" · ");
}

/** "pop 168k" / "168 mil hab." */
function population(p: number): string {
  if (i18n.lang === "pt-BR") {
    if (p < 1_000) return `${p} hab.`;
    if (p < 1_000_000) return `${(p / 1e3).toFixed(0)} mil hab.`;
    return `${(p / 1e6).toFixed(1).replace(".", ",")} mi hab.`;
  }
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

const WEEKDAYS = {
  en: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
  "pt-BR": ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"],
};
const MONTHS = {
  en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
  "pt-BR": ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"],
};

/** Parses "YYYY-MM-DD" without timezone shifts. */
function parseDate(date: string): Date {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(Date.UTC(y, m - 1, d));
}

export const weekday = (date: string) => WEEKDAYS[i18n.lang][parseDate(date).getUTCDay()];
export const dayOfMonth = (date: string) => parseDate(date).getUTCDate();
export function longDate(date: string): string {
  const d = parseDate(date);
  const day = WEEKDAYS[i18n.lang][d.getUTCDay()];
  const month = MONTHS[i18n.lang][d.getUTCMonth()];
  // "Thu, Nov 27" / "Qui, 27 de nov"
  return i18n.lang === "pt-BR" ? `${day}, ${d.getUTCDate()} de ${month}` : `${day}, ${month} ${d.getUTCDate()}`;
}

/** Converts from °C, the unit both providers use, to the chosen one. */
export const toUnit = (celsius: number) =>
  prefs.units === "fahrenheit" ? (celsius * 9) / 5 + 32 : celsius;
export const unitLetter = () => (prefs.units === "fahrenheit" ? "F" : "C");

/** "26°" */
export const temp = (t: number) => `${Math.round(toUnit(t))}°`;
/** "26°C" */
export const tempUnit = (t: number) => `${Math.round(toUnit(t))}°${unitLetter()}`;
export const mm = (p: number) => decimal(p >= 10 ? p.toFixed(0) : p.toFixed(1));
