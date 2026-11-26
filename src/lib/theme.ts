import type { Palette, PaletteColors, ThemeMode } from "./types";
import type { Key } from "./i18n.svelte";

const CACHE_KEY = "theme";

export type ColorRole = keyof PaletteColors;

export const COLOR_ROLES: { id: ColorRole; name: Key }[] = [
  { id: "accent", name: "roleAccent" },
  { id: "temp", name: "roleTemp" },
  { id: "rain", name: "roleRain" },
];

/** Preset colors (light variant), shown in settings and used as picker defaults. */
export const PALETTES: { id: Palette; name: Key; colors: Required<PaletteColors> }[] = [
  { id: "ocean", name: "paletteOcean", colors: { accent: "#3867f0", temp: "#f08a24", rain: "#2f8ff5" } },
  { id: "forest", name: "paletteForest", colors: { accent: "#23845a", temp: "#d4900c", rain: "#1f8aa3" } },
  { id: "sunset", name: "paletteSunset", colors: { accent: "#c2417a", temp: "#ea5f2b", rain: "#6a5acd" } },
];

export function paletteColors(palette: Palette, custom?: PaletteColors): Required<PaletteColors> {
  const preset = PALETTES.find((p) => p.id === palette)!.colors;
  return { ...preset, ...stripEmpty(custom) };
}

const stripEmpty = (c?: PaletteColors) =>
  Object.fromEntries(Object.entries(c ?? {}).filter(([, v]) => v)) as PaletteColors;

/** Text color readable on `hex`. */
function onColor(hex: string): string {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const luminance = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  return luminance > 0.4 ? "#10131a" : "#ffffff";
}

export function applyTheme(mode: ThemeMode, palette: Palette, custom?: PaletteColors) {
  const root = document.documentElement;
  if (mode === "system") delete root.dataset.theme;
  else root.dataset.theme = mode;
  root.dataset.palette = palette;

  // Custom colors override the preset's light/dark pair with one color.
  const c = stripEmpty(custom);
  for (const role of ["accent", "temp", "rain"] as const) {
    if (c[role]) root.style.setProperty(`--${role}`, c[role]!);
    else root.style.removeProperty(`--${role}`);
  }
  if (c.accent) root.style.setProperty("--on-accent", onColor(c.accent));
  else root.style.removeProperty("--on-accent");

  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ mode, palette, custom: c }));
  } catch {
    // Storage unavailable: the theme still applies once the config loads.
  }
}

export function applyCachedTheme() {
  try {
    const cached = JSON.parse(localStorage.getItem(CACHE_KEY) ?? "null");
    if (cached?.mode && cached?.palette) applyTheme(cached.mode, cached.palette, cached.custom);
  } catch {
    // Ignore a missing or broken cache.
  }
}

/** "#rrggbb" ⇄ HSL (h 0-360, s/l 0-100), for the color picker. */
export function hexToHsl(hex: string): [number, number, number] {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const d = max - min;
  if (d === 0) return [0, 0, Math.round(l * 100)];
  const s = d / (1 - Math.abs(2 * l - 1));
  let h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  h = (h * 60 + 360) % 360;
  return [Math.round(h), Math.round(s * 100), Math.round(l * 100)];
}

export function hslToHex(h: number, s: number, l: number): string {
  const sat = s / 100;
  const light = l / 100;
  const k = (n: number) => (n + h / 30) % 12;
  const a = sat * Math.min(light, 1 - light);
  const f = (n: number) => light - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return "#" + [f(0), f(8), f(4)].map((x) => Math.round(x * 255).toString(16).padStart(2, "0")).join("");
}
