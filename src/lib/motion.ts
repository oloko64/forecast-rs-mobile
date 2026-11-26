import { cubicInOut, cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

/** Shared timing so everything moves the same way. */
export const EASE = cubicInOut;
export const DURATION = 320;

const reduced = () =>
  typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Slides an element up from below its own height (sheets). */
export function slideUp(_node: Element, { duration = 360 } = {}): TransitionConfig {
  return {
    duration: reduced() ? 0 : duration,
    easing: cubicOut,
    css: (t, u) => `transform: translateY(${u * 100}%);`,
  };
}

/** Small rise + fade, for content that changes in place. */
export function rise(_node: Element, { y = 10, duration = DURATION, delay = 0 } = {}): TransitionConfig {
  return {
    duration: reduced() ? 0 : duration,
    delay,
    easing: EASE,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * y}px);`,
  };
}
