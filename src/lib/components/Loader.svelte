<script lang="ts">
  import { onMount } from "svelte";
  import type { Weather } from "$lib/types";
  import { t, type Key } from "$lib/i18n.svelte";
  import WeatherIcon from "./WeatherIcon.svelte";

  /** First-load screen: a weather icon cycling through conditions, rotating messages
   *  and a shimmering preview of the page. Tap the icon to skip to the next weather. */

  const SCENES: Weather[] = [
    { description: "Clear sky", icon: "", kind: "clear" },
    { description: "Partly cloudy", icon: "", kind: "cloudy" },
    { description: "Moderate rain", icon: "", kind: "rain" },
    { description: "Thunderstorm", icon: "", kind: "storm" },
    { description: "Slight snow fall", icon: "", kind: "snow" },
    { description: "Rain showers", icon: "", kind: "rain" },
  ];
  const MESSAGES: Key[] = ["loaderSky", "loaderClouds", "loaderStations", "loaderAlmost"];

  let scene = $state(0);
  let message = $state(0);
  let slow = $state(false);
  let popKey = $state(0);
  let cycle: ReturnType<typeof setInterval>;

  function next() {
    scene = (scene + 1) % SCENES.length;
  }

  function tap() {
    next();
    popKey++;
    // Restart the timer so a tap doesn't get followed by an immediate auto change.
    clearInterval(cycle);
    cycle = setInterval(next, 2200);
  }

  onMount(() => {
    cycle = setInterval(next, 2200);
    const messages = setInterval(() => (message = Math.min(message + 1, MESSAGES.length - 1)), 2600);
    const slowTimer = setTimeout(() => (slow = true), 12_000);
    return () => {
      clearInterval(cycle);
      clearInterval(messages);
      clearTimeout(slowTimer);
    };
  });
</script>

<div class="loader" role="status" aria-live="polite">
  <button class="stage" onclick={tap} aria-label={t("loaderTap")}>
    {#key popKey}
      <span class="pop">
        <span class="float">
          {#each SCENES as w, i}
            <span class="scene" class:shown={i === scene} aria-hidden={i !== scene}>
              <WeatherIcon weather={w} size={120} animated={i === scene} />
            </span>
          {/each}
        </span>
      </span>
    {/key}
    <span class="shadow"></span>
  </button>

  <p class="message">
    {#key slow ? "slow" : message}
      <span class="text">{slow ? t("loaderSlow") : t(MESSAGES[message])}</span>
    {/key}
    <span class="dots"><i></i><i></i><i></i></span>
  </p>
  <p class="hint">{t("loaderTap")}</p>

  <!-- Shimmering preview of the page that is about to appear. -->
  <div class="skeleton" aria-hidden="true">
    <div class="days">
      {#each Array(5) as _, i}<div class="bone day" style:--d="{i * 0.08}s"></div>{/each}
    </div>
    <div class="bone chart"></div>
    {#each Array(3) as _, i}<div class="bone row" style:--d="{0.2 + i * 0.08}s"></div>{/each}
  </div>
</div>

<style>
  .loader {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 6vh 16px 16px;
    min-height: 0;
  }
  .stage {
    position: relative;
    display: grid;
    place-items: center;
    width: 160px;
    height: 160px;
    border: none;
    background: none;
    padding: 0;
    -webkit-tap-highlight-color: transparent;
  }
  .stage:active {
    transform: none;
  }
  .pop,
  .float {
    display: grid;
  }
  .pop {
    animation: pop 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .float {
    animation: float 3s ease-in-out infinite;
  }
  /* All scenes stacked; the shown one crossfades in with a slight zoom. */
  .scene {
    grid-area: 1 / 1;
    opacity: 0;
    transform: scale(0.82) rotate(-6deg);
    transition:
      opacity 0.6s ease-in-out,
      transform 0.7s cubic-bezier(0.34, 1.3, 0.64, 1);
  }
  .scene.shown {
    opacity: 1;
    transform: none;
  }
  .shadow {
    position: absolute;
    bottom: 6px;
    width: 70px;
    height: 10px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--fg) 14%, transparent);
    animation: shadow 3s ease-in-out infinite;
  }
  .message {
    display: flex;
    align-items: baseline;
    justify-content: center;
    min-height: 26px;
    margin: 14px 0 2px;
    font-size: 17px;
    font-weight: 600;
    text-align: center;
  }
  .text {
    animation: rise 0.45s ease-in-out;
  }
  .dots {
    display: inline-flex;
    gap: 3px;
    margin-left: 4px;
  }
  .dots i {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    animation: dot 1.2s ease-in-out infinite;
  }
  .dots i:nth-child(2) {
    animation-delay: 0.15s;
  }
  .dots i:nth-child(3) {
    animation-delay: 0.3s;
  }
  .hint {
    margin: 0 0 22px;
    font-size: 12px;
    color: var(--muted);
  }

  .skeleton {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .days {
    display: flex;
    gap: 8px;
    overflow: hidden;
  }
  .bone {
    border-radius: 16px;
    background: linear-gradient(
        100deg,
        transparent 30%,
        color-mix(in srgb, var(--fg) 7%, transparent) 50%,
        transparent 70%
      )
      var(--card);
    background-size: 250% 100%;
    animation:
      shimmer 1.6s ease-in-out infinite,
      appear 0.6s ease-out both;
    animation-delay: var(--d, 0s), var(--d, 0s);
  }
  .day {
    flex: 0 0 70px;
    height: 128px;
    border-radius: 18px;
  }
  .chart {
    height: 150px;
    --d: 0.15s;
  }
  .row {
    height: 46px;
    border-radius: 12px;
  }

  @keyframes float {
    50% {
      transform: translateY(-10px);
    }
  }
  @keyframes shadow {
    50% {
      transform: scaleX(0.75);
      opacity: 0.6;
    }
  }
  @keyframes pop {
    from {
      transform: scale(0.8);
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @keyframes dot {
    0%,
    100% {
      transform: translateY(0);
      opacity: 0.35;
    }
    40% {
      transform: translateY(-5px);
      opacity: 1;
    }
  }
  @keyframes shimmer {
    from {
      background-position: 120% 0;
    }
    to {
      background-position: -120% 0;
    }
  }
  @keyframes appear {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
</style>
