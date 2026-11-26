<script lang="ts">
  import { onMount } from "svelte";
  import type { Map as MapLibreMap } from "maplibre-gl";
  // Served as-is rather than through the CSS minifier, which can reorder rules.
  import mapCss from "maplibre-gl/dist/maplibre-gl.css?url";

  /**
   * A map with a fixed pin in the middle: move the map to place it.
   * Reports the pin position whenever the map stops moving.
   * Vector maps from OpenFreeMap (free, no key), OpenStreetMap data.
   */
  let {
    lat,
    lon,
    zoom = 12,
    onmove,
  }: {
    lat: number;
    lon: number;
    zoom?: number;
    onmove: (lat: number, lon: number) => void;
  } = $props();

  let container: HTMLDivElement;
  let map: MapLibreMap | undefined;
  let lifted = $state(false);
  let ready = $state(false);
  let failed = $state(false);
  let dark = $state(false);

  /** Smoothly moves the map (e.g. to a search result). */
  export function flyTo(lat: number, lon: number, zoom = 13) {
    map?.flyTo({ center: [lon, lat], zoom, duration: 1400, curve: 1.3, essential: true });
  }

  function isDark() {
    const forced = document.documentElement.dataset.theme;
    return forced ? forced === "dark" : matchMedia("(prefers-color-scheme: dark)").matches;
  }

  onMount(() => {
    let disposed = false;
    (async () => {
      const maplibregl = (await import("maplibre-gl")).default;
      if (disposed) return;
      dark = isDark();
      map = new maplibregl.Map({
        container,
        style: `https://tiles.openfreemap.org/styles/${dark ? "dark" : "liberty"}`,
        center: [lon, lat],
        zoom,
        attributionControl: { compact: true },
        dragRotate: false,
        pitchWithRotate: false,
        touchPitch: false,
        renderWorldCopies: true,
        fadeDuration: 250,
      });
      map.touchZoomRotate.disableRotation();
      map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "bottom-right");

      map.on("load", () => (ready = true));
      map.on("error", (e) => {
        // A missing tile now and then is fine; no style at all is not.
        if (!map?.isStyleLoaded()) failed = !ready;
        console.warn("map:", e.error?.message);
      });
      map.on("movestart", () => (lifted = true));
      map.on("moveend", () => {
        lifted = false;
        const c = map!.getCenter();
        onmove(c.lat, c.lng);
      });
      // The sheet animates in: measure again once it settled.
      setTimeout(() => map?.resize(), 400);
    })();
    return () => {
      disposed = true;
      map?.remove();
    };
  });
</script>

<svelte:head>
  <link rel="stylesheet" href={mapCss} />
</svelte:head>

<div class="wrap" class:ready class:dark>
  <div class="map" bind:this={container}></div>
  {#if failed}
    <p class="failed">Couldn't load the map. Check your connection.</p>
  {/if}
  <div class="pin" class:lifted aria-hidden="true">
    <svg width="40" height="48" viewBox="0 0 40 48">
      <path
        d="M20 46s-15-13-15-26a15 15 0 0 1 30 0c0 13-15 26-15 26z"
        fill="var(--accent)"
        stroke="var(--on-accent)"
        stroke-width="2.5"
      />
      <circle cx="20" cy="20" r="5.5" fill="var(--on-accent)" />
    </svg>
  </div>
  <div class="shadow" class:lifted aria-hidden="true"></div>
</div>

<style>
  .wrap {
    position: relative;
    height: 100%;
    border-radius: 16px;
    overflow: hidden;
    background: var(--card);
  }
  .map {
    opacity: 0;
    transition: opacity 0.45s ease-in-out;
  }
  .wrap.ready .map {
    opacity: 1;
  }
  .failed {
    position: absolute;
    inset: auto 16px 50% 16px;
    margin: 0;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
    z-index: 2;
  }
  .wrap :global(.maplibregl-ctrl-group) {
    background: var(--card);
    border-radius: 12px;
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.15);
  }
  .wrap :global(.maplibregl-ctrl-group button) {
    width: 38px;
    height: 38px;
  }
  .wrap :global(.maplibregl-ctrl-group button + button) {
    border-top-color: var(--line);
  }
  .wrap.dark :global(.maplibregl-ctrl-icon) {
    filter: invert(1);
  }
  .wrap :global(.maplibregl-ctrl-attrib) {
    font-size: 10px;
    background: color-mix(in srgb, var(--card) 85%, transparent);
    color: var(--muted);
  }
  .wrap :global(.maplibregl-ctrl-attrib a) {
    color: inherit;
  }
  .wrap.dark :global(.maplibregl-ctrl-attrib-button) {
    filter: invert(1);
  }
  .map {
    position: absolute;
    inset: 0;
    background: var(--card);
  }
  /* The pin's tip sits on the map center; it lifts while the map moves. */
  .pin {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 40px;
    height: 48px;
    margin: -46px 0 0 -20px;
    z-index: 3;
    pointer-events: none;
    filter: drop-shadow(0 3px 4px rgb(0 0 0 / 0.3));
    transition: transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .pin.lifted {
    transform: translateY(-12px);
    transition: transform 0.25s ease-out;
  }
  .shadow {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 14px;
    height: 5px;
    margin: -2px 0 0 -7px;
    border-radius: 50%;
    background: rgb(0 0 0 / 0.35);
    z-index: 2;
    pointer-events: none;
    transition:
      transform 0.3s ease-in-out,
      opacity 0.3s ease-in-out;
  }
  .shadow.lifted {
    transform: scale(1.6);
    opacity: 0.5;
  }
</style>
