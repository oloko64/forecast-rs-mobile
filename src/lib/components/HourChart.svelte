<script lang="ts">
  import type { HourForecast } from "$lib/types";
  import { hourOf, mm, temp } from "$lib/format";

  let {
    hours,
    selected,
    onselect,
  }: {
    hours: HourForecast[];
    /** Index into `hours`. */
    selected: number;
    onselect: (index: number) => void;
  } = $props();

  let width = $state(320);
  const H = 190;
  const PAD_X = 18;
  const TEMP_TOP = 30;
  const TEMP_BOTTOM = 118;
  const RAIN_TOP = 132;
  const RAIN_BOTTOM = 166;

  const n = $derived(hours.length);
  const step = $derived(n > 1 ? (width - 2 * PAD_X) / (n - 1) : 0);
  const x = (i: number) => PAD_X + i * step;

  const temps = $derived(hours.map((h) => h.temperature));
  const tMin = $derived(Math.min(...temps));
  const tMax = $derived(Math.max(...temps));
  const y = (t: number) => {
    const span = Math.max(tMax - tMin, 2);
    const mid = (tMax + tMin) / 2;
    return TEMP_BOTTOM - ((t - (mid - span / 2)) / span) * (TEMP_BOTTOM - TEMP_TOP);
  };

  const line = $derived(hours.map((h, i) => `${i ? "L" : "M"}${x(i)},${y(h.temperature)}`).join(""));
  const area = $derived(n ? `${line}L${x(n - 1)},${TEMP_BOTTOM + 8}L${x(0)},${TEMP_BOTTOM + 8}Z` : "");

  const rainPeak = $derived(Math.max(0, ...hours.map((h) => h.precipitation)));
  const rainMax = $derived(Math.max(1, rainPeak));
  const barW = $derived(Math.max(2, step * 0.6));
  const rainH = (p: number) => (p <= 0 ? 0 : Math.max(2, (p / rainMax) * (RAIN_BOTTOM - RAIN_TOP)));

  const sel = $derived(hours[selected]);
  const tickEvery = $derived(width < 360 ? 6 : 3);

  function pick(e: PointerEvent) {
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
    const i = Math.round((e.clientX - rect.left - PAD_X) / (step || 1));
    const clamped = Math.min(n - 1, Math.max(0, i));
    if (clamped !== selected) onselect(clamped);
  }

  function down(e: PointerEvent) {
    (e.currentTarget as SVGElement).setPointerCapture(e.pointerId);
    pick(e);
  }
</script>

<div class="chart" bind:clientWidth={width}>
  {#if n > 0}
    <svg
      {width}
      height={H}
      role="slider"
      aria-label="Hour"
      aria-valuemin={0}
      aria-valuemax={n - 1}
      aria-valuenow={selected}
      tabindex="0"
      onpointerdown={down}
      onpointermove={(e) => e.buttons && pick(e)}
    >
      <defs>
        <linearGradient id="tempFill" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0" stop-color="var(--temp)" stop-opacity="0.35" />
          <stop offset="1" stop-color="var(--temp)" stop-opacity="0" />
        </linearGradient>
      </defs>

      <path d={area} fill="url(#tempFill)" />
      <path d={line} fill="none" stroke="var(--temp)" stroke-width="2.5" stroke-linejoin="round" />

      {#each hours as h, i}
        {#if h.precipitation > 0}
          <rect
            x={x(i) - barW / 2}
            y={RAIN_BOTTOM - rainH(h.precipitation)}
            width={barW}
            height={rainH(h.precipitation)}
            rx="1.5"
            fill="var(--rain)"
            opacity={i === selected ? 1 : 0.6}
          />
        {/if}
      {/each}
      <line x1={PAD_X} x2={width - PAD_X} y1={RAIN_BOTTOM} y2={RAIN_BOTTOM} stroke="var(--line)" />

      {#each hours as h, i}
        {#if i % tickEvery === 0}
          <text x={x(i)} y={H - 6} class="tick">{hourOf(h).slice(0, 2)}h</text>
        {/if}
      {/each}

      {#if sel}
        <line x1={x(selected)} x2={x(selected)} y1={TEMP_TOP - 12} y2={RAIN_BOTTOM} stroke="var(--fg)" stroke-opacity="0.25" stroke-dasharray="3 3" />
        <circle cx={x(selected)} cy={y(sel.temperature)} r="5" fill="var(--bg)" stroke="var(--temp)" stroke-width="2.5" />
        <text
          x={Math.min(width - 24, Math.max(24, x(selected)))}
          y={Math.max(14, y(sel.temperature) - 12)}
          class="value">{temp(sel.temperature)}</text
        >
      {/if}
    </svg>
    <div class="legend">
      <span><i class="dot temp"></i>Temperature</span>
      <span><i class="dot rain"></i>{rainPeak > 0 ? `Rain · max ${mm(rainPeak)} mm/h` : "Rain · none"}</span>
    </div>
  {/if}
</div>

<style>
  .chart {
    width: 100%;
    touch-action: pan-y;
  }
  svg {
    display: block;
    outline: none;
    user-select: none;
  }
  .tick {
    font-size: 11px;
    fill: var(--muted);
    text-anchor: middle;
  }
  .value {
    font-size: 13px;
    font-weight: 700;
    fill: var(--fg);
    text-anchor: middle;
  }
  .legend {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
    padding: 0 18px;
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 6px;
  }
  .dot.temp {
    background: var(--temp);
  }
  .dot.rain {
    background: var(--rain);
  }
</style>
