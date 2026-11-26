<script lang="ts">
  import { hexToHsl, hslToHex } from "$lib/theme";

  /** Hue / saturation / lightness sliders plus quick swatches. */
  let { value, onchange }: { value: string; onchange: (hex: string) => void } = $props();

  const SWATCHES = [
    "#e5484d", "#f76b15", "#f5a623", "#30a46c", "#12a594", "#0090ff",
    "#3867f0", "#6e56cf", "#c2417a", "#8d6e63", "#5f6b7a", "#1f2937",
  ];

  let h = $state(0);
  let s = $state(0);
  let l = $state(0);
  // Sync from outside (e.g. switching the role being edited), not while dragging.
  let last = "";
  $effect(() => {
    if (value !== last) [h, s, l] = hexToHsl(value);
  });

  function emit() {
    last = hslToHex(h, s, l);
    onchange(last);
  }
</script>

<div class="picker">
  <div class="swatches">
    {#each SWATCHES as c}
      <button
        class="swatch"
        class:active={c === value}
        style:background={c}
        aria-label="Color {c}"
        onclick={() => {
          [h, s, l] = hexToHsl(c);
          last = c;
          onchange(c);
        }}
      ></button>
    {/each}
  </div>

  <label>
    <span>Hue</span>
    <input
      type="range"
      min="0"
      max="359"
      bind:value={h}
      oninput={emit}
      style:--track="linear-gradient(to right, {[0, 60, 120, 180, 240, 300, 359].map((x) => hslToHex(x, s, l)).join(',')})"
    />
  </label>
  <label>
    <span>Saturation</span>
    <input
      type="range"
      min="0"
      max="100"
      bind:value={s}
      oninput={emit}
      style:--track="linear-gradient(to right, {hslToHex(h, 0, l)}, {hslToHex(h, 100, l)})"
    />
  </label>
  <label>
    <span>Lightness</span>
    <input
      type="range"
      min="15"
      max="85"
      bind:value={l}
      oninput={emit}
      style:--track="linear-gradient(to right, {hslToHex(h, s, 15)}, {hslToHex(h, s, 50)}, {hslToHex(h, s, 85)})"
    />
  </label>
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border-radius: 14px;
    background: var(--card);
    border: 1px solid var(--line);
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 8px;
  }
  .swatch {
    aspect-ratio: 1;
    border-radius: 50%;
    border: 3px solid transparent;
    outline: 2px solid transparent;
    transition:
      transform 0.2s ease-in-out,
      outline-color 0.2s ease-in-out;
  }
  .swatch.active {
    outline-color: var(--fg);
    transform: scale(1.08);
  }
  label {
    display: grid;
    grid-template-columns: 82px 1fr;
    align-items: center;
    font-size: 13px;
    color: var(--muted);
  }
  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 14px;
    border-radius: 7px;
    background: var(--track);
    margin: 8px 0;
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: #fff;
    border: 2px solid rgb(0 0 0 / 0.25);
    box-shadow: 0 1px 4px rgb(0 0 0 / 0.3);
  }
</style>
