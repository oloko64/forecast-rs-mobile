<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import { PALETTES } from "$lib/theme";
  import { PROVIDER_NAMES, type Config, type Provider, type SettingsPatch, type ThemeMode } from "$lib/types";

  let {
    config,
    dayChoices,
    onchange,
    onclose,
  }: {
    config: Config;
    dayChoices: number[];
    onchange: (patch: SettingsPatch) => void;
    onclose: () => void;
  } = $props();

  const MODES: { id: ThemeMode; name: string }[] = [
    { id: "system", name: "System" },
    { id: "light", name: "Light" },
    { id: "dark", name: "Dark" },
  ];
  const PROVIDERS = Object.entries(PROVIDER_NAMES) as [Provider, string][];
</script>

<Sheet label="Settings" {onclose}>
  <div class="scroll">
    <h2>Settings</h2>

    <h3>Appearance</h3>
    <div class="segmented">
      {#each MODES as m (m.id)}
        <button class:active={config.theme_mode === m.id} onclick={() => onchange({ theme_mode: m.id })}>
          {m.name}
        </button>
      {/each}
    </div>

    <h3>Color theme</h3>
    <div class="palettes">
      {#each PALETTES as p (p.id)}
        <button class="palette" class:active={config.palette === p.id} onclick={() => onchange({ palette: p.id })}>
          <span class="swatches">
            {#each p.colors as c}<i style:background={c}></i>{/each}
          </span>
          <span>{p.name}</span>
        </button>
      {/each}
    </div>

    <h3>Days to forecast</h3>
    <div class="segmented">
      {#each dayChoices as d}
        <button class:active={config.days === d} onclick={() => onchange({ days: d })}>{d}</button>
      {/each}
    </div>

    <h3>Weather data</h3>
    <div class="segmented">
      {#each PROVIDERS as [id, name] (id)}
        <button class:active={config.provider === id} onclick={() => onchange({ provider: id })}>{name}</button>
      {/each}
    </div>
    <p class="hint">
      If {PROVIDER_NAMES[config.provider]} fails 3 times in a row, the app switches to
      {PROVIDER_NAMES[config.provider === "open_meteo" ? "met_norway" : "open_meteo"]} for a while.
      MET Norway forecasts up to ~10 days.
    </p>
  </div>
</Sheet>

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
  .segmented {
    display: flex;
    gap: 6px;
    padding: 4px;
    border-radius: 14px;
    background: var(--card);
    border: 1px solid var(--line);
  }
  .segmented button {
    flex: 1;
    padding: 10px 0;
    border: none;
    border-radius: 10px;
    background: none;
    color: var(--fg);
    font: inherit;
    font-weight: 600;
  }
  .segmented button.active {
    background: var(--accent);
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
  }
  .swatches i:first-child {
    margin-left: 0;
  }
  .hint {
    font-size: 13px;
    color: var(--muted);
    margin: 8px 2px 0;
    line-height: 1.4;
  }
</style>
