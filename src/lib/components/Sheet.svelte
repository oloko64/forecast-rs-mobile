<script lang="ts">
  import type { Snippet } from "svelte";
  import { cubicInOut, cubicOut } from "svelte/easing";
  import type { TransitionConfig } from "svelte/transition";

  let { label, onclose, children }: { label: string; onclose: () => void; children: Snippet } =
    $props();

  const reduced = () => matchMedia("(prefers-reduced-motion: reduce)").matches;

  // Swipe down from the handle to close.
  let sheet = $state<HTMLDivElement>();
  let dragY = $state(0);
  let dragging = $state(false);
  /** Closed by a swipe: the sheet already slid out, so skip the outro. */
  let swiped = false;
  let startY = 0;
  let lastY = 0;
  let lastT = 0;
  let velocity = 0;

  const progress = $derived(sheet ? Math.min(1, dragY / sheet.offsetHeight) : 0);

  function down(e: PointerEvent) {
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragging = true;
    startY = lastY = e.clientY;
    lastT = e.timeStamp;
    velocity = 0;
  }

  function move(e: PointerEvent) {
    if (!dragging) return;
    const dy = e.clientY - startY;
    // Follows the finger down; resists a little when pulled up.
    dragY = dy > 0 ? dy : -12 * (1 - Math.exp(dy / 60));
    const dt = e.timeStamp - lastT;
    if (dt > 0) velocity = (e.clientY - lastY) / dt;
    lastY = e.clientY;
    lastT = e.timeStamp;
  }

  function up() {
    if (!dragging) return;
    dragging = false;
    const height = sheet?.offsetHeight ?? 0;
    const far = dragY > Math.min(120, height * 0.25);
    const flick = velocity > 0.5 && dragY > 20;
    if (far || flick) {
      swiped = true;
      dragY = height;
      setTimeout(onclose, reduced() ? 0 : 260);
    } else {
      dragY = 0;
    }
  }

  function slideIn(_node: Element): TransitionConfig {
    return {
      duration: reduced() ? 0 : 360,
      easing: cubicOut,
      css: (_t, u) => `transform: translateY(${u * 100}%);`,
    };
  }
  function slideOut(_node: Element): TransitionConfig {
    return {
      duration: swiped || reduced() ? 0 : 320,
      easing: cubicInOut,
      css: (_t, u) => `transform: translateY(${u * 100}%);`,
    };
  }
  function fadeBackdrop(_node: Element): TransitionConfig {
    return {
      duration: swiped || reduced() ? 0 : 280,
      easing: cubicInOut,
      css: (t) => `opacity: ${t};`,
    };
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="backdrop"
  class:dragging
  style:opacity={1 - progress}
  onclick={onclose}
  in:fadeBackdrop|global
  out:fadeBackdrop|global
></div>
<div
  class="sheet"
  class:dragging
  bind:this={sheet}
  role="dialog"
  aria-label={label}
  style:transform={dragY ? `translateY(${dragY}px)` : undefined}
  in:slideIn|global
  out:slideOut|global
>
  <div
    class="handle"
    role="presentation"
    onpointerdown={down}
    onpointermove={move}
    onpointerup={up}
    onpointercancel={up}
  >
    <div class="grip"></div>
  </div>
  {@render children()}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    z-index: 10;
    transition: opacity 0.26s ease-in-out;
  }
  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    max-width: 640px;
    margin: 0 auto;
    max-height: 88dvh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: 20px 20px 0 0;
    padding: 0 0 calc(12px + env(safe-area-inset-bottom));
    z-index: 11;
    box-shadow: 0 -8px 30px rgb(0 0 0 / 0.15);
    /* Springs back or slides out after a swipe; follows the finger while dragging. */
    transition: transform 0.26s cubic-bezier(0.32, 0.72, 0, 1);
    will-change: transform;
  }
  .dragging,
  .backdrop.dragging {
    transition: none;
  }
  /* A tall, full-width grab area: easy to hit with a thumb. */
  .handle {
    flex: none;
    display: grid;
    place-items: center;
    height: 26px;
    margin-bottom: 4px;
    touch-action: none;
    cursor: grab;
  }
  .grip {
    width: 40px;
    height: 5px;
    border-radius: 3px;
    background: var(--line);
    transition:
      background-color 0.2s ease-in-out,
      width 0.2s ease-in-out;
  }
  .dragging .grip {
    width: 52px;
    background: var(--muted);
  }
</style>
