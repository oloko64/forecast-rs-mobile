<script lang="ts">
  import type { Snippet } from "svelte";

  let { label, onclose, children }: { label: string; onclose: () => void; children: Snippet } =
    $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose}></div>
<div class="sheet" role="dialog" aria-label={label}>
  <div class="grip"></div>
  {@render children()}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    z-index: 10;
    animation: fade 0.15s ease-out;
  }
  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    max-width: 640px;
    margin: 0 auto;
    max-height: 85dvh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: 20px 20px 0 0;
    padding: 8px 0 calc(12px + env(safe-area-inset-bottom));
    z-index: 11;
    animation: up 0.2s ease-out;
  }
  .grip {
    flex: none;
    width: 40px;
    height: 4px;
    border-radius: 2px;
    background: var(--line);
    margin: 0 auto 10px;
  }
  @keyframes up {
    from {
      transform: translateY(100%);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
</style>
