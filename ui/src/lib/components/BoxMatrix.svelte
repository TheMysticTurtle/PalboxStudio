<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { ui } from "$lib/stores/ui.svelte";
  import { box, selectSlot } from "$lib/stores/box.svelte";
  import { palToBoxPal, tileDtoToBoxPal } from "$lib/data/mapper";
  import { resolveSpecies } from "$lib/data/refdata.svelte";
  import {
    activeFilterCount,
    createSpeciesFilter,
    speciesMatches,
  } from "$lib/data/speciesFilter.svelte";
  import BoxTile from "./BoxTile.svelte";
  import GroupFilter from "./GroupFilter.svelte";
  import GroupTags from "./GroupTags.svelte";
  import SpeciesFilter from "./SpeciesFilter.svelte";
  import { groupIdsFor, groupNamesFor } from "$lib/stores/library.svelte";
  import { matchesAllGroups } from "$lib/data/groupFilter";

  let search = $state("");
  let sort = $state<"slot" | "name" | "level">("slot");
  const filter = createSpeciesFilter();
  let selectedGroups = $state(new Set<number>());

  let source: BoxPal[] = $derived(
    box.open
      ? box.tiles.map((tile) =>
          tile.slot === box.selectedSlot && box.pal
            ? palToBoxPal(box.pal, tile.slot, groupNamesFor(box.pal.instanceId))
            : tileDtoToBoxPal(tile, groupNamesFor(tile.instanceId)),
        )
      : [],
  );
  let filtered = $derived.by(() => {
    const query = search.trim().toLowerCase();
    const rows = source.filter((pal) => {
      if (
        query
        && !`${pal.name} ${pal.speciesName} ${pal.nickname}`.toLowerCase().includes(query)
      ) return false;
      if (!matchesAllGroups(groupIdsFor(pal.instanceId), selectedGroups)) return false;
      const species = resolveSpecies(pal.species);
      return species ? speciesMatches(species, filter) : activeFilterCount(filter) === 0;
    });
    return rows.sort((a, b) => {
      if (sort === "name") return a.name.localeCompare(b.name) || a.slot - b.slot;
      if (sort === "level") return b.level - a.level || a.name.localeCompare(b.name);
      return a.slot - b.slot;
    });
  });

  function select(slot: number) {
    if (box.open) selectSlot(slot);
  }
  const isSelected = (slot: number) => box.open && box.selectedSlot === slot;
</script>

<div class="overlay">
  <div class="head">
    <span class="diamond"></span>
    <h2>GLOBAL PALBOX</h2>
    <span class="count">{filtered.length} / {source.length} pals</span>
    <button class="collapse" onclick={() => (ui.boxExpanded = false)} aria-label="Collapse to drawer">⤡ Collapse</button>
  </div>
  <div class="controls">
    <div class="searchbox">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
      <input placeholder="Search nickname or species…" bind:value={search} spellcheck="false" />
    </div>
    <label class="sort">
      <span>Sort</span>
      <select bind:value={sort}>
        <option value="slot">Box order</option>
        <option value="name">Name</option>
        <option value="level">Level</option>
      </select>
    </label>
  </div>
  <div class="filters"><SpeciesFilter {filter} showSearch={false} collapsible /></div>
  <div class="group-controls">
    <GroupFilter bind:selected={selectedGroups} />
    {#if box.pal}
      <div class="selected-groups">
        <span>EDIT SELECTED</span>
        <GroupTags instanceId={box.pal.instanceId} />
      </div>
    {/if}
  </div>
  <div class="grid">
    {#each filtered as p (p.slot)}
      <BoxTile pal={p} size="lg" selected={isSelected(p.slot)} onselect={select} />
    {/each}
    {#if !filtered.length}<div class="empty">{source.length ? "No Pals match these filters." : "No Pals to display."}</div>{/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: var(--topbar-h);
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 70;
    background: linear-gradient(155deg, rgba(24, 17, 32, 0.98), rgba(15, 13, 22, 0.99));
    backdrop-filter: blur(24px);
    display: flex;
    flex-direction: column;
  }
  .head { display: flex; align-items: center; gap: 11px; padding: 16px 30px; border-bottom: 1px solid rgba(176, 96, 224, 0.22); }
  .diamond { width: 11px; height: 11px; transform: rotate(45deg); background: var(--accent-purple); box-shadow: 0 0 8px var(--accent-purple); }
  h2 { margin: 0; font-family: var(--font-head); font-weight: 700; font-size: 20px; letter-spacing: 0.14em; color: #e7daf4; }
  .count { color: #9782a8; font-size: var(--type-body); }
  .collapse {
    margin-left: auto;
    padding: 9px 15px;
    border-radius: 9px;
    border: 1px solid rgba(176, 96, 224, 0.4);
    background: rgba(176, 96, 224, 0.12);
    color: #d6bef2;
    cursor: pointer;
    font-size: var(--type-body);
  }
  .collapse:hover { background: rgba(176, 96, 224, 0.22); }
  .controls { display: flex; align-items: center; gap: 12px; padding: 12px 30px 0; }
  .searchbox {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    max-width: 520px;
    padding: 9px 12px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.045);
    border: 1px solid rgba(255, 255, 255, 0.09);
  }
  .searchbox input { flex: 1; min-width: 0; color: #e6dfee; background: transparent; border: 0; outline: 0; font-size: var(--type-control); }
  .sort { display: flex; align-items: center; gap: 7px; color: #95889f; font-size: var(--type-caption); }
  .sort select { min-height: var(--control-min); padding: 8px 10px; border-radius: 8px; color: #c9bdd4; background: #1b1722; border: 1px solid rgba(255, 255, 255, 0.11); font-size: var(--type-body); }
  .filters { padding: 10px 30px 0; }
  .group-controls { display: flex; align-items: center; gap: 18px; padding: 10px 30px 0; }
  .group-controls :global(.group-filter) { flex: 1; }
  .selected-groups { min-width: 280px; display: flex; align-items: center; gap: 9px; }
  .selected-groups > span { flex: none; color: #8f819b; font: 600 var(--type-micro) var(--font-head); letter-spacing: .08em; }
  .grid {
    flex: 1;
    overflow: auto;
    padding: 24px 30px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(325px, 1fr));
    grid-auto-rows: max-content;
    gap: 14px;
    align-content: start;
    align-items: stretch;
  }
  .empty { grid-column: 1 / -1; padding: 36px; text-align: center; color: #7f718c; }
</style>
