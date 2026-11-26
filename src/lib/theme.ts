import type { Palette, ThemeMode } from "./types";

const CACHE_KEY = "theme";

export const PALETTES: { id: Palette; name: string; colors: [string, string, string] }[] = [
  { id: "ocean", name: "Ocean", colors: ["#3867f0", "#f08a24", "#2f8ff5"] },
  { id: "forest", name: "Forest", colors: ["#23845a", "#d4900c", "#1f8aa3"] },
  { id: "sunset", name: "Sunset", colors: ["#c2417a", "#ea5f2b", "#6a5acd"] },
];

export function applyTheme(mode: ThemeMode, palette: Palette) {
  const root = document.documentElement;
  if (mode === "system") delete root.dataset.theme;
  else root.dataset.theme = mode;
  root.dataset.palette = palette;
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ mode, palette }));
  } catch {
    // Storage unavailable: the theme still applies once the config loads.
  }
}

export function applyCachedTheme() {
  try {
    const cached = JSON.parse(localStorage.getItem(CACHE_KEY) ?? "null");
    if (cached?.mode && cached?.palette) applyTheme(cached.mode, cached.palette);
  } catch {
    // Ignore a missing or broken cache.
  }
}
