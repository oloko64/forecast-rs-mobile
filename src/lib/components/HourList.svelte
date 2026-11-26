<script lang="ts">
  import type { HourForecast } from "$lib/types";
  import { hourKey, hourOf, mm, temp } from "$lib/format";

  let {
    hours,
    selected,
    nowKey,
    onselect,
  }: {
    hours: HourForecast[];
    selected: number;
    nowKey: string;
    onselect: (index: number) => void;
  } = $props();

  let list: HTMLDivElement;

  $effect(() => {
    list?.querySelector<HTMLElement>(`[data-i="${selected}"]`)?.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
    });
  });
</script>

<div class="list" bind:this={list}>
  {#each hours as h, i (h.time)}
    <button class="row" class:active={i === selected} data-i={i} onclick={() => onselect(i)}>
      <span class="time">{hourOf(h)}</span>
      <span class="icon">{h.weather.icon}</span>
      <span class="desc">
        {h.weather.description}
        {#if hourKey(h) === nowKey}<em class="now">now</em>{/if}
      </span>
      <span class="rain" class:dry={h.precipitation === 0 && !h.precipitation_probability}>
        {#if h.precipitation_probability != null}{Math.round(h.precipitation_probability)}% · {/if}{mm(h.precipitation)}mm
      </span>
      <span class="temp">{temp(h.temperature)}</span>
    </button>
  {/each}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: grid;
    grid-template-columns: 48px 32px 1fr auto 44px;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border: none;
    border-bottom: 1px solid var(--line);
    background: transparent;
    color: var(--fg);
    font: inherit;
    text-align: left;
  }
  .row.active {
    background: var(--highlight);
  }
  .time {
    font-variant-numeric: tabular-nums;
    color: var(--muted);
  }
  .icon {
    font-size: 20px;
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
    margin-left: 6px;
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
</style>
