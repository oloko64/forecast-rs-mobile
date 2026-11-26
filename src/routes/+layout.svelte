<script lang="ts">
  import "../app.css";
  import { applyCachedTheme } from "$lib/theme";

  let { children } = $props();
  // Before the first paint, from the last session; the saved config confirms it later.
  applyCachedTheme();
</script>

{@render children()}
