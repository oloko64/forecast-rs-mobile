import type { Units } from "./types";

/** Display preferences read by the formatters; reactive, so labels update on change. */
export const prefs = $state<{ units: Units }>({ units: "celsius" });
