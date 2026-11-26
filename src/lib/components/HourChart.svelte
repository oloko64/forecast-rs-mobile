<script lang="ts">
  import type { HourForecast } from "$lib/types";
  import { hourOf, mm, temp } from "$lib/format";
  import { Tween } from "svelte/motion";
  import { untrack } from "svelte";
  import { t } from "$lib/i18n.svelte";
  import { cubicInOut } from "svelte/easing";

  let {
    hours,
    selected,
    onselect,
    onscrub,
    fill = false,
  }: {
    hours: HourForecast[];
    /** Index into `hours`. */
    selected: number;
    onselect: (index: number) => void;
    /** True while the user drags across the chart. */
    onscrub?: (scrubbing: boolean) => void;
    /** Grow to the container's height (landscape) instead of a fixed height. */
    fill?: boolean;
  } = $props();

  let scrubbing = false;

  let width = $state(320);
  let boxHeight = $state(0);
  let legendHeight = $state(22);
  const H = $derived(fill && boxHeight > 0 ? Math.max(100, boxHeight - legendHeight) : 190);
  const PAD_X = 18;
  // Layout of the default 190 px chart, scaled to the actual height.
  const TEMP_TOP = 30;
  const TEMP_BOTTOM = $derived(H * (118 / 190));
  const RAIN_TOP = $derived(H * (132 / 190));
  const RAIN_BOTTOM = $derived(H - 24);

  const n = $derived(hours.length);
  const step = $derived(n > 1 ? (width - 2 * PAD_X) / (n - 1) : 0);
  const x = (i: number) => PAD_X + i * step;

  // Values tween between days/hours so the chart morphs instead of jumping.
  const MOTION = { duration: 450, easing: cubicInOut };
  const target = $derived(hours.map((h) => h.temperature));
  const rainTarget = $derived(hours.map((h) => h.precipitation));
  const tempsT = new Tween<number[]>([], MOTION);
  const rainT = new Tween<number[]>([], MOTION);
  const selT = new Tween(0, MOTION);
  $effect(() => {
    // Tweening needs equal lengths (MET Norway days can be partial): jump otherwise.
    // untrack: reading the animated value must not re-run this effect, or every
    // frame would restart the tween and it would never reach the new day.
    const same = untrack(() => tempsT.current.length) === target.length;
    tempsT.set(target, same ? MOTION : { duration: 0 });
    rainT.set(rainTarget, same ? MOTION : { duration: 0 });
  });
  $effect(() => {
    // Follow the finger exactly while dragging; ease for taps elsewhere.
    selT.set(selected, scrubbing ? { duration: 0 } : MOTION);
  });
  const temps = $derived(tempsT.current.length === n ? tempsT.current : target);
  const rains = $derived(rainT.current.length === n ? rainT.current : rainTarget);
  // Scale over both the new values and the ones still morphing towards them,
  // so nothing leaves the chart mid-animation.
  const tMin = $derived(Math.min(...target, ...temps));
  const tMax = $derived(Math.max(...target, ...temps));
  const y = (t: number) => {
    const span = Math.max(tMax - tMin, 2);
    const mid = (tMax + tMin) / 2;
    return TEMP_BOTTOM - ((t - (mid - span / 2)) / span) * (TEMP_BOTTOM - TEMP_TOP);
  };

  // Smooth curve through the points (Catmull-Rom → cubic Bézier).
  const line = $derived.by(() => {
    const p = temps.map((t, i) => [x(i), y(t)] as const);
    if (p.length < 2) return "";
    let d = `M${p[0][0]},${p[0][1]}`;
    for (let i = 0; i < p.length - 1; i++) {
      const [x0, y0] = p[Math.max(0, i - 1)];
      const [x1, y1] = p[i];
      const [x2, y2] = p[i + 1];
      const [x3, y3] = p[Math.min(p.length - 1, i + 2)];
      d += `C${x1 + (x2 - x0) / 6},${y1 + (y2 - y0) / 6} ${x2 - (x3 - x1) / 6},${y2 - (y3 - y1) / 6} ${x2},${y2}`;
    }
    return d;
  });
  const area = $derived(n ? `${line}L${x(n - 1)},${TEMP_BOTTOM + 8}L${x(0)},${TEMP_BOTTOM + 8}Z` : "");

  const rainPeak = $derived(Math.max(0, ...hours.map((h) => h.precipitation)));
  const rainMax = $derived(Math.max(1, rainPeak, ...rains));
  const barW = $derived(Math.max(2, step * 0.6));
  const rainH = (p: number) => (p <= 0 ? 0 : Math.max(2, (p / rainMax) * (RAIN_BOTTOM - RAIN_TOP)));

  const sel = $derived(hours[selected]);
  /** Interpolated position of the selection marker. */
  const selX = $derived(x(selT.current));
  const selY = $derived.by(() => {
    const i = Math.min(Math.max(selT.current, 0), Math.max(0, temps.length - 1));
    const lo = Math.floor(i);
    const hi = Math.min(lo + 1, temps.length - 1);
    return y((temps[lo] ?? 0) + ((temps[hi] ?? 0) - (temps[lo] ?? 0)) * (i - lo));
  });
  const tickEvery = $derived(width < 360 ? 6 : 3);

  function pick(e: PointerEvent) {
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
    const i = Math.round((e.clientX - rect.left - PAD_X) / (step || 1));
    const clamped = Math.min(n - 1, Math.max(0, i));
    if (clamped !== selected) onselect(clamped);
  }

  function down(e: PointerEvent) {
    (e.currentTarget as SVGElement).setPointerCapture(e.pointerId);
    scrubbing = true;
    onscrub?.(true);
    pick(e);
  }

  function up() {
    if (!scrubbing) return;
    scrubbing = false;
    onscrub?.(false);
  }
</script>

<div class="chart" class:fill bind:clientWidth={width} bind:clientHeight={boxHeight}>
  {#if n > 0}
    <svg
      {width}
      height={H}
      role="slider"
      aria-label={t("hour")}
      aria-valuemin={0}
      aria-valuemax={n - 1}
      aria-valuenow={selected}
      tabindex="0"
      onpointerdown={down}
      onpointermove={(e) => scrubbing && pick(e)}
      onpointerup={up}
      onpointercancel={up}
      onlostpointercapture={up}
    >
      <defs>
        <linearGradient id="tempFill" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0" stop-color="var(--temp)" stop-opacity="0.35" />
          <stop offset="1" stop-color="var(--temp)" stop-opacity="0" />
        </linearGradient>
      </defs>

      <path d={area} fill="url(#tempFill)" />
      <path d={line} fill="none" stroke="var(--temp)" stroke-width="2.5" stroke-linejoin="round" />

      {#each rains as p, i}
        {#if p > 0.001}
          <rect
            class="bar"
            x={x(i) - barW / 2}
            y={RAIN_BOTTOM - rainH(p)}
            width={barW}
            height={rainH(p)}
            rx="1.5"
            fill="var(--rain)"
            opacity={i === selected ? 1 : 0.55}
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
        <line x1={selX} x2={selX} y1={TEMP_TOP - 12} y2={RAIN_BOTTOM} stroke="var(--fg)" stroke-opacity="0.25" stroke-dasharray="3 3" />
        <circle cx={selX} cy={selY} r="9" fill="var(--temp)" opacity="0.18" />
        <circle cx={selX} cy={selY} r="5" fill="var(--bg)" stroke="var(--temp)" stroke-width="2.5" />
        <text x={Math.min(width - 30, Math.max(30, selX))} y={Math.max(14, selY - 14)} class="value"
          >{temp(sel.temperature)}</text
        >
        <!-- That hour's rain, under the point (same as the widget). -->
        <text x={Math.min(width - 30, Math.max(30, selX))} y={Math.min(RAIN_BOTTOM - 6, selY + 22)} class="rain-value"
          >{mm(sel.precipitation)} mm</text
        >
      {/if}
    </svg>
    <div class="legend" bind:clientHeight={legendHeight}>
      <span><i class="dot temp"></i>{t("temperature")}</span>
      <span><i class="dot rain"></i>{rainPeak > 0 ? t("rainMax", { value: mm(rainPeak) }) : t("rainNone")}</span>
    </div>
  {/if}
</div>

<style>
  .chart {
    width: 100%;
    touch-action: pan-y;
  }
  .chart.fill {
    height: 100%;
    overflow: hidden;
  }
  svg {
    display: block;
    outline: none;
    user-select: none;
  }
  path,
  .bar {
    transition:
      fill 0.3s ease-in-out,
      stroke 0.3s ease-in-out,
      opacity 0.3s ease-in-out;
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
  .rain-value {
    font-size: 12px;
    font-weight: 700;
    fill: var(--rain);
    text-anchor: middle;
    paint-order: stroke;
    stroke: var(--card);
    stroke-width: 3px;
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
