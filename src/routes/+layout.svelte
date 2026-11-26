<script lang="ts">
  import "../app.css";
  import { applyCachedTheme } from "$lib/theme";

  let { children } = $props();
  // Before the first paint, from the last session; the saved config confirms it later.
  applyCachedTheme();

  // A focused input can still make the WebView scroll the document to reveal it,
  // leaving fixed sheets shifted after the keyboard closes. Keep it at the top.
  $effect(() => {
    const reset = () => {
      if (window.scrollY !== 0 || document.documentElement.scrollTop !== 0) window.scrollTo(0, 0);
    };
    window.addEventListener("scroll", reset, { passive: true });
    window.visualViewport?.addEventListener("resize", reset);
    return () => {
      window.removeEventListener("scroll", reset);
      window.visualViewport?.removeEventListener("resize", reset);
    };
  });
</script>

{@render children()}
