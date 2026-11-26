<script lang="ts">
  /**
   * A refresh arrow that spins while `spinning`. When loading ends it finishes
   * the current turn instead of snapping back, and always does at least one turn.
   */
  let { spinning, size = 24 }: { spinning: boolean; size?: number } = $props();

  let active = $state(false);
  $effect(() => {
    if (spinning) active = true;
  });
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class:active
  onanimationiteration={() => {
    if (!spinning) active = false;
  }}
>
  <path d="M20 12a8 8 0 1 1-2.35-5.65" />
  <path d="M20 4v4.5h-4.5" />
</svg>

<style>
  svg {
    display: block;
    flex: none;
    transform-origin: 50% 50%;
    will-change: transform;
  }
  .active {
    animation: spin 0.9s cubic-bezier(0.45, 0.05, 0.55, 0.95) infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
