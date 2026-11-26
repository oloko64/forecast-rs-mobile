<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { slideUp } from "$lib/motion";

  let { label, onclose, children }: { label: string; onclose: () => void; children: Snippet } =
    $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose} transition:fade|global={{ duration: 280, easing: cubicInOut }}></div>
<div class="sheet" role="dialog" aria-label={label} transition:slideUp|global>
  <div class="grip"></div>
  {@render children()}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    z-index: 10;
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
    padding: 8px 0 calc(12px + env(safe-area-inset-bottom));
    z-index: 11;
    box-shadow: 0 -8px 30px rgb(0 0 0 / 0.15);
  }
  .grip {
    flex: none;
    width: 40px;
    height: 4px;
    border-radius: 2px;
    background: var(--line);
    margin: 0 auto 10px;
  }
</style>
