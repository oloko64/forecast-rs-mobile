<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import ColorPicker from "./ColorPicker.svelte";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { COLOR_ROLES, PALETTES, paletteColors, type ColorRole } from "$lib/theme";
  import {
    PROVIDER_NAMES,
    type Config,
    type PaletteColors,
    type Provider,
    type SettingsPatch,
    type ThemeMode,
    type Units,
  } from "$lib/types";

  let {
    config,
    dayChoices,
    onchange,
    onpreview,
    onclose,
  }: {
    config: Config;
    dayChoices: number[];
    onchange: (patch: SettingsPatch) => void;
    /** Live color preview while dragging; saved after a short pause. */
    onpreview: (colors: PaletteColors | null) => void;
    onclose: () => void;
  } = $props();

  const MODES: { id: ThemeMode; name: string }[] = [
    { id: "system", name: "System" },
    { id: "light", name: "Light" },
    { id: "dark", name: "Dark" },
  ];
  const UNITS: { id: Units; name: string }[] = [
    { id: "celsius", name: "°C  Celsius" },
    { id: "fahrenheit", name: "°F  Fahrenheit" },
  ];
  const PROVIDERS = Object.entries(PROVIDER_NAMES) as [Provider, string][];

  let editing = $state<ColorRole | null>(null);
  let draft = $state<PaletteColors | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const custom = $derived(draft ?? config.custom_colors[config.palette] ?? {});
  const colors = $derived(paletteColors(config.palette, custom));
  const isCustom = $derived(Object.values(custom).some(Boolean));

  function setColor(role: ColorRole, hex: string) {
    draft = { ...custom, [role]: hex };
    onpreview(draft);
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, 350);
  }

  function flush() {
    clearTimeout(saveTimer);
    if (!draft) return;
    onchange({ custom_colors: [config.palette, draft] });
    draft = null;
    onpreview(null);
  }

  function reset() {
    clearTimeout(saveTimer);
    draft = null;
    onpreview(null);
    editing = null;
    onchange({ custom_colors: [config.palette, {}] });
  }

  function pickPalette(id: typeof config.palette) {
    flush();
    editing = null;
    onchange({ palette: id });
  }

  function close() {
    flush();
    onclose();
  }
</script>

<Sheet label="Settings" onclose={close}>
  <div class="scroll">
    <h2>Settings</h2>

    <h3>Appearance</h3>
    {@render segmented(MODES, config.theme_mode, (id) => onchange({ theme_mode: id as ThemeMode }))}

    <h3>Color theme</h3>
    <div class="palettes">
      {#each PALETTES as p (p.id)}
        {@const c = paletteColors(p.id, p.id === config.palette ? custom : config.custom_colors[p.id])}
        <button class="palette" class:active={config.palette === p.id} onclick={() => pickPalette(p.id)}>
          <span class="swatches">
            <i style:background={c.accent}></i><i style:background={c.temp}></i><i style:background={c.rain}></i>
          </span>
          <span>{p.name}{config.custom_colors[p.id] ? " *" : ""}</span>
        </button>
      {/each}
    </div>

    <div class="custom">
      <span class="custom-label">Customize</span>
      {#each COLOR_ROLES as role (role.id)}
        <button
          class="role"
          class:active={editing === role.id}
          onclick={() => (editing = editing === role.id ? null : role.id)}
        >
          <i style:background={colors[role.id]}></i>{role.name}
        </button>
      {/each}
    </div>
    {#if editing}
      <div class="picker-wrap" transition:slide={{ duration: 300, easing: cubicInOut }}>
        <ColorPicker value={colors[editing]} onchange={(hex) => setColor(editing!, hex)} />
      </div>
    {/if}
    {#if isCustom}
      <button class="reset" onclick={reset} transition:slide={{ duration: 250, easing: cubicInOut }}>
        Reset {PALETTES.find((p) => p.id === config.palette)?.name} to preset colors
      </button>
    {/if}

    <h3>Temperature</h3>
    {@render segmented(UNITS, config.units, (id) => onchange({ units: id as Units }))}

    <h3>Days to forecast</h3>
    {@render segmented(
      dayChoices.map((d) => ({ id: d, name: String(d) })),
      config.days,
      (id) => onchange({ days: id as number }),
    )}

    <h3>Weather data</h3>
    {@render segmented(
      PROVIDERS.map(([id, name]) => ({ id, name })),
      config.provider,
      (id) => onchange({ provider: id as Provider }),
    )}
    <p class="hint">
      If {PROVIDER_NAMES[config.provider]} fails 3 times in a row, the app switches to
      {PROVIDER_NAMES[config.provider === "open_meteo" ? "met_norway" : "open_meteo"]} for a while.
      MET Norway forecasts up to ~10 days.
    </p>
  </div>
</Sheet>

{#snippet segmented<T>(items: { id: T; name: string }[], value: T, pick: (id: T) => void)}
  {@const index = Math.max(0, items.findIndex((i) => i.id === value))}
  <div class="segmented" style:--count={items.length} style:--index={index}>
    <span class="thumb"></span>
    {#each items as item}
      <button class:active={item.id === value} onclick={() => pick(item.id)}>{item.name}</button>
    {/each}
  </div>
{/snippet}

<style>
  .scroll {
    overflow-y: auto;
    padding: 0 16px 8px;
  }
  h2 {
    font-size: 20px;
    margin: 0 0 4px;
  }
  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin: 18px 0 8px;
  }
  /* Segmented control with a sliding thumb. */
  .segmented {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--count), 1fr);
    padding: 4px;
    border-radius: 14px;
    background: var(--card);
    border: 1px solid var(--line);
  }
  .thumb {
    position: absolute;
    top: 4px;
    bottom: 4px;
    left: 4px;
    width: calc((100% - 8px) / var(--count));
    transform: translateX(calc(100% * var(--index)));
    border-radius: 10px;
    background: var(--accent);
    transition:
      transform 0.35s cubic-bezier(0.65, 0, 0.35, 1),
      background-color 0.3s ease-in-out;
  }
  .segmented button {
    position: relative;
    padding: 10px 0;
    border: none;
    border-radius: 10px;
    background: none;
    color: var(--fg);
    font: inherit;
    font-weight: 600;
    white-space: pre;
    transition: color 0.3s ease-in-out;
  }
  .segmented button.active {
    color: var(--on-accent);
  }
  .palettes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .palette {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px 4px;
    border-radius: 14px;
    border: 2px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    font-weight: 600;
    transition: border-color 0.3s ease-in-out;
  }
  .palette.active {
    border-color: var(--accent);
  }
  .swatches {
    display: flex;
  }
  .swatches i {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid var(--card);
    margin-left: -6px;
    transition: background-color 0.3s ease-in-out;
  }
  .swatches i:first-child {
    margin-left: 0;
  }
  .custom {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
  }
  .custom-label {
    font-size: 13px;
    color: var(--muted);
    margin-right: 2px;
  }
  .role {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 4px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    transition: border-color 0.25s ease-in-out;
  }
  .role.active {
    border-color: var(--fg);
  }
  .role i {
    width: 16px;
    height: 16px;
    border-radius: 50%;
  }
  .picker-wrap {
    padding-top: 8px;
  }
  .reset {
    margin-top: 8px;
    width: 100%;
    padding: 10px;
    border-radius: 12px;
    border: 1px dashed var(--line);
    background: none;
    color: var(--muted);
    font: inherit;
    font-size: 13px;
  }
  .hint {
    font-size: 13px;
    color: var(--muted);
    margin: 8px 2px 0;
    line-height: 1.4;
  }
</style>
