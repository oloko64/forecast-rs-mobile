<script lang="ts">
  import type { Weather } from "$lib/types";

  /** Animated SVG weather icon. `animated` off draws the same icon still. */
  let { weather, size = 32, animated = false }: { weather: Weather; size?: number; animated?: boolean } =
    $props();

  type Scene = {
    sun: "none" | "full" | "behind";
    clouds: 0 | 1 | 2;
    drops: number;
    flakes: number;
    bolt: boolean;
    fog: boolean;
    dark: boolean;
  };

  const scene = $derived.by((): Scene => {
    const d = weather.description;
    const base: Scene = { sun: "none", clouds: 1, drops: 0, flakes: 0, bolt: false, fog: false, dark: false };
    switch (weather.kind) {
      case "clear":
        return d === "Mainly clear" ? { ...base, sun: "behind", clouds: 1 } : { ...base, sun: "full", clouds: 0 };
      case "cloudy":
        return d === "Partly cloudy" ? { ...base, sun: "behind" } : { ...base, clouds: 2 };
      case "fog":
        return { ...base, fog: true };
      case "rain": {
        const drops = /Heavy/.test(d) ? 5 : /Drizzle|Slight/.test(d) ? 3 : 4;
        const showers = /showers/.test(d);
        return { ...base, sun: showers ? "behind" : "none", drops, flakes: /Freezing/.test(d) ? 2 : 0, dark: /Heavy/.test(d) };
      }
      case "snow":
        return { ...base, flakes: /Heavy/.test(d) ? 5 : 3, sun: /showers/.test(d) ? "behind" : "none" };
      case "storm":
        return { ...base, clouds: 2, drops: 3, bolt: true, dark: true };
      default:
        return base;
    }
  });

  // Spread precipitation under the cloud.
  const xs = (n: number) => Array.from({ length: n }, (_, i) => 22 + (i * 22) / Math.max(1, n - 1));
  const CLOUD = "M19 47h27a9.5 9.5 0 0 0 .6-19A13.5 13.5 0 0 0 20.6 26 10.5 10.5 0 0 0 19 47z";
</script>

<svg
  class:animated
  width={size}
  height={size}
  viewBox="0 0 64 64"
  role="img"
  aria-label={weather.description}
>
  {#if scene.sun !== "none"}
    <g
      class="sun"
      transform={scene.sun === "behind" ? "translate(-9 -9) scale(0.8)" : ""}
      style:transform-origin="32px 32px"
    >
      <g class="rays">
        {#each Array(8) as _, i}
          <line
            x1="32"
            y1="9"
            x2="32"
            y2="14"
            transform="rotate({i * 45} 32 32)"
            stroke="var(--sun)"
            stroke-width="3.5"
            stroke-linecap="round"
          />
        {/each}
      </g>
      <circle cx="32" cy="32" r="11" fill="var(--sun)" />
    </g>
  {/if}

  {#if scene.clouds === 2}
    <path class="cloud back" d={CLOUD} transform="translate(9 -8) scale(0.78)" fill="var(--cloud-back)" />
  {/if}
  {#if scene.clouds > 0}
    <path class="cloud front" d={CLOUD} fill={scene.dark ? "var(--cloud-dark)" : "var(--cloud)"} />
  {/if}

  {#if scene.bolt}
    <path class="bolt" d="M34 42l-7 11h6l-3 9 10-13h-6l3-7z" fill="var(--sun)" />
  {/if}

  {#each xs(scene.drops) as x, i}
    <line
      class="drop"
      x1={x}
      y1="51"
      x2={x - 2}
      y2="57"
      stroke="var(--rain)"
      stroke-width="3"
      stroke-linecap="round"
      style:animation-delay="{i * 0.23}s"
    />
  {/each}

  {#each xs(scene.flakes) as x, i}
    <circle class="flake" cx={x + (i % 2) * 3} cy="53" r="2.6" fill="var(--snow)" style:animation-delay="{i * 0.45}s" />
  {/each}

  {#if scene.fog}
    {#each [50, 56] as y, i}
      <line
        class="fog"
        x1="16"
        x2="48"
        y1={y}
        y2={y}
        stroke="var(--cloud-back)"
        stroke-width="3.5"
        stroke-linecap="round"
        style:animation-delay="{i * 0.8}s"
      />
    {/each}
  {/if}
</svg>

<style>
  svg {
    display: block;
    flex: none;
    overflow: visible;
    --sun: #f7b733;
    --cloud: light-dark(#e3e9f2, #c9d3e3);
    --cloud-back: light-dark(#bcc7d8, #7f8ca3);
    --cloud-dark: light-dark(#a3afc2, #8d9ab0);
    --snow: light-dark(#9fb8d6, #e9f1ff);
  }
  .cloud {
    stroke: rgb(0 0 0 / 0.06);
    stroke-width: 1;
  }

  /* Only animated icons move; the rest of the list stays calm. */
  .animated .rays {
    transform-origin: 32px 32px;
    animation: rotate 14s linear infinite;
  }
  .animated .sun {
    animation: glow 3.2s ease-in-out infinite alternate;
  }
  .animated .cloud.front {
    animation: drift 4.5s ease-in-out infinite alternate;
  }
  .animated .cloud.back {
    animation: drift-back 5.5s ease-in-out infinite alternate;
  }
  .animated .drop {
    animation: fall 1.15s cubic-bezier(0.55, 0, 0.85, 0.4) infinite;
  }
  .animated .flake {
    animation: snow 2.4s ease-in-out infinite;
  }
  .animated .bolt {
    animation: flash 2.6s ease-in-out infinite;
  }
  .animated .fog {
    animation: slide 3.5s ease-in-out infinite alternate;
  }

  @keyframes rotate {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes glow {
    to {
      filter: brightness(1.12);
    }
  }
  @keyframes drift {
    from {
      transform: translateX(-1.5px);
    }
    to {
      transform: translateX(2px);
    }
  }
  @keyframes drift-back {
    from {
      transform: translate(10px, -8px) scale(0.78);
    }
    to {
      transform: translate(7px, -8px) scale(0.78);
    }
  }
  @keyframes fall {
    0% {
      transform: translateY(-5px);
      opacity: 0;
    }
    20% {
      opacity: 1;
    }
    100% {
      transform: translateY(8px);
      opacity: 0;
    }
  }
  @keyframes snow {
    0% {
      transform: translate(0, -5px);
      opacity: 0;
    }
    25% {
      opacity: 1;
    }
    50% {
      transform: translate(2px, 2px);
    }
    100% {
      transform: translate(-1px, 9px);
      opacity: 0;
    }
  }
  @keyframes flash {
    0%,
    60%,
    100% {
      opacity: 1;
    }
    66% {
      opacity: 0.15;
    }
    72% {
      opacity: 1;
    }
    78% {
      opacity: 0.3;
    }
  }
  @keyframes slide {
    from {
      transform: translateX(-3px);
    }
    to {
      transform: translateX(3px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    svg * {
      animation: none !important;
    }
  }
</style>
