<script lang="ts">
  import { onMount, untrack } from "svelte";
  import * as api from "$lib/api";
  import {
    PROVIDER_NAMES,
    type Config,
    type Forecast,
    type Location,
    type PaletteColors,
    type SettingsPatch,
  } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import { prefs } from "$lib/prefs.svelte";
  import { i18n, resolveLang, t, weatherText } from "$lib/i18n.svelte";
  import { Tween } from "svelte/motion";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { rise } from "$lib/motion";
  import {
    dateOf,
    hourKey,
    hourOf,
    label,
    longDate,
    mm,
    nowHourKey,
    samePlace,
    tempUnit,
    toUnit,
    unitLetter,
  } from "$lib/format";
  import Icon from "$lib/components/Icon.svelte";
  import RefreshIcon from "$lib/components/RefreshIcon.svelte";
  import WeatherIcon from "$lib/components/WeatherIcon.svelte";
  import Loader from "$lib/components/Loader.svelte";
  import DayStrip from "$lib/components/DayStrip.svelte";
  import HourChart from "$lib/components/HourChart.svelte";
  import HourList from "$lib/components/HourList.svelte";
  import LocationSheet from "$lib/components/LocationSheet.svelte";
  import SettingsSheet from "$lib/components/SettingsSheet.svelte";

  const PULL_TRIGGER = 70;

  let config = $state<Config>({
    last_location: null,
    favorites: [],
    days: 7,
    theme_mode: "system",
    palette: "ocean",
    provider: "open_meteo",
    units: "celsius",
    custom_colors: {},
    refresh_minutes: 15,
    resume_minutes: 30,
    language: "system",
    widget_refresh_minutes: 30,
    widget: {
      location: false,
      now: true,
      summary: false,
      day_picker: true,
      return_to_today: true,
      refresh_button: true,
      rain: true,
      clock: "big",
      background: 100,
    },
  });
  /** Landscape on a phone or wider: two columns (see `split`). */
  let wide = $state(false);
  /** The user is dragging across the chart: skip easing so everything follows. */
  let scrubbing = $state(false);
  /** Unsaved colors while dragging a slider in settings. */
  let previewColors = $state<PaletteColors | null>(null);
  let dayChoices = $state<number[]>([3, 7, 10, 16]);
  let location = $state<Location | null>(null);
  let forecast = $state<Forecast | null>(null);
  let fetchedAt = 0;
  let loading = $state(false);
  /** Until the first forecast attempt (or the location prompt) on launch. */
  let booting = $state(true);
  let error = $state<string | null>(null);
  let sheetOpen = $state(false);
  let settingsOpen = $state(false);
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
  /** Two-column layout: only once there is a forecast to lay out. */
  const split = $derived(wide && !!forecast);
  const isFavorite = $derived(!!location && config.favorites.some((f) => samePlace(f, location!)));

  function nowIndex(): number {
    if (!forecast) return 0;
    const i = hourly.findIndex((h) => hourKey(h) === nowHourKey(forecast!.utc_offset_seconds));
    return i < 0 ? 0 : i;
  }

  // When the hour turns, a selection on the hour that just ended moves to the new
  // one; an hour picked by hand stays. Doesn't wait for the next fetch.
  let lastNowKey = "";
  $effect(() => {
    const key = nowKey;
    const previous = lastNowKey;
    lastNowKey = key;
    if (!previous || key === previous) return;
    untrack(() => {
      if (selectedHour && hourKey(selectedHour) === previous) selected = nowIndex();
    });
  });

  async function load(target: Location, { keepSelection = false } = {}) {
    loading = true;
    error = null;
    // An hour picked by hand survives a refresh; "now" follows the clock.
    const picked = selectedHour && hourKey(selectedHour) !== nowKey;
    const keep = keepSelection && picked ? selectedHour.time : null;
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

  const refresh = async () => {
    if (location && !loading) await load(location, { keepSelection: true });
  };

  async function toggleFavorite(target: Location) {
    try {
      config = await api.toggleFavorite(target);
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() =>
    applyTheme(config.theme_mode, config.palette, previewColors ?? config.custom_colors[config.palette]),
  );
  $effect(() => {
    prefs.units = config.units;
  });
  $effect(() => {
    i18n.lang = resolveLang(config.language);
    document.documentElement.lang = i18n.lang;
  });

  // The big temperature counts to its new value.
  const heroTemp = new Tween(0, { duration: 500, easing: cubicInOut });
  let heroShown = false;
  $effect(() => {
    if (!selectedHour) return;
    const value = toUnit(selectedHour.temperature);
    // Show the first value directly; count only when it changes.
    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
    heroTemp.set(value, heroShown && !reduced && !scrubbing ? undefined : { duration: 0 });
    heroShown = true;
  });

  async function changeSettings(patch: SettingsPatch) {
    const before = config;
    try {
      config = await api.updateSettings(patch);
    } catch (e) {
      error = String(e);
      return;
    }
    const refetch = config.days !== before.days || config.provider !== before.provider;
    if (refetch && location) await load(location, { keepSelection: true });
  }

  function selectDay(date: string) {
    const start = hourly.findIndex((h) => dateOf(h) === date);
    if (start < 0) return;
    // Keep the same time of day when switching days, or "now" when going to today.
    const offset = date === today ? nowIndex() - start : selected - dayStart;
    const count = hourly.filter((h) => dateOf(h) === date).length;
    selected = start + Math.min(Math.max(offset, 0), count - 1);
  }

  // Pull to refresh: follows the finger, then holds a spinner until loaded.
  const PULL_HOLD = 52;
  let pullStart: number | null = null;
  let pull = $state(0);
  let dragging = $state(false);
  let pullRefreshing = $state(false);
  const pullHeight = $derived(pullRefreshing ? PULL_HOLD : pull);

  function touchStart(e: TouchEvent) {
    // The hour list scrolls on its own: pulling there must not refresh.
    const inList = (e.target as Element | null)?.closest?.(".hours");
    const free = !inList && !sheetOpen && !settingsOpen && !pullRefreshing;
    pullStart = free ? e.touches[0].clientY : null;
  }
  function touchMove(e: TouchEvent) {
    if (pullStart === null) return;
    dragging = true;
    // Resistance grows as you pull further.
    const dy = Math.max(0, e.touches[0].clientY - pullStart);
    pull = 110 * (1 - Math.exp(-dy / 220));
  }
  async function touchEnd() {
    const triggered = pull >= PULL_TRIGGER && !!location;
    dragging = false;
    pull = 0;
    pullStart = null;
    if (triggered) {
      pullRefreshing = true;
      await refresh();
      pullRefreshing = false;
    }
  }

  onMount(() => {
    const landscape = matchMedia("(orientation: landscape) and (min-width: 600px)");
    const onOrientation = () => (wide = landscape.matches);
    onOrientation();
    landscape.addEventListener("change", onOrientation);
    // Refresh every `refresh_minutes` while the app is open and visible, and on
    // coming back if the forecast is older than `resume_minutes`.
    const olderThan = (minutes: number) => fetchedAt > 0 && Date.now() - fetchedAt >= minutes * 60_000;
    const tick = setInterval(() => {
      clock = Date.now();
      if (document.visibilityState === "visible" && olderThan(config.refresh_minutes)) refresh();
    }, 30_000);
    const onVisible = () => {
      if (document.visibilityState !== "visible") return;
      clock = Date.now();
      // Coming back to the app starts again from now.
      if (forecast) selected = nowIndex();
      if (olderThan(config.resume_minutes)) refresh();
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
      booting = false;
    })();

    return () => {
      clearInterval(tick);
      document.removeEventListener("visibilitychange", onVisible);
      landscape.removeEventListener("change", onOrientation);
    };
  });
</script>

<svelte:window ontouchstart={touchStart} ontouchmove={touchMove} ontouchend={touchEnd} />

<!-- Portrait: one column. Landscape (split): forecast summary and chart on the
     left, days and hours on the right. -->
<main class:split>
  <div class="top">
    <div
      class="pull"
      class:dragging
      style:height="{pullHeight}px"
      style:opacity={pullRefreshing ? 1 : Math.min(1, pull / PULL_TRIGGER)}
    >
      <span
        class="pull-icon"
        class:ready={pull >= PULL_TRIGGER || pullRefreshing}
        style:transform={pullRefreshing ? "" : `rotate(${pull * 3}deg)`}
      >
        <RefreshIcon spinning={pullRefreshing} size={22} />
      </span>
    </div>

    <header>
      <button class="place" onclick={() => (sheetOpen = true)}>
        <span class="name">
          <span class="name-text">{location ? location.name : t("chooseLocation")}</span>
          <Icon name="chevron-down" size={18} />
        </span>
        {#if location}<span class="sub">{label(location).split(", ").slice(1).join(", ") || forecast?.timezone}</span>{/if}
      </button>
      {#if location}
        <button class="icon-btn star" class:on={isFavorite} onclick={() => toggleFavorite(location!)} aria-label={t("toggleFavorite")}>
          <Icon name={isFavorite ? "star-filled" : "star"} />
        </button>
      {/if}
      <button class="icon-btn" onclick={refresh} disabled={!location} aria-label={t("refresh")}>
        <RefreshIcon spinning={loading && !pullRefreshing} />
      </button>
      <button class="icon-btn gear" onclick={() => (settingsOpen = true)} aria-label={t("settings")}>
        <Icon name="gear" />
      </button>
    </header>

    {#if error}
      <div class="banner error" transition:slide={{ duration: 300, easing: cubicInOut }}>
        <span>{error}</span>
        {#if location}<button onclick={refresh}>{t("retry")}</button>{/if}
      </div>
    {:else if forecast?.fallback}
      <div class="banner notice" transition:slide={{ duration: 300, easing: cubicInOut }}>
        <span>
          {t("fallbackNotice", {
            preferred: PROVIDER_NAMES[config.provider],
            source: PROVIDER_NAMES[forecast.source],
          })}
        </span>
      </div>
    {/if}

    {#if forecast && selectedHour}
      <section class="hero" in:rise={{ y: 12, duration: 450 }}>
        <div class="big-icon">
          {#key selectedHour.weather.description}
            <div class="icon-swap" in:rise={{ y: 8, duration: 400 }}>
              <WeatherIcon weather={selectedHour.weather} size={split ? 64 : 84} animated />
            </div>
          {/key}
        </div>
        <div class="hero-main">
          <div class="hero-temp">{Math.round(heroTemp.current)}°<small>{unitLetter()}</small></div>
          {#key selectedHour.weather.description}
            <div class="hero-desc" in:rise={{ y: 6, duration: 350 }}>{weatherText(selectedHour.weather.description)}</div>
          {/key}
          <div class="hero-meta">
            {selectedDate === today ? t("today") : longDate(selectedDate)} · {hourOf(selectedHour)}
            {#if hourKey(selectedHour) !== nowKey}
              <button class="now-btn" onclick={() => (selected = nowIndex())} transition:rise={{ y: 4, duration: 250 }}>{t("nowButton")}</button>
            {/if}
          </div>
          {#if selectedDay}
            <div class="hero-meta">
              <Icon name="arrow-up" size={14} />{tempUnit(selectedDay.temperature_max)}
              <Icon name="arrow-down" size={14} />{tempUnit(selectedDay.temperature_min)}
              <span class="dot">·</span>
              <span class="rain-meta"><Icon name="drop" size={14} />{mm(selectedDay.precipitation)} mm</span>
            </div>
          {/if}
        </div>
      </section>

      {#if !split}
        <DayStrip days={forecast.daily} selected={selectedDate} {today} onselect={selectDay} />
      {/if}

      <section class="card chart-card" in:rise={{ y: 16, duration: 450 }}>
        <HourChart
          hours={dayHours}
          selected={selected - dayStart}
          onselect={(i) => (selected = dayStart + i)}
          onscrub={(on) => (scrubbing = on)}
          fill={split}
        />
      </section>
    {/if}
  </div>

  <div class="side">
  {#if forecast && selectedHour}
    {#if split}
      <DayStrip days={forecast.daily} selected={selectedDate} {today} onselect={selectDay} />
    {/if}
    <section class="card hours" in:rise={{ y: 20, duration: 500 }}>
      <HourList
        hours={dayHours}
        selected={selected - dayStart}
        {nowKey}
        instant={scrubbing}
        onselect={(i) => (selected = dayStart + i)}
      >
        {#snippet footer()}
          <p class="credit">
            {#if forecast!.source === "met_norway"}
              {t("creditMet")}
            {:else}
              {t("creditOpenMeteo")}
            {/if}
          </p>
        {/snippet}
      </HourList>
    </section>
  {:else if booting || loading}
    <Loader />
  {:else if !location}
    <p class="placeholder">{t("getStarted")}</p>
  {/if}
  </div>
</main>

{#if sheetOpen}
  <LocationSheet
    current={location}
    favorites={config.favorites}
    onpick={pick}
    ontogglefavorite={toggleFavorite}
    onclose={() => (sheetOpen = false)}
  />
{/if}

{#if settingsOpen}
  <SettingsSheet
    {config}
    {dayChoices}
    onchange={changeSettings}
    onpreview={(c) => (previewColors = c)}
    onclose={() => (settingsOpen = false)}
  />
{/if}

<style>
  /* Top part stays put; the hour list fills the rest and scrolls on its own. */
  main {
    max-width: 640px;
    height: 100dvh;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    padding-top: env(safe-area-inset-top);
    padding-bottom: env(safe-area-inset-bottom);
    overflow-y: auto;
  }
  .top {
    flex: none;
  }
  /* Portrait: the right column's children sit in <main>'s single column. */
  .side {
    display: contents;
  }

  /* ---- Landscape: two columns, each filling the height ---------------- */
  main.split {
    max-width: none;
    flex-direction: row;
    padding-left: env(safe-area-inset-left);
    padding-right: env(safe-area-inset-right);
    overflow: hidden;
  }
  .split .top,
  .split .side {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .split .top {
    flex: 1.1 1 0;
    overflow-y: auto;
  }
  .split .side {
    flex: 1 1 0;
    padding-top: 6px;
  }
  /* Takes whatever height is left in the column; the chart scales to it. */
  .split .chart-card {
    flex: 1 1 0;
    min-height: 0;
    margin-bottom: 12px;
    padding: 8px 0;
  }
  .split .hours {
    flex: 1 1 0;
    min-height: 0;
    margin-top: 4px;
  }
  .split .hero {
    padding-top: 0;
    padding-bottom: 2px;
  }
  .split header {
    padding-top: 6px;
  }
  .split .big-icon {
    width: 64px;
    height: 64px;
  }
  .split .hero-temp {
    font-size: 40px;
  }
  .hours {
    flex: 1 0 240px;
    min-height: 240px;
    padding: 0;
    margin-bottom: 12px;
  }
  .credit {
    text-align: center;
    font-size: 11px;
    color: var(--muted);
    padding: 12px;
    margin: 0;
  }
  .pull {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    color: var(--muted);
    /* Eases back when released; follows the finger 1:1 while dragging. */
    transition:
      height 0.4s cubic-bezier(0.65, 0, 0.35, 1),
      opacity 0.3s ease-in-out;
  }
  .pull.dragging {
    transition: none;
  }
  .pull-icon {
    display: flex;
    padding: 7px;
    border-radius: 50%;
    background: var(--card);
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.12);
    transition:
      color 0.25s ease-in-out,
      transform 0.25s ease-out;
  }
  .pull.dragging .pull-icon {
    transition: color 0.25s ease-in-out;
  }
  .pull-icon.ready {
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
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 22px;
    font-weight: 700;
    max-width: 100%;
  }
  .name :global(svg) {
    color: var(--muted);
  }
  .name-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: 13px;
    color: var(--muted);
  }
  .icon-btn {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border: none;
    background: none;
    color: var(--fg);
    border-radius: 50%;
    transition:
      background-color 0.25s ease-in-out,
      transform 0.2s ease-in-out,
      color 0.25s ease-in-out;
  }
  .icon-btn:active {
    background: var(--highlight);
  }
  .icon-btn.star {
    color: var(--star);
  }
  .icon-btn.star.on :global(svg) {
    animation: pop 0.45s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .icon-btn.gear :global(svg) {
    transition: transform 0.5s cubic-bezier(0.65, 0, 0.35, 1);
  }
  .icon-btn.gear:active :global(svg) {
    transform: rotate(90deg);
  }
  .icon-btn:disabled {
    opacity: 0.3;
  }
  @keyframes pop {
    50% {
      transform: scale(1.3);
    }
  }
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 8px 16px;
    padding: 10px 14px;
    border-radius: 12px;
    font-size: 14px;
  }
  .banner.error {
    background: color-mix(in srgb, var(--error) 15%, var(--card));
    color: var(--error);
  }
  .banner.notice {
    background: color-mix(in srgb, var(--temp) 15%, var(--card));
    color: var(--fg);
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
    padding: 4px 16px 10px;
  }
  .big-icon {
    display: grid;
    width: 84px;
    height: 84px;
  }
  .icon-swap {
    grid-area: 1 / 1;
  }
  .hero-temp {
    font-size: 46px;
    font-weight: 300;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  .hero-temp small {
    font-size: 20px;
    font-weight: 400;
    color: var(--muted);
    margin-left: 2px;
    vertical-align: top;
    position: relative;
    top: 4px;
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
    gap: 4px;
    margin-top: 2px;
    min-height: 22px;
  }
  .hero-meta .dot {
    margin: 0 4px;
  }
  .rain-meta {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    color: var(--rain);
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
    margin-left: 4px;
  }
  .card {
    transition: background-color 0.3s ease-in-out;
    background: var(--card);
    border-radius: 18px;
    margin: 8px 16px;
    padding: 12px 0;
    overflow: hidden;
  }
  .placeholder {
    text-align: center;
    color: var(--muted);
    margin-top: 30vh;
    padding: 0 24px;
  }
</style>
