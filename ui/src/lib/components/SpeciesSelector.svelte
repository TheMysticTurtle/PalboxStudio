<script lang="ts">
  import { ref, baseSpeciesCode } from "$lib/data/refdata.svelte";
  import SpeciesFilter from "./SpeciesFilter.svelte";
  import SpeciesMiniTile from "./SpeciesMiniTile.svelte";
  import {
    createSpeciesFilter,
    filterSpecies,
  } from "$lib/data/speciesFilter.svelte";

  let {
    open = $bindable(false),
    current = "",
    onpick,
  }: {
    open?: boolean;
    /** CharacterID of the pal's current species (highlighted). */
    current?: string;
    onpick: (code: string) => void;
  } = $props();

  const filter = createSpeciesFilter();

  // The reference bundle retains every engine row so unusual saves can still
  // be decoded. The selector exposes only the audited canonical Palbox roster.
  let allSpecies = $derived(
    ref.species.filter((species) => species.palboxSelectable).sort((a, b) => {
      const ai = a.deckIndex >= 0 ? a.deckIndex : 1e9;
      const bi = b.deckIndex >= 0 ? b.deckIndex : 1e9;
      return ai !== bi ? ai - bi : a.name.localeCompare(b.name);
    }),
  );
  let filtered = $derived(filterSpecies(allSpecies, filter));

  // Match the pal's current species tolerant of a BOSS_ prefix.
  const baseCode = $derived(baseSpeciesCode(current));

  function choose(code: string) {
    onpick(code);
    open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") open = false;
  }
</script>

<svelte:window onkeydown={open ? onKey : undefined} />

{#if open}
  <div class="scrim" role="presentation" onclick={() => (open = false)}></div>
  <div class="panel" role="dialog" aria-label="Select species" aria-modal="true">
    <header>
      <span class="diamond"></span>
      <h2>SELECT SPECIES</h2>
      <span class="count">{filtered.length} / {allSpecies.length}</span>
      <button class="x" onclick={() => (open = false)} aria-label="Close">✕</button>
    </header>

    <div class="filterbar">
      <SpeciesFilter {filter} showCategories={false} placeholder="Search species…" />
    </div>

    <div class="grid">
      {#each filtered as sp (sp.code)}
        <SpeciesMiniTile species={sp} current={sp.code === baseCode} onselect={choose} />
      {/each}
      {#if !filtered.length}<div class="empty">No species match these filters.</div>{/if}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 90;
    background: rgba(10, 8, 15, 0.62);
    backdrop-filter: blur(4px);
  }
  .panel {
    position: fixed;
    z-index: 91;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(920px, 92vw);
    height: min(78vh, 820px);
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    border: 1px solid rgba(176, 96, 224, 0.35);
    background: linear-gradient(155deg, rgba(28, 20, 38, 0.98), rgba(17, 14, 24, 0.99));
    box-shadow: 0 30px 90px rgba(0, 0, 0, 0.6), 0 0 50px rgba(176, 96, 224, 0.18);
    overflow: hidden;
  }
  header { display: flex; align-items: center; gap: 11px; padding: 15px 20px; border-bottom: 1px solid rgba(176, 96, 224, 0.22); }
  .diamond { width: 10px; height: 10px; transform: rotate(45deg); background: var(--accent-purple); box-shadow: 0 0 8px var(--accent-purple); }
  h2 { margin: 0; font-family: var(--font-head); font-weight: 700; font-size: 17px; letter-spacing: 0.14em; color: #e7daf4; }
  .count { color: #9782a8; font-size: 12.5px; font-variant-numeric: tabular-nums; }
  .x { margin-left: auto; width: 30px; height: 30px; border-radius: 8px; cursor: pointer; color: #b0a0be; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.12); font-size: 13px; }
  .x:hover { color: #fff; background: rgba(224, 90, 90, 0.18); border-color: rgba(224, 90, 90, 0.4); }

  .filterbar { padding: 14px 20px; border-bottom: 1px solid rgba(255, 255, 255, 0.07); }

  .grid {
    flex: 1;
    overflow: auto;
    padding: 16px 20px 22px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(145px, 1fr));
    gap: 11px;
    align-content: start;
  }
  .empty { grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 40px; font-size: 13px; }
</style>
