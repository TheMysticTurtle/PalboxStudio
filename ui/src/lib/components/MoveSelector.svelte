<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { elementColor, normalizeElement } from "$lib/data/palPresentation";
  import { ref, resolveSpecies } from "$lib/data/refdata.svelte";
  import { toggleIn } from "$lib/data/speciesFilter.svelte";
  import ElementIcon from "./ElementIcon.svelte";

  let {
    open = $bindable(false),
    species,
    equipped,
    onpick,
  }: {
    open?: boolean;
    species: string;
    equipped: string[];
    onpick: (code: string) => void;
  } = $props();

  let search = $state("");
  let scope = $state<"natural" | "fruit" | "all">("natural");
  let elements = $state(new Set<ElementName>());
  let category = $state("All");
  let sort = $state<"power" | "name">("power");

  let elementOptions = $derived(
    Object.entries(ref.elements)
      .sort(([, left], [, right]) => left.sortOrder - right.sortOrder)
      .map(([code]) => code as ElementName),
  );
  let categories = $derived([
    "All",
    ...Array.from(new Set(Object.values(ref.moves).map((m) => m.category).filter(Boolean))).sort(),
  ]);
  let rows = $derived.by(() => {
    const natural = new Set(resolveSpecies(species)?.moves ?? []);
    const q = search.trim().toLowerCase();
    return Object.entries(ref.moves)
      .filter(([code, move]) => {
        if (scope === "natural" && !natural.has(code)) return false;
        if (scope === "fruit" && !move.skillFruit) return false;
        if (elements.size && (!move.element || !elements.has(move.element as ElementName))) return false;
        if (category !== "All" && move.category !== category) return false;
        return !q || code.toLowerCase().includes(q) || move.name.toLowerCase().includes(q);
      })
      .sort(([ac, a], [bc, b]) => {
        if (sort === "power" && a.power !== b.power) return b.power - a.power;
        return a.name.localeCompare(b.name) || ac.localeCompare(bc);
      })
      .map(([code, move]) => ({
        code,
        move,
        equipped: equipped.includes(code),
        natural: natural.has(code),
      }));
  });

  function choose(code: string) {
    onpick(code);
    open = false;
  }
  function clear() {
    search = "";
    scope = "natural";
    elements = new Set();
    category = "All";
    sort = "power";
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") open = false;
  }
</script>

<svelte:window onkeydown={open ? onKey : undefined} />

{#if open}
  <div class="scrim" role="presentation" onclick={() => (open = false)}></div>
  <div class="panel" role="dialog" aria-label="Select Pal move" aria-modal="true">
    <header>
      <span class="diamond"></span>
      <div>
        <h2>SELECT PAL MOVE</h2>
        <p>Filtering the reference data already loaded in memory</p>
      </div>
      <span class="count">{rows.length} / {Object.keys(ref.moves).length}</span>
      <button class="x" onclick={() => (open = false)} aria-label="Close">✕</button>
    </header>

    <div class="filters">
      <div class="row">
        <div class="searchbox">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
          <input placeholder="Search move or code…" bind:value={search} spellcheck="false" />
        </div>
        <button class="clear" onclick={clear}>Clear filters</button>
      </div>
      <div class="facets">
        <span class="label">Pool</span>
        <button class:on={scope === "natural"} onclick={() => (scope = "natural")}>Default moves</button>
        <button class:on={scope === "fruit"} onclick={() => (scope = "fruit")}>Skill Fruit</button>
        <button class:on={scope === "all"} onclick={() => (scope = "all")}>All moves</button>
        <span class="divider"></span>
        <span class="label">Category</span>
        <select bind:value={category}>{#each categories as c}<option value={c}>{c}</option>{/each}</select>
        <span class="divider"></span>
        <span class="label">Sort</span>
        <button class:on={sort === "power"} onclick={() => (sort = "power")}>Power</button>
        <button class:on={sort === "name"} onclick={() => (sort = "name")}>Name</button>
      </div>
      <div class="elements">
        <span class="label">Element</span>
        {#each elementOptions as element}
          <button
            class:on={elements.has(element)}
            style="--c:{elementColor(element)}"
            title={element}
            aria-label={element}
            aria-pressed={elements.has(element)}
            onclick={() => (elements = toggleIn(elements, element))}
          ><ElementIcon {element} size={24} muted={!elements.has(element)} /></button>
        {/each}
      </div>
    </div>

    <div class="list">
      {#each rows as row (row.code)}
        <button
          class="move"
          class:equipped={row.equipped}
          disabled={row.equipped}
          style="--c:{elementColor(normalizeElement(row.move.element))}"
          onclick={() => choose(row.code)}
        >
          <ElementIcon element={normalizeElement(row.move.element)} size={22} />
          <span class="info">
            <span class="name">{row.move.name}</span>
            <span class="meta">
              {row.move.element || "Neutral"} · {row.move.category || "Move"}
              {#if row.natural} · Default{:else if row.move.skillFruit} · Skill Fruit{/if}
            </span>
          </span>
          <span class="pcap">PWR</span>
          <span class="power">{row.move.power}</span>
          {#if row.equipped}<span class="tag">Equipped</span>{/if}
        </button>
      {/each}
      {#if !rows.length}<div class="empty">No Pal moves match these filters.</div>{/if}
    </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; z-index: 94; background: rgba(10,8,15,.66); backdrop-filter: blur(4px); }
  .panel {
    position: fixed; z-index: 95; top: 50%; left: 50%; transform: translate(-50%, -50%);
    width: min(980px, 95vw); height: min(84vh, 860px); display: flex; flex-direction: column;
    border-radius: 16px; overflow: hidden; border: 1px solid rgba(63,199,224,.32);
    background: linear-gradient(155deg, rgba(22,31,40,.99), rgba(14,18,24,.99));
    box-shadow: 0 30px 90px rgba(0,0,0,.62), 0 0 48px rgba(63,199,224,.12);
  }
  header { display: flex; align-items: center; gap: 12px; padding: 16px 20px; border-bottom: 1px solid rgba(63,199,224,.18); }
  .diamond { width: 11px; height: 11px; flex: none; transform: rotate(45deg); background: var(--accent-cyan); box-shadow: 0 0 8px var(--accent-cyan); }
  h2 { margin: 0; font: 700 20px var(--font-head); letter-spacing: .13em; color: #eafbff; }
  header p { margin: 3px 0 0; color: #8797a4; font-size: var(--type-caption); }
  .count { margin-left: auto; color: #8797a4; font-size: var(--type-body); font-variant-numeric: tabular-nums; }
  .x { width: 36px; height: 36px; border-radius: 9px; cursor: pointer; color: #b0bbc4; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.12); font-size: 16px; }
  .filters { padding: 13px 18px; display: flex; flex-direction: column; gap: 9px; border-bottom: 1px solid rgba(255,255,255,.07); }
  .row, .facets, .elements { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .searchbox { flex: 1; min-height: 40px; display: flex; align-items: center; gap: 9px; padding: 9px 13px; border-radius: 9px; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.09); }
  .searchbox input { flex: 1; min-width: 0; color: #e7edf2; background: transparent; border: 0; outline: 0; font-size: var(--type-control); }
  .clear { min-height: var(--control-min); color: #9ba8b0; background: transparent; border: 0; cursor: pointer; font-size: var(--type-caption); }
  .label { color: #8797a4; font: 600 var(--type-label) var(--font-head); letter-spacing: .08em; text-transform: uppercase; }
  .facets button {
    min-height: var(--control-min); padding: 6px 11px; color: #aeb9c2; background: rgba(255,255,255,.03);
    border: 1px solid rgba(255,255,255,.1); border-radius: 17px; cursor: pointer; font-size: var(--type-caption);
  }
  .facets button.on { color: #bff3fb; border-color: rgba(63,199,224,.5); background: rgba(63,199,224,.13); }
  .divider { width: 1px; height: 22px; margin: 0 4px; background: rgba(255,255,255,.09); }
  select { min-height: var(--control-min); color: #bdc7ce; background: #1c2630; border: 1px solid rgba(255,255,255,.12); border-radius: 8px; padding: 6px 9px; font-size: var(--type-caption); }
  .elements button { width: 36px; height: 36px; display: grid; place-items: center; cursor: pointer; border-radius: 8px; background: rgba(255,255,255,.03); border: 1px solid rgba(255,255,255,.1); }
  .elements button.on { border-color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); }
  .list { flex: 1; min-height: 0; overflow: auto; padding: 16px 20px 22px; display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: 10px; align-content: start; }
  .move {
    min-width: 0; display: flex; align-items: center; gap: 11px; padding: 12px 13px; cursor: pointer;
    color: #e9eef1; text-align: left; border-radius: 9px; background: rgba(255,255,255,.025);
    border: 1px solid rgba(255,255,255,.09);
  }
  .move:hover:not(:disabled) { border-color: color-mix(in srgb, var(--c) 60%, transparent); background: color-mix(in srgb, var(--c) 7%, transparent); }
  .move.equipped { opacity: .48; cursor: default; }
  .info { min-width: 0; flex: 1; display: flex; flex-direction: column; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font: 600 17px var(--font-cond); }
  .meta { color: #93a0a9; font-size: var(--type-caption); }
  .pcap { color: #8797a4; font-size: var(--type-micro); }
  .power { min-width: 32px; text-align: right; color: #d5dde2; font: 700 19px var(--font-head); }
  .tag { color: #aeb8bf; padding: 4px 7px; border-radius: 8px; background: rgba(255,255,255,.06); font-size: var(--type-micro); }
  .empty { grid-column: 1 / -1; padding: 44px; text-align: center; color: var(--text-muted); }
  @media (max-width: 680px) { .list { grid-template-columns: 1fr; } }
</style>
