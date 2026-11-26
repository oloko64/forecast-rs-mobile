<script lang="ts">
  import type { DayForecast } from "$lib/types";
  import { dayOfMonth, mm, temp, weekday } from "$lib/format";

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

  $effect(() => {
    // Keep the selected chip visible when it changes (e.g. hour navigation crosses days).
    strip?.querySelector<HTMLElement>(`[data-date="${selected}"]`)?.scrollIntoView({
      behavior: "smooth",
      inline: "center",
      block: "nearest",
    });
  });
</script>

<div class="strip" bind:this={strip}>
  {#each days as day (day.date)}
    <button
      class="day"
      class:active={day.date === selected}
      data-date={day.date}
      onclick={() => onselect(day.date)}
    >
      <span class="name">{day.date === today ? "Today" : weekday(day.date)}</span>
      <span class="num">{dayOfMonth(day.date)}</span>
      <span class="icon">{day.weather.icon}</span>
      <span class="max">{temp(day.temperature_max)}</span>
      <span class="min">{temp(day.temperature_min)}</span>
      <span class="rain" class:dry={day.precipitation < 0.1}>💧{mm(day.precipitation)}</span>
    </button>
  {/each}
</div>

<style>
  .strip {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 4px 16px 8px;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .day {
    flex: 0 0 64px;
    scroll-snap-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 4px;
    border-radius: 16px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
  }
  .day.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
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
    font-size: 22px;
    margin: 2px 0;
  }
  .max {
    font-weight: 700;
  }
  .min {
    font-size: 13px;
    opacity: 0.7;
  }
  .rain {
    font-size: 11px;
    margin-top: 2px;
  }
  .rain.dry {
    opacity: 0.4;
  }
</style>
