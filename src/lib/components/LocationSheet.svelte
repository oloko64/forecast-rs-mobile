<script lang="ts">
  import type { Location } from "$lib/types";
  import { details, label, samePlace } from "$lib/format";
  import { describeSpot, gpsLocation, searchLocations } from "$lib/api";
  import { rise } from "$lib/motion";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import Sheet from "./Sheet.svelte";
  import Icon from "./Icon.svelte";
  import MapPicker from "./MapPicker.svelte";
  import { t } from "$lib/i18n.svelte";

  let {
    current,
    favorites,
    onpick,
    ontogglefavorite,
    onclose,
  }: {
    current: Location | null;
    favorites: Location[];
    onpick: (location: Location) => void;
    ontogglefavorite: (location: Location) => void;
    onclose: () => void;
  } = $props();

  let query = $state("");
  let results = $state<Location[]>([]);
  let searching = $state(false);
  let gpsBusy = $state(false);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  // Map mode: the pin's place, and where the map opened.
  let mapOpen = $state(false);
  let mapStart = $state({ lat: 0, lon: 0, zoom: 2 });
  let spot = $state<Location | null>(null);
  let resolving = $state(false);
  let picker = $state<MapPicker>();
  let spotSeq = 0;
  let spotTimer: ReturnType<typeof setTimeout> | undefined;

  const isFav = (l: Location) => favorites.some((f) => samePlace(f, l));

  function onInput() {
    clearTimeout(timer);
    error = null;
    if (query.trim().length < 2) {
      results = [];
      searching = false;
      return;
    }
    searching = true;
    timer = setTimeout(search, 350);
  }

  async function search() {
    const mine = ++seq;
    try {
      const found = await searchLocations(query);
      if (mine === seq) results = found;
    } catch (e) {
      if (mine === seq) error = String(e);
    } finally {
      if (mine === seq) searching = false;
    }
  }

  function clearSearch() {
    query = "";
    results = [];
    searching = false;
    seq++;
  }

  async function useGps() {
    gpsBusy = true;
    error = null;
    try {
      const here = await gpsLocation();
      if (mapOpen) moveMapTo(here, 16);
      else onpick(here);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      gpsBusy = false;
    }
  }

  /** Opens the map on `at` (a result, the current place, or the world). */
  function openMap(at: Location | null) {
    const start = at ?? current;
    mapStart = start ? { lat: start.lat, lon: start.lon, zoom: at ? 13 : 11 } : { lat: 15, lon: 0, zoom: 2 };
    spot = start;
    clearSearch();
    mapOpen = true;
  }

  function moveMapTo(l: Location, zoom = 13) {
    clearTimeout(spotTimer);
    spot = l;
    clearSearch();
    picker?.flyTo(l.lat, l.lon, zoom);
  }

  /** The map stopped: name the spot under the pin, unless it is where we are. */
  function onMapMove(lat: number, lon: number) {
    if (spot && Math.abs(spot.lat - lat) < 1e-4 && Math.abs(spot.lon - lon) < 1e-4) return;
    clearTimeout(spotTimer);
    resolving = true;
    const mine = ++spotSeq;
    // Debounced: OpenStreetMap allows about one lookup per second.
    spotTimer = setTimeout(async () => {
      const found = await describeSpot(lat, lon, spot?.timezone ?? current?.timezone);
      if (mine === spotSeq) {
        spot = found;
        resolving = false;
      }
    }, 700);
  }

  function useSpot() {
    if (spot && !resolving) onpick(spot);
  }
</script>

<Sheet label={t("chooseLocation")} {onclose}>
  <div class="search">
    {#if mapOpen}
      <button class="square" onclick={() => (mapOpen = false)} aria-label={t("backToList")} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }}>
        <span class="back"><Icon name="chevron-down" /></span>
      </button>
    {/if}
    <input
      type="search"
      placeholder={mapOpen ? t("searchMap") : t("searchCity")}
      bind:value={query}
      oninput={onInput}
      onkeydown={(e) => e.key === "Enter" && (clearTimeout(timer), search())}
      autocomplete="off"
      enterkeyhint="search"
    />
    <button class="square gps" onclick={useGps} disabled={gpsBusy} aria-label={t("useMyLocation")}>
      <span class:busy={gpsBusy}><Icon name="pin" /></span>
    </button>
  </div>

  {#if error}<p class="error" transition:slide={{ duration: 250 }}>{error}</p>{/if}

  {#if mapOpen}
    <div class="map-mode" in:rise={{ y: 16, duration: 380 }}>
      <div class="map-area">
        <MapPicker bind:this={picker} lat={mapStart.lat} lon={mapStart.lon} zoom={mapStart.zoom} onmove={onMapMove} />

        {#if query.trim().length >= 2}
          <div class="overlay" transition:slide={{ duration: 250, easing: cubicInOut }}>
            {#if searching}
              <p class="hint">{t("searching")}</p>
            {:else if results.length === 0 && !error}
              <p class="hint">{t("noPlaces")}</p>
            {/if}
            {#each results as r (`${r.lat},${r.lon}`)}
              <button class="pick" onclick={() => moveMapTo(r)}>
                <span class="label">{label(r)}</span>
                <span class="details">{details(r)}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <div class="spot">
        <div class="spot-text">
          {#if resolving}
            <span class="label muted">{t("findingPlace")}</span>
            <span class="details">&nbsp;</span>
          {:else if spot}
            {#key `${spot.lat},${spot.lon}`}
              <span class="label" in:rise={{ y: 4, duration: 250 }}>{label(spot)}</span>
              <span class="details" in:rise={{ y: 4, duration: 250 }}>{details(spot)}</span>
            {/key}
          {:else}
            <span class="label muted">{t("movePin")}</span>
          {/if}
        </div>
        <button class="use" onclick={useSpot} disabled={!spot || resolving}>{t("useThisLocation")}</button>
      </div>
    </div>
  {:else}
    <div class="scroll" in:rise={{ y: 10, duration: 300 }}>
      <button class="map-entry" onclick={() => openMap(null)}>
        <span class="map-glyph"><Icon name="pin" size={20} /></span>
        <span>
          <span class="label">{t("pickOnMap")}</span>
          <span class="details">{t("pickOnMapHint")}</span>
        </span>
      </button>

      {#if query.trim().length >= 2}
        <h3>{t("results")}</h3>
        {#if searching}
          <p class="hint">{t("searching")}</p>
        {:else if results.length === 0 && !error}
          <p class="hint">{t("noPlaces")}</p>
        {/if}
        {#each results as r (`${r.lat},${r.lon}`)}
          {@render item(r)}
        {/each}
      {:else}
        <h3>{t("favorites")}</h3>
        {#if favorites.length === 0}
          <p class="hint">{t("favoritesHint")}</p>
        {/if}
        {#each favorites as f (`${f.lat},${f.lon}`)}
          {@render item(f)}
        {/each}
      {/if}
    </div>
  {/if}
</Sheet>

{#snippet item(l: Location)}
  <div class="item" class:current={current && samePlace(current, l)}>
    <button class="pick" onclick={() => onpick(l)}>
      <span class="label">{label(l)}</span>
      <span class="details">{details(l)}</span>
    </button>
    <button class="star map-btn" onclick={() => openMap(l)} aria-label={t("adjustOnMap")}>
      <Icon name="map" size={21} />
    </button>
    <button class="star" onclick={() => ontogglefavorite(l)} aria-label={t("toggleFavorite")}>
      <Icon name={isFav(l) ? "star-filled" : "star"} size={22} />
    </button>
  </div>
{/snippet}

<style>
  .square {
    display: grid;
    place-items: center;
    width: 48px;
    flex: none;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
  }
  .back {
    display: flex;
    transform: rotate(90deg);
  }
  .map-btn {
    color: var(--muted) !important;
  }
  .map-entry {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 6px;
    padding: 12px;
    border-radius: 14px;
    border: 1px dashed var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    text-align: left;
  }
  .map-entry > span:last-child {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .map-glyph {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: var(--highlight);
    color: var(--accent);
  }
  .map-mode {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 0 16px;
    min-height: 0;
  }
  .map-area {
    position: relative;
    height: min(56dvh, 480px);
  }
  .overlay {
    position: absolute;
    top: 8px;
    left: 8px;
    right: 8px;
    max-height: 70%;
    overflow-y: auto;
    z-index: 1000;
    padding: 4px 12px;
    border-radius: 14px;
    background: var(--bg);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.2);
  }
  .overlay .pick {
    width: 100%;
    border-bottom: 1px solid var(--line);
  }
  .spot {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .spot-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .spot-text .label,
  .spot-text .details {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .muted {
    color: var(--muted);
    font-weight: 500 !important;
  }
  .use {
    flex: none;
    padding: 12px 16px;
    border-radius: 12px;
    border: none;
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    font-weight: 700;
    transition:
      opacity 0.3s ease-in-out,
      background-color 0.3s ease-in-out,
      transform 0.18s ease-in-out;
  }
  .use:disabled {
    opacity: 0.45;
  }
  .search {
    display: flex;
    gap: 8px;
    padding: 0 16px 8px;
  }
  input {
    flex: 1;
    min-width: 0;
    font: inherit;
    font-size: 16px;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
  }
  .gps {
    display: grid;
    place-items: center;
    color: var(--accent);
    width: 48px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--card);
  }
  .scroll {
    overflow-y: auto;
    padding: 0 16px;
  }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin: 14px 0 6px;
  }
  .hint {
    color: var(--muted);
    font-size: 14px;
    margin: 6px 0;
  }
  .error {
    color: var(--error);
    font-size: 14px;
    margin: 0 16px 4px;
  }
  .item {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--line);
  }
  .item.current .label {
    color: var(--accent);
  }
  .pick {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 10px 0;
    border: none;
    background: none;
    color: var(--fg);
    font: inherit;
    text-align: left;
  }
  .label {
    font-weight: 600;
  }
  .details {
    font-size: 12px;
    color: var(--muted);
  }
  .busy {
    display: flex;
    animation: bob 0.9s ease-in-out infinite alternate;
  }
  @keyframes bob {
    to {
      transform: translateY(-4px);
    }
  }
  .star {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: none;
    background: none;
    color: var(--star);
  }
</style>
