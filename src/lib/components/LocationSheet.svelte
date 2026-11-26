<script lang="ts">
  import type { Location } from "$lib/types";
  import { details, label, samePlace } from "$lib/format";
  import { gpsLocation, searchLocations } from "$lib/api";

  let {
    current,
    favorites,
    days,
    dayChoices,
    onpick,
    ontogglefavorite,
    ondays,
    onclose,
  }: {
    current: Location | null;
    favorites: Location[];
    days: number;
    dayChoices: number[];
    onpick: (location: Location) => void;
    ontogglefavorite: (location: Location) => void;
    ondays: (days: number) => void;
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

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose}></div>
<div class="sheet" role="dialog" aria-label="Choose location">
  <div class="grip"></div>

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

      <h3>Days to forecast</h3>
      <div class="days">
        {#each dayChoices as d}
          <button class:active={d === days} onclick={() => ondays(d)}>{d}</button>
        {/each}
      </div>
    {/if}
  </div>
</div>

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
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    z-index: 10;
    animation: fade 0.15s ease-out;
  }
  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    max-height: 85dvh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: 20px 20px 0 0;
    padding: 8px 0 calc(12px + env(safe-area-inset-bottom));
    z-index: 11;
    animation: up 0.2s ease-out;
  }
  .grip {
    width: 40px;
    height: 4px;
    border-radius: 2px;
    background: var(--line);
    margin: 0 auto 10px;
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
  .days {
    display: flex;
    gap: 8px;
  }
  .days button {
    flex: 1;
    padding: 10px 0;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    font-weight: 600;
  }
  .days button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  @keyframes up {
    from {
      transform: translateY(100%);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
</style>
