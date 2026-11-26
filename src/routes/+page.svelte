<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "$lib/api";
  import type { Config, Forecast, Location } from "$lib/types";
  import { dateOf, hourKey, hourOf, label, longDate, mm, nowHourKey, samePlace, temp } from "$lib/format";
  import DayStrip from "$lib/components/DayStrip.svelte";
  import HourChart from "$lib/components/HourChart.svelte";
  import HourList from "$lib/components/HourList.svelte";
  import LocationSheet from "$lib/components/LocationSheet.svelte";

  /** Refresh automatically when the app comes back after this long. */
  const STALE_MS = 30 * 60 * 1000;
  const PULL_TRIGGER = 70;

  let config = $state<Config>({ last_location: null, favorites: [], days: 7 });
  let dayChoices = $state<number[]>([3, 7, 10, 16]);
  let location = $state<Location | null>(null);
  let forecast = $state<Forecast | null>(null);
  let fetchedAt = 0;
  let loading = $state(false);
  let error = $state<string | null>(null);
  let sheetOpen = $state(false);
  /** Index into forecast.hourly. */
  let selected = $state(0);
  let clock = $state(Date.now());

  const hourly = $derived(forecast?.hourly ?? []);
  const nowKey = $derived(forecast ? nowHourKey(forecast.utc_offset_seconds, clock) : "");
  const today = $derived(nowKey.slice(0, 10));
  const selectedHour = $derived(hourly[selected]);
  const selectedDate = $derived(selectedHour ? dateOf(selectedHour) : "");
  const dayStart = $derived(hourly.findIndex((h) => dateOf(h) === selectedDate));
  const dayHours = $derived(hourly.filter((h) => dateOf(h) === selectedDate));
  const selectedDay = $derived(forecast?.daily.find((d) => d.date === selectedDate));
  const isFavorite = $derived(!!location && config.favorites.some((f) => samePlace(f, location!)));

  function nowIndex(): number {
    if (!forecast) return 0;
    const i = hourly.findIndex((h) => hourKey(h) === nowHourKey(forecast!.utc_offset_seconds));
    return i < 0 ? 0 : i;
  }

  async function load(target: Location, { keepSelection = false } = {}) {
    loading = true;
    error = null;
    const keep = keepSelection && selectedHour ? selectedHour.time : null;
    try {
      const result = await api.getForecast(target, config.days);
      forecast = result;
      location = target;
      fetchedAt = Date.now();
      clock = fetchedAt;
      const kept = keep ? result.hourly.findIndex((h) => h.time === keep) : -1;
      selected = kept >= 0 ? kept : nowIndex();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function pick(target: Location) {
    sheetOpen = false;
    await load(target);
    if (location && samePlace(location, target)) {
      config = await api.setLastLocation(target).catch(() => config);
    }
  }

  const refresh = () => location && !loading && load(location, { keepSelection: true });

  async function toggleFavorite(target: Location) {
    try {
      config = await api.toggleFavorite(target);
    } catch (e) {
      error = String(e);
    }
  }

  async function changeDays(days: number) {
    config = await api.setDays(days);
    if (location) await load(location, { keepSelection: true });
  }

  function selectDay(date: string) {
    const start = hourly.findIndex((h) => dateOf(h) === date);
    if (start < 0) return;
    // Keep the same time of day when switching days, or "now" when going to today.
    const offset = date === today ? nowIndex() - start : selected - dayStart;
    const count = hourly.filter((h) => dateOf(h) === date).length;
    selected = start + Math.min(Math.max(offset, 0), count - 1);
  }

  // Pull to refresh.
  let pullStart: number | null = null;
  let pull = $state(0);
  function touchStart(e: TouchEvent) {
    pullStart = window.scrollY <= 0 && !sheetOpen ? e.touches[0].clientY : null;
  }
  function touchMove(e: TouchEvent) {
    if (pullStart === null) return;
    pull = Math.max(0, Math.min(110, (e.touches[0].clientY - pullStart) * 0.5));
  }
  function touchEnd() {
    if (pull >= PULL_TRIGGER) refresh();
    pull = 0;
    pullStart = null;
  }

  onMount(() => {
    const tick = setInterval(() => (clock = Date.now()), 60_000);
    const onVisible = () => {
      if (document.visibilityState !== "visible") return;
      clock = Date.now();
      if (Date.now() - fetchedAt > STALE_MS) refresh();
    };
    document.addEventListener("visibilitychange", onVisible);

    (async () => {
      try {
        [config, dayChoices] = await Promise.all([api.loadConfig(), api.dayChoices()]);
      } catch (e) {
        error = String(e);
      }
      if (config.last_location) {
        await load(config.last_location);
      } else {
        // First run: try GPS, fall back to the search sheet.
        try {
          await pick(await api.gpsLocation());
        } catch {
          sheetOpen = true;
        }
      }
    })();

    return () => {
      clearInterval(tick);
      document.removeEventListener("visibilitychange", onVisible);
    };
  });
</script>

<svelte:window ontouchstart={touchStart} ontouchmove={touchMove} ontouchend={touchEnd} />

<main>
  <div class="pull" style:height="{pull}px" style:opacity={pull / PULL_TRIGGER}>
    <span class:ready={pull >= PULL_TRIGGER}>↻</span>
  </div>

  <header>
    <button class="place" onclick={() => (sheetOpen = true)}>
      <span class="name">{location ? location.name : "Choose a location"} <span class="caret">▾</span></span>
      {#if location}<span class="sub">{label(location).split(", ").slice(1).join(", ") || forecast?.timezone}</span>{/if}
    </button>
    {#if location}
      <button class="icon-btn star" onclick={() => toggleFavorite(location!)} aria-label="Toggle favorite">
        {isFavorite ? "★" : "☆"}
      </button>
    {/if}
    <button class="icon-btn" class:spin={loading} onclick={refresh} disabled={!location} aria-label="Refresh">↻</button>
  </header>

  {#if error}
    <div class="banner">
      <span>{error}</span>
      {#if location}<button onclick={refresh}>Retry</button>{/if}
    </div>
  {/if}

  {#if forecast && selectedHour}
    <section class="hero">
      <div class="big-icon">{selectedHour.weather.icon}</div>
      <div class="hero-main">
        <div class="hero-temp">{temp(selectedHour.temperature)}</div>
        <div class="hero-desc">{selectedHour.weather.description}</div>
        <div class="hero-meta">
          {selectedDate === today ? "Today" : longDate(selectedDate)} · {hourOf(selectedHour)}
          {#if hourKey(selectedHour) !== nowKey}
            <button class="now-btn" onclick={() => (selected = nowIndex())}>Now</button>
          {/if}
        </div>
        {#if selectedDay}
          <div class="hero-meta">
            ↑ {temp(selectedDay.temperature_max)} ↓ {temp(selectedDay.temperature_min)} · 💧 {mm(selectedDay.precipitation)} mm
          </div>
        {/if}
      </div>
    </section>

    <DayStrip days={forecast.daily} selected={selectedDate} {today} onselect={selectDay} />

    <section class="card">
      <HourChart hours={dayHours} selected={selected - dayStart} onselect={(i) => (selected = dayStart + i)} />
    </section>

    <section class="card list">
      <HourList hours={dayHours} selected={selected - dayStart} {nowKey} onselect={(i) => (selected = dayStart + i)} />
    </section>

    <footer>Weather data by Open-Meteo.com</footer>
  {:else if loading}
    <p class="placeholder">Loading forecast…</p>
  {:else if !location}
    <p class="placeholder">Search a city or use 📍 GPS to get started.</p>
  {/if}
</main>

{#if sheetOpen}
  <LocationSheet
    current={location}
    favorites={config.favorites}
    days={config.days}
    {dayChoices}
    onpick={pick}
    ontogglefavorite={toggleFavorite}
    ondays={changeDays}
    onclose={() => (sheetOpen = false)}
  />
{/if}

<style>
  :global(:root) {
    --bg: #f4f6fa;
    --card: #ffffff;
    --fg: #172033;
    --muted: #66728a;
    --line: #e1e6ef;
    --highlight: #e8eefc;
    --accent: #3867f0;
    --on-accent: #ffffff;
    --temp: #f08a24;
    --rain: #2f8ff5;
    --star: #f2b01e;
    --error: #d93a3a;
    color-scheme: light;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #0f141d;
      --card: #182030;
      --fg: #e9eef7;
      --muted: #8e9ab0;
      --line: #263147;
      --highlight: #22304d;
      --accent: #6b8cff;
      --on-accent: #0b1020;
      --temp: #ffa14a;
      --rain: #5aa9ff;
      --star: #ffc93d;
      --error: #ff6b6b;
      color-scheme: dark;
    }
  }
  :global(*) {
    box-sizing: border-box;
    -webkit-tap-highlight-color: transparent;
  }
  :global(button) {
    cursor: pointer;
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--fg);
    font-family: system-ui, -apple-system, Roboto, "Segoe UI", sans-serif;
    font-size: 15px;
    overscroll-behavior-y: contain;
  }
  main {
    max-width: 640px;
    margin: 0 auto;
    padding-top: env(safe-area-inset-top);
    padding-bottom: calc(16px + env(safe-area-inset-bottom));
  }
  .pull {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    font-size: 22px;
    color: var(--muted);
  }
  .pull .ready {
    color: var(--accent);
  }
  header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 12px 8px 4px 16px;
  }
  .place {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    border: none;
    background: none;
    color: var(--fg);
    font: inherit;
    padding: 0;
    text-align: left;
  }
  .name {
    font-size: 22px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .caret {
    font-size: 14px;
    color: var(--muted);
  }
  .sub {
    font-size: 13px;
    color: var(--muted);
  }
  .icon-btn {
    width: 44px;
    height: 44px;
    border: none;
    background: none;
    color: var(--fg);
    font-size: 22px;
    border-radius: 50%;
  }
  .icon-btn.star {
    color: var(--star);
  }
  .icon-btn:disabled {
    opacity: 0.3;
  }
  .spin {
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 8px 16px;
    padding: 10px 14px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--error) 15%, var(--card));
    color: var(--error);
    font-size: 14px;
  }
  .banner span {
    flex: 1;
  }
  .banner button {
    border: 1px solid currentColor;
    background: none;
    color: inherit;
    border-radius: 8px;
    padding: 4px 10px;
    font: inherit;
  }
  .hero {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 16px 16px;
  }
  .big-icon {
    font-size: 72px;
    line-height: 1;
  }
  .hero-temp {
    font-size: 52px;
    font-weight: 300;
    line-height: 1;
  }
  .hero-desc {
    font-size: 17px;
    font-weight: 600;
    margin: 4px 0;
  }
  .hero-meta {
    font-size: 13px;
    color: var(--muted);
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
  }
  .now-btn {
    border: none;
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    font-size: 12px;
    font-weight: 700;
    border-radius: 8px;
    padding: 2px 10px;
  }
  .card {
    background: var(--card);
    border-radius: 18px;
    margin: 8px 16px;
    padding: 12px 0;
    overflow: hidden;
  }
  .card.list {
    padding: 0;
  }
  .placeholder {
    text-align: center;
    color: var(--muted);
    margin-top: 30vh;
    padding: 0 24px;
  }
  footer {
    text-align: center;
    font-size: 11px;
    color: var(--muted);
    padding: 12px;
  }
</style>
