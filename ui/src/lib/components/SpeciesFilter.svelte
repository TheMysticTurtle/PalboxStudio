<script lang="ts">
  import type { ElementName, Category } from "$lib/data/types";
  import { ELEMENT_COLOR, WORK_SUITS } from "$lib/data/constants";
  import {
    type SpeciesFilterState,
    toggleIn,
    clearFilter,
    activeFilterCount,
    ranchDropOptions,
    CATEGORY_LABELS,
  } from "$lib/data/speciesFilter.svelte";
  import ElementIcon from "./ElementIcon.svelte";

  let {
    filter,
    showSearch = true,
    showCategories = true,
    showRanch = true,
    collapsible = false,
    placeholder = "Search species…",
  }: {
    filter: SpeciesFilterState;
    showSearch?: boolean;
    showCategories?: boolean;
    showRanch?: boolean;
    /** Hide the facet controls behind a "Filters" toggle (for cramped drawers). */
    collapsible?: boolean;
    placeholder?: string;
  } = $props();

  const elements: ElementName[] = [
    "Neutral", "Fire", "Water", "Grass", "Electric", "Ice", "Ground", "Dark", "Dragon",
  ];
  const ranchOptions = ranchDropOptions();
  let count = $derived(activeFilterCount(filter));
  // Non-search facets start collapsed when collapsible; otherwise always open.
  let userExpanded = $state(false);
  let expanded = $derived(!collapsible || userExpanded);
</script>

<div class="filter">
  {#if showSearch}
    <div class="row">
      <div class="searchbox">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
        <input {placeholder} bind:value={filter.search} spellcheck="false" />
      </div>
      {#if count}<button class="clear" onclick={() => clearFilter(filter)}>clear {count}</button>{/if}
    </div>
  {/if}

  {#if collapsible}
    <button class="ftoggle" class:active={count > 0} onclick={() => (userExpanded = !userExpanded)} aria-expanded={expanded}>
      <span class="chev" class:open={expanded}>▸</span>
      <span class="ftitle">Filters</span>
      {#if count}<span class="fn">{count}</span>{/if}
      {#if count}<span
        class="clearx"
        role="button"
        tabindex="0"
        onclick={(e) => { e.stopPropagation(); clearFilter(filter); }}
        onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.stopPropagation(); clearFilter(filter); } }}
      >clear</span>{/if}
    </button>
  {/if}

  {#if expanded}
  <!-- Elements -->
  <div class="grp">
    <span class="lbl">Element</span>
    <div class="els">
      {#each elements as el (el)}
        <button
          class="eltog" class:on={filter.elements.has(el)}
          style="--c:{ELEMENT_COLOR[el]}"
          onclick={() => (filter.elements = toggleIn(filter.elements, el))}
          title={el} aria-label={el} aria-pressed={filter.elements.has(el)}
        ><ElementIcon element={el} size={20} muted={!filter.elements.has(el)} /></button>
      {/each}
    </div>
  </div>

  <!-- Work suitability -->
  <div class="grp">
    <span class="lbl">Work</span>
    <div class="works">
      {#each WORK_SUITS as w (w.name)}
        <button
          class="worktog" class:on={filter.work.has(w.name)}
          onclick={() => (filter.work = toggleIn(filter.work, w.name))}
          title={w.name} aria-label={w.name} aria-pressed={filter.work.has(w.name)}
        ><img src={`/icons/work/${w.icon}.png`} alt="" /></button>
      {/each}
    </div>
  </div>

  <!-- Rideable + categories -->
  <div class="grp inline">
    <button
      class="pill ride" class:on={filter.rideable}
      onclick={() => (filter.rideable = !filter.rideable)}
      aria-pressed={filter.rideable}
    >🐎 Rideable mount</button>
    {#if showCategories}
      {#each CATEGORY_LABELS as c (c.value)}
        <button
          class="pill cat" class:on={filter.categories.has(c.value)}
          onclick={() => (filter.categories = toggleIn(filter.categories, c.value as Category))}
          aria-pressed={filter.categories.has(c.value)}
        >{c.label}</button>
      {/each}
    {/if}
  </div>

  <!-- Ranch / farm drops -->
  {#if showRanch && ranchOptions.length}
    <details class="ranch" open={filter.ranchDrops.size > 0}>
      <summary>Ranch drops {#if filter.ranchDrops.size}<span class="n">{filter.ranchDrops.size}</span>{/if}</summary>
      <div class="drops">
        {#each ranchOptions as item (item)}
          <button
            class="drop" class:on={filter.ranchDrops.has(item)}
            onclick={() => (filter.ranchDrops = toggleIn(filter.ranchDrops, item))}
            aria-pressed={filter.ranchDrops.has(item)}
          >{item}</button>
        {/each}
      </div>
    </details>
  {/if}
  {/if}
</div>

<style>
  .filter { display: flex; flex-direction: column; gap: 10px; }
  .row { display: flex; align-items: center; gap: 8px; }

  .ftoggle {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px; border-radius: 9px; cursor: pointer;
    background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1);
    color: #b0a0be; font-size: 13px; text-align: left;
  }
  .ftoggle:hover { border-color: rgba(255, 255, 255, 0.22); }
  .ftoggle.active { border-color: rgba(176, 96, 224, 0.4); background: rgba(176, 96, 224, 0.08); }
  .ftoggle .chev { color: #7f7090; transition: transform 0.15s; }
  .ftoggle .chev.open { transform: rotate(90deg); }
  .ftoggle .ftitle { font-family: var(--font-head); font-weight: 600; letter-spacing: 0.06em; color: #cbbfe0; }
  .ftoggle .fn { display: grid; place-items: center; min-width: 18px; height: 18px; padding: 0 5px; border-radius: 9px; background: rgba(176, 96, 224, 0.25); color: #e0ccf5; font-size: 11px; font-weight: 700; }
  .ftoggle .clearx { margin-left: auto; color: #8b7c99; font-size: 12px; }
  .ftoggle .clearx:hover { color: #d6bef2; }
  .searchbox { flex: 1; display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: 9px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.09); }
  .searchbox input { flex: 1; min-width: 0; background: transparent; border: 0; outline: none; color: #e7daf4; font-size: 14px; }
  .clear { background: none; border: 0; color: #8b7c99; cursor: pointer; font-size: 12px; white-space: nowrap; }
  .clear:hover { color: #c9b4e0; }

  .grp { display: flex; align-items: center; gap: 9px; flex-wrap: wrap; }
  .grp.inline { gap: 7px; }
  .lbl { font-family: var(--font-head); font-size: 10.5px; letter-spacing: 0.1em; color: #7f7090; text-transform: uppercase; min-width: 42px; }

  .els { display: flex; gap: 5px; flex-wrap: wrap; }
  .eltog { width: 29px; height: 29px; display: grid; place-items: center; border-radius: 7px; cursor: pointer; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); }
  .eltog.on { border-color: var(--c); background: color-mix(in srgb, var(--c) 18%, transparent); }

  .works { display: flex; gap: 4px; flex-wrap: wrap; }
  .worktog { width: 28px; height: 28px; display: grid; place-items: center; border-radius: 7px; cursor: pointer; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); }
  .worktog img { width: 18px; height: 18px; object-fit: contain; opacity: 0.5; filter: grayscale(0.4); }
  .worktog.on { border-color: rgba(245, 166, 35, 0.55); background: rgba(245, 166, 35, 0.14); }
  .worktog.on img { opacity: 1; filter: none; }

  .pill { padding: 6px 12px; border-radius: 15px; cursor: pointer; font-size: 12.5px; color: #b0a0be; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); white-space: nowrap; }
  .pill:hover { border-color: rgba(255, 255, 255, 0.24); }
  .pill.ride.on { color: #9fe6c0; background: rgba(63, 224, 150, 0.14); border-color: rgba(63, 224, 150, 0.5); }
  .pill.cat.on { color: #d6bef2; background: rgba(176, 96, 224, 0.16); border-color: rgba(176, 96, 224, 0.5); font-weight: 600; }

  .ranch { border: 1px solid rgba(255, 255, 255, 0.09); border-radius: 9px; background: rgba(255, 255, 255, 0.02); }
  .ranch summary { cursor: pointer; padding: 8px 12px; font-size: 12.5px; color: #b0a0be; user-select: none; list-style: none; display: flex; align-items: center; gap: 8px; }
  .ranch summary::-webkit-details-marker { display: none; }
  .ranch summary::before { content: "▸"; color: #7f7090; transition: transform 0.15s; }
  .ranch[open] summary::before { transform: rotate(90deg); }
  .ranch .n { display: grid; place-items: center; min-width: 17px; height: 17px; padding: 0 4px; border-radius: 9px; background: rgba(245, 166, 35, 0.2); color: #f5c97a; font-size: 11px; font-weight: 700; }
  .drops { display: flex; flex-wrap: wrap; gap: 6px; padding: 4px 12px 12px; max-height: 148px; overflow: auto; }
  .drop { padding: 5px 10px; border-radius: 8px; cursor: pointer; font-size: 11.5px; color: #b0a0be; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); }
  .drop.on { color: #f5c97a; background: rgba(245, 166, 35, 0.14); border-color: rgba(245, 166, 35, 0.5); }
</style>
