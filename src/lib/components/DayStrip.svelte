<script lang="ts">
  import type { DayForecast } from "$lib/types";
  import { dayOfMonth, mm, tempUnit, weekday } from "$lib/format";
  import WeatherIcon from "./WeatherIcon.svelte";
  import { t } from "$lib/i18n.svelte";

  let {
    days,
    selected,
    today,
    onselect,
  }: {
    days: DayForecast[];
    selected: string;
    today: string;
    onselect: (date: string) => void;
  } = $props();

  let strip: HTMLDivElement;
  let stripWidth = $state(0);
  let placed = false;

  $effect(() => {
    // Center the selected chip; again when the strip's width changes (rotation).
    // Scrolls the strip only, never the page.
    const width = stripWidth;
    const chip = strip?.querySelector<HTMLElement>(`[data-date="${selected}"]`);
    if (!chip || width === 0) return;
    const max = strip.scrollWidth - width;
    const left = Math.min(max, Math.max(0, chip.offsetLeft - (width - chip.offsetWidth) / 2));
    strip.scrollTo({ left, behavior: placed ? "smooth" : "instant" });
    placed = true;
  });
</script>

<div class="strip" bind:this={strip} bind:clientWidth={stripWidth}>
  {#each days as day (day.date)}
    {@const active = day.date === selected}
    <button class="day" class:active data-date={day.date} onclick={() => onselect(day.date)}>
      <span class="name">{day.date === today ? t("today") : weekday(day.date)}</span>
      <span class="num">{dayOfMonth(day.date)}</span>
      <span class="icon"><WeatherIcon weather={day.weather} size={34} animated={active} /></span>
      <span class="max">{tempUnit(day.temperature_max)}</span>
      <span class="min">{tempUnit(day.temperature_min)}</span>
      <span class="rain" class:dry={day.precipitation < 0.1}>{mm(day.precipitation)} mm</span>
    </button>
  {/each}
</div>

<style>
  .strip {
    position: relative;
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 6px 16px 10px;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .day {
    flex: 0 0 70px;
    scroll-snap-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 10px 4px;
    border-radius: 18px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    transition:
      background-color 0.35s ease-in-out,
      border-color 0.35s ease-in-out,
      color 0.35s ease-in-out,
      transform 0.35s cubic-bezier(0.65, 0, 0.35, 1),
      box-shadow 0.35s ease-in-out;
  }
  .day.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
    transform: translateY(-3px);
    box-shadow: 0 6px 16px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .name {
    font-size: 12px;
    font-weight: 600;
  }
  .num {
    font-size: 11px;
    opacity: 0.7;
  }
  .icon {
    margin: 2px 0;
  }
  .max {
    font-weight: 700;
    font-size: 14px;
  }
  .min {
    font-size: 12px;
    opacity: 0.7;
  }
  .rain {
    font-size: 11px;
    margin-top: 3px;
    white-space: nowrap;
  }
  .rain.dry {
    opacity: 0.45;
  }
</style>
