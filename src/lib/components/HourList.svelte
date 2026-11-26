<script lang="ts">
  import type { HourForecast } from "$lib/types";
  import { hourKey, hourOf, mm, temp } from "$lib/format";
  import type { Snippet } from "svelte";
  import WeatherIcon from "./WeatherIcon.svelte";
  import { t, weatherText } from "$lib/i18n.svelte";
  import { rise } from "$lib/motion";

  let {
    hours,
    selected,
    nowKey,
    onselect,
    footer,
    instant = false,
  }: {
    hours: HourForecast[];
    selected: number;
    nowKey: string;
    onselect: (index: number) => void;
    footer?: Snippet;
    /** Jump instead of smooth-scrolling (while scrubbing the chart). */
    instant?: boolean;
  } = $props();

  let list: HTMLDivElement;
  let first = true;

  $effect(() => {
    // Bring the selected hour to the top of this list. Only the list scrolls.
    void hours;
    const row = list?.querySelector<HTMLElement>(`[data-i="${selected}"]`);
    if (!row) return;
    list.scrollTo({ top: row.offsetTop, behavior: first || instant ? "instant" : "smooth" });
    first = false;
  });
</script>

<div class="list" bind:this={list}>
  {#key hours[0]?.time}
  {#each hours as h, i (h.time)}
    <button
      class="row"
      class:active={i === selected}
      data-i={i}
      onclick={() => onselect(i)}
      in:rise={{ y: 14, duration: 380, delay: Math.max(0, i - selected) < 8 ? Math.max(0, i - selected) * 25 : 0 }}
    >
      <span class="time">
        {hourOf(h)}
        {#if hourKey(h) === nowKey}<em class="now">{t("now")}</em>{/if}
      </span>
      <span class="icon"><WeatherIcon weather={h.weather} size={i === selected ? 40 : 28} animated={i === selected} /></span>
      <span class="desc">{weatherText(h.weather.description)}</span>
      <span class="rain" class:dry={h.precipitation === 0 && !h.precipitation_probability}>
        {h.precipitation_probability != null ? `${Math.round(h.precipitation_probability)}% · ` : ""}{mm(h.precipitation)}mm
      </span>
      <span class="temp">{temp(h.temperature)}</span>
    </button>
  {/each}
  {/key}
  {@render footer?.()}
  <!-- Room to scroll the last hours to the top too. -->
  <div class="tail"></div>
</div>

<style>
  .list {
    position: relative;
    height: 100%;
    overflow-y: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
  }
  .row {
    flex: none;
    display: grid;
    grid-template-columns: 48px 32px 1fr auto 44px;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border: none;
    border-bottom: 1px solid var(--line);
    border-left: 4px solid transparent;
    background: transparent;
    color: var(--fg);
    font: inherit;
    text-align: left;
    transition:
      background-color 0.35s ease-in-out,
      border-color 0.35s ease-in-out,
      padding 0.35s cubic-bezier(0.65, 0, 0.35, 1);
  }
  .row:active {
    transform: none;
  }
  .row.active {
    background: var(--highlight);
    border-left-color: var(--accent);
    padding-top: 16px;
    padding-bottom: 16px;
  }
  .row.active .time {
    color: var(--accent);
    font-weight: 700;
    font-size: 16px;
  }
  .icon {
    display: flex;
    justify-content: center;
  }
  /* Selected hour: two lines, rain under the description, everything bigger. */
  .row.active {
    grid-template-columns: 56px 40px 1fr 56px;
    row-gap: 2px;
  }
  .row.active .time,
  .row.active .icon,
  .row.active .temp {
    grid-row: 1 / span 2;
  }
  .row.active .desc {
    grid-column: 3;
    font-weight: 700;
    font-size: 16px;
  }
  .row.active .rain {
    grid-column: 3;
    grid-row: 2;
    font-size: 13px;
  }
  .row.active .temp {
    font-size: 24px;
  }
  .time,
  .desc,
  .temp,
  .rain {
    transition:
      font-size 0.3s ease-in-out,
      color 0.3s ease-in-out;
  }
  .time {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
  }

  .desc {
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .now {
    font-style: normal;
    font-size: 11px;
    font-weight: 700;
    color: var(--on-accent);
    background: var(--accent);
    border-radius: 6px;
    padding: 1px 6px;
  }
  .rain {
    font-size: 12px;
    color: var(--rain);
    font-variant-numeric: tabular-nums;
  }
  .rain.dry {
    opacity: 0.35;
    color: var(--muted);
  }
  .temp {
    text-align: right;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .tail {
    flex: none;
    height: 70%;
  }
</style>
