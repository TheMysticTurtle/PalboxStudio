<script lang="ts">
  // Self-hosted fonts (offline; no CDN) — the game-like feel.
  import "@fontsource/rajdhani/500.css";
  import "@fontsource/rajdhani/600.css";
  import "@fontsource/rajdhani/700.css";
  import "@fontsource/barlow/400.css";
  import "@fontsource/barlow/500.css";
  import "@fontsource/barlow/600.css";
  import "@fontsource/barlow-semi-condensed/600.css";
  import "$lib/styles/tokens.css";
  import { onMount } from "svelte";
  import { loadRefData } from "$lib/data/refdata.svelte";
  import { openBoxFile, startBoxSourceMonitor } from "$lib/stores/box.svelte";
  import {
    boxPreferences,
    loadBoxPreferences,
  } from "$lib/stores/boxPreferences.svelte";
  import { loadUserLibrary } from "$lib/stores/library.svelte";

  let { children } = $props();

  // Load the static reference tables (passives/moves/species/elements/schema) once.
  $effect(() => {
    loadRefData();
    loadUserLibrary();
  });

  onMount(() => {
    loadBoxPreferences();
    if (boxPreferences.autoReopen && boxPreferences.lastBoxPath) {
      void openBoxFile(boxPreferences.lastBoxPath, { automatic: true });
    }
    return startBoxSourceMonitor();
  });
</script>

{@render children()}

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }
  :global(body) {
    background: #0a0d12;
    color: var(--text-1);
    font-family: var(--font-body);
    overflow: hidden;
  }
  :global(*) {
    box-sizing: border-box;
  }
  :global(::-webkit-scrollbar) {
    width: 10px;
    height: 10px;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: rgba(176, 96, 224, 0.35);
    border-radius: 6px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
</style>
