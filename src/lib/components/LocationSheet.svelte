<script lang="ts">
  import type { Location } from "$lib/types";
  import { details, label, samePlace } from "$lib/format";
  import { gpsLocation, searchLocations } from "$lib/api";
  import Sheet from "./Sheet.svelte";

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

  async function useGps() {
    gpsBusy = true;
    error = null;
    try {
      onpick(await gpsLocation());
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      gpsBusy = false;
    }
  }
</script>

<Sheet label="Choose location" {onclose}>

  <div class="search">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="search"
      placeholder="Search a city…"
      bind:value={query}
      oninput={onInput}
      onkeydown={(e) => e.key === "Enter" && (clearTimeout(timer), search())}
      autocomplete="off"
      enterkeyhint="search"
    />
    <button class="gps" onclick={useGps} disabled={gpsBusy} aria-label="Use my location">
      {gpsBusy ? "…" : "📍"}
    </button>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  <div class="scroll">
    {#if query.trim().length >= 2}
      <h3>Results</h3>
      {#if searching}
        <p class="hint">Searching…</p>
      {:else if results.length === 0 && !error}
        <p class="hint">No places found</p>
      {/if}
      {#each results as r (`${r.lat},${r.lon}`)}
        {@render item(r)}
      {/each}
    {:else}
      <h3>Favorites</h3>
      {#if favorites.length === 0}
        <p class="hint">Tap ☆ on a place to keep it here.</p>
      {/if}
      {#each favorites as f (`${f.lat},${f.lon}`)}
        {@render item(f)}
      {/each}
    {/if}
  </div>
</Sheet>

{#snippet item(l: Location)}
  <div class="item" class:current={current && samePlace(current, l)}>
    <button class="pick" onclick={() => onpick(l)}>
      <span class="label">{label(l)}</span>
      <span class="details">{details(l)}</span>
    </button>
    <button class="star" onclick={() => ontogglefavorite(l)} aria-label="Toggle favorite">
      {isFav(l) ? "★" : "☆"}
    </button>
  </div>
{/snippet}

<style>
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
    width: 48px;
    font-size: 20px;
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
  .star {
    width: 44px;
    height: 44px;
    font-size: 22px;
    border: none;
    background: none;
    color: var(--star);
  }
</style>
