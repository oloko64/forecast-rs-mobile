<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import ColorPicker from "./ColorPicker.svelte";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import type { TransitionConfig } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { COLOR_ROLES, PALETTES, paletteColors, type ColorRole } from "$lib/theme";
  import { LANGUAGES, t } from "$lib/i18n.svelte";
  import {
    PROVIDER_NAMES,
    type Config,
    type PaletteColors,
    type Provider,
    type SettingsPatch,
    type Language,
    type ThemeMode,
    type Units,
    type WidgetOptions,
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

  // Derived so labels follow the language setting.
  const MODES = $derived<{ id: ThemeMode; name: string }[]>([
    { id: "system", name: t("system") },
    { id: "light", name: t("light") },
    { id: "dark", name: t("dark") },
  ]);
  const LANGS = $derived(LANGUAGES.map((l) => ({ id: l.id, name: l.name || t("system") })));
  const UNITS: { id: Units; name: string }[] = [
    { id: "celsius", name: "°C  Celsius" },
    { id: "fahrenheit", name: "°F  Fahrenheit" },
  ];
  const PROVIDERS = Object.entries(PROVIDER_NAMES) as [Provider, string][];
  const minutes = (list: number[]) => list.map((m) => ({ id: m, name: m < 60 ? `${m}m` : `${m / 60}h` }));
  const minuteLabel = (m: number) => (m < 60 ? `${m}m` : `${m / 60}h`);
  const REFRESH = minutes([10, 15, 30, 60, 120]);
  const RESUME = minutes([15, 30, 60, 120]);
  const WIDGET_REFRESH = minutes([15, 30, 60, 120, 240]);
  type WidgetPart = "location" | "now" | "summary" | "day_picker" | "refresh_button" | "rain";
  const WIDGET_PARTS: { id: WidgetPart; name: Parameters<typeof t>[0] }[] = [
    { id: "location", name: "widgetLocation" },
    { id: "now", name: "widgetNow" },
    { id: "summary", name: "widgetSummary" },
    { id: "day_picker", name: "widgetDayPicker" },
    { id: "refresh_button", name: "widgetRefreshButton" },
    { id: "rain", name: "widgetRain" },
  ];

  const CLOCKS = $derived<{ id: WidgetOptions["clock"]; name: string }[]>([
    { id: "off", name: t("clockOff") },
    { id: "small", name: t("clockSmall") },
    { id: "big", name: t("clockBig") },
  ]);
  const RETURN_AFTER = [30, 60, 120, 300, 600, 1200].map((s) => ({ id: s, name: s < 60 ? `${s}s` : `${s / 60}m` }));
  const BACKGROUNDS = [100, 75, 50, 25, 0].map((p) => ({ id: p, name: `${p}%` }));

  /** The installed app's version, from tauri.conf.json. */
  let version = $state("");
  getVersion()
    .then((v) => (version = v))
    .catch(() => {});

  // The sheet opens on a list of sections; each one is its own page.
  type Page = "appearance" | "forecast" | "updates" | "widget";
  let page = $state<Page | null>(null);
  /** +1 when going into a section, -1 when coming back: sets the slide direction. */
  let direction = 1;
  function open(p: Page | null) {
    direction = p ? 1 : -1;
    if (!p) {
      flush();
      editing = null;
    }
    page = p;
  }
  const reduced = () => matchMedia("(prefers-reduced-motion: reduce)").matches;
  function pageIn(_node: Element): TransitionConfig {
    return {
      duration: reduced() ? 0 : 300,
      easing: cubicInOut,
      css: (t, u) => `opacity: ${t}; transform: translateX(${u * 40 * direction}px);`,
    };
  }

  const modeName = $derived(MODES.find((m) => m.id === config.theme_mode)?.name ?? "");
  const langName = $derived(LANGS.find((l) => l.id === config.language)?.name ?? "");
  const paletteName = $derived(t(PALETTES.find((p) => p.id === config.palette)!.name));
  const shownParts = $derived(WIDGET_PARTS.filter((p) => config.widget[p.id]).length);
  const SECTIONS = $derived<{ id: Page; title: string; summary: string; icon: string }[]>([
    { id: "appearance", title: t("appearance"), summary: `${modeName} · ${paletteName} · ${langName}`, icon: "🎨" },
    {
      id: "forecast",
      title: t("sectionForecast"),
      summary: `${config.units === "fahrenheit" ? "°F" : "°C"} · ${t("daysCount", { value: config.days })} · ${PROVIDER_NAMES[config.provider]}`,
      icon: "🌦️",
    },
    {
      id: "updates",
      title: t("sectionUpdates"),
      summary: t("openAndBack", { open: minuteLabel(config.refresh_minutes), back: minuteLabel(config.resume_minutes) }),
      icon: "🔄",
    },
    {
      id: "widget",
      title: t("sectionWidget"),
      summary: `${t("everyMinutes", { value: minuteLabel(config.widget_refresh_minutes) })} · ${t("widgetShowCount", { value: shownParts })}`,
      icon: "📱",
    },
  ]);

  function toggleWidgetPart(id: (typeof WIDGET_PARTS)[number]["id"]) {
    onchange({ widget: { ...config.widget, [id]: !config.widget[id] } });
  }
  const setWidget = (patch: Partial<WidgetOptions>) => onchange({ widget: { ...config.widget, ...patch } });

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

<Sheet label={t("settings")} onclose={close}>
  <div class="scroll">
    <div class="title-row">
      {#if page}
        <button class="back" onclick={() => open(null)} aria-label={t("back")}>
          <span class="chev"><Icon name="chevron-down" size={22} /></span>
        </button>
      {/if}
      <h2>{page ? SECTIONS.find((x) => x.id === page)!.title : t("settings")}</h2>
    </div>

    {#key page}
      <div class="page" in:pageIn>
        {#if page === null}
          <div class="sections">
            {#each SECTIONS as section (section.id)}
              <button class="section" onclick={() => open(section.id)}>
                <span class="section-icon">{section.icon}</span>
                <span class="section-text">
                  <span class="section-title">{section.title}</span>
                  <span class="section-summary">{section.summary}</span>
                </span>
                <span class="chev right"><Icon name="chevron-down" size={20} /></span>
              </button>
            {/each}
          </div>
          {#if version}<p class="version">{t("version", { value: version })}</p>{/if}
        {:else if page === "appearance"}
          <h3>{t("language")}</h3>
          {@render segmented(LANGS, config.language, (id) => onchange({ language: id as Language }))}

          <h3>{t("themeMode")}</h3>
          {@render segmented(MODES, config.theme_mode, (id) => onchange({ theme_mode: id as ThemeMode }))}

          <h3>{t("colorTheme")}</h3>
          <div class="palettes">
            {#each PALETTES as p (p.id)}
              {@const c = paletteColors(p.id, p.id === config.palette ? custom : config.custom_colors[p.id])}
              <button class="palette" class:active={config.palette === p.id} onclick={() => pickPalette(p.id)}>
                <span class="swatches">
                  <i style:background={c.accent}></i><i style:background={c.temp}></i><i style:background={c.rain}></i>
                </span>
                <span>{t(p.name)}{config.custom_colors[p.id] ? " *" : ""}</span>
              </button>
            {/each}
          </div>

          <div class="custom">
            <span class="custom-label">{t("customize")}</span>
            {#each COLOR_ROLES as role (role.id)}
              <button
                class="role"
                class:active={editing === role.id}
                onclick={() => (editing = editing === role.id ? null : role.id)}
              >
                <i style:background={colors[role.id]}></i>{t(role.name)}
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
              {t("resetColors", { palette: paletteName })}
            </button>
          {/if}
        {:else if page === "forecast"}
          <h3>{t("temperatureUnit")}</h3>
          {@render segmented(UNITS, config.units, (id) => onchange({ units: id as Units }))}

          <h3>{t("daysToForecast")}</h3>
          {@render segmented(
            dayChoices.map((d) => ({ id: d, name: String(d) })),
            config.days,
            (id) => onchange({ days: id as number }),
          )}

          <h3>{t("weatherData")}</h3>
          {@render segmented(
            PROVIDERS.map(([id, name]) => ({ id, name })),
            config.provider,
            (id) => onchange({ provider: id as Provider }),
          )}
          <p class="hint">
            {t("fallbackHint", {
              preferred: PROVIDER_NAMES[config.provider],
              other: PROVIDER_NAMES[config.provider === "open_meteo" ? "met_norway" : "open_meteo"],
            })}
          </p>
        {:else if page === "updates"}
          <h3>{t("refreshWhileOpen")}</h3>
          {@render segmented(REFRESH, config.refresh_minutes, (id) => onchange({ refresh_minutes: id as number }))}

          <h3>{t("whenComingBack")}</h3>
          {@render segmented(RESUME, config.resume_minutes, (id) => onchange({ resume_minutes: id as number }))}
          <p class="hint">{t("whenComingBackHint")}</p>
        {:else if page === "widget"}
          <h3>{t("widgetRefresh")}</h3>
          {@render segmented(WIDGET_REFRESH, config.widget_refresh_minutes, (id) =>
            onchange({ widget_refresh_minutes: id as number }),
          )}
          <p class="hint">{t("widgetRefreshHint")}</p>

          <h3>{t("widgetClock")}</h3>
          {@render segmented(CLOCKS, config.widget.clock, (id) => setWidget({ clock: id as WidgetOptions["clock"] }))}
          <p class="hint">{t("widgetClockHint")}</p>

          <h3>{t("widgetBackground")}</h3>
          {@render segmented(BACKGROUNDS, config.widget.background, (id) => setWidget({ background: id as number }))}
          <p class="hint">{t("widgetBackgroundHint")}</p>

          <h3>{t("widgetShow")}</h3>
          <div class="toggles">
            {#each WIDGET_PARTS as part (part.id)}
              <button
                class="toggle-row"
                role="switch"
                aria-checked={config.widget[part.id]}
                onclick={() => toggleWidgetPart(part.id)}
              >
                <span>{t(part.name)}</span>
                <span class="switch" class:on={config.widget[part.id]}><i></i></span>
              </button>
            {/each}
          </div>

          {#if config.widget.day_picker}
            <div class="toggles">
              <button
                class="toggle-row"
                role="switch"
                aria-checked={config.widget.return_to_today}
                onclick={() => setWidget({ return_to_today: !config.widget.return_to_today })}
              >
                <span>{t("widgetReturnToToday")}</span>
                <span class="switch" class:on={config.widget.return_to_today}><i></i></span>
              </button>
            </div>
            {#if config.widget.return_to_today}
              <div class="return-after">
                {@render segmented(RETURN_AFTER, config.widget.return_after_seconds, (id) =>
                  setWidget({ return_after_seconds: id as number }),
                )}
              </div>
              <p class="hint">{t("widgetReturnToTodayHint")}</p>
            {/if}
          {/if}
          <p class="hint">{t("widgetHowTo")}</p>
        {/if}
      </div>
    {/key}
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
    margin: 0;
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 40px;
    margin-bottom: 4px;
  }
  .back {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin-left: -8px;
    border: none;
    border-radius: 50%;
    background: none;
    color: var(--fg);
  }
  .chev {
    display: flex;
    transform: rotate(90deg);
  }
  .chev.right {
    transform: rotate(-90deg);
    color: var(--muted);
  }
  /* Section list (first page). */
  .sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
  }
  .version {
    text-align: center;
    font-size: 13px;
    color: var(--muted);
    margin: 20px 0 4px;
  }
  .section {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    border-radius: 16px;
    border: 1px solid var(--line);
    background: var(--card);
    color: var(--fg);
    font: inherit;
    text-align: left;
    transition:
      background-color 0.25s ease-in-out,
      transform 0.18s ease-in-out;
  }
  .section:active {
    background: var(--highlight);
  }
  .section-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: 12px;
    background: var(--highlight);
    font-size: 20px;
  }
  .section-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .section-title {
    font-weight: 700;
  }
  .section-summary {
    font-size: 13px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  /* Switch rows (widget parts). */
  .toggles {
    display: flex;
    flex-direction: column;
    border-radius: 14px;
    border: 1px solid var(--line);
    background: var(--card);
    overflow: hidden;
  }
  .toggles + .toggles {
    margin-top: 12px;
  }
  .return-after {
    margin-top: 8px;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border: none;
    border-bottom: 1px solid var(--line);
    background: none;
    color: var(--fg);
    font: inherit;
    text-align: left;
  }
  .toggle-row:last-child {
    border-bottom: none;
  }
  .toggle-row:active {
    transform: none;
    background: var(--highlight);
  }
  .switch {
    position: relative;
    flex: none;
    width: 44px;
    height: 26px;
    border-radius: 13px;
    background: var(--line);
    transition: background-color 0.25s ease-in-out;
  }
  .switch i {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgb(0 0 0 / 0.3);
    transition: transform 0.28s cubic-bezier(0.65, 0, 0.35, 1);
  }
  .switch.on {
    background: var(--accent);
  }
  .switch.on i {
    transform: translateX(18px);
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
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
  }
  /* On its own line: the labels are longer in some languages. */
  .custom-label {
    flex-basis: 100%;
    font-size: 13px;
    color: var(--muted);
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
