<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { ELEMENT_COLOR } from "$lib/data/constants";
  import { ref, resolveSpecies } from "$lib/data/refdata.svelte";
  import { toggleIn } from "$lib/data/speciesFilter.svelte";

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
  let scope = $state<"species" | "all">("species");
  let elements = $state(new Set<ElementName>());
  let category = $state("All");
  let sort = $state<"power" | "name">("power");

  const elementOptions: ElementName[] = [
    "Neutral", "Fire", "Water", "Grass", "Electric", "Ice", "Ground", "Dark", "Dragon",
  ];
  let categories = $derived([
    "All",
    ...Array.from(new Set(Object.values(ref.moves).map((m) => m.category).filter(Boolean))).sort(),
  ]);
  let rows = $derived.by(() => {
    const natural = new Set(resolveSpecies(species)?.moves ?? []);
    const q = search.trim().toLowerCase();
    return Object.entries(ref.moves)
      .filter(([code, move]) => {
        if (scope === "species" && !natural.has(code)) return false;
        if (elements.size && (!move.element || !elements.has(move.element as ElementName))) return false;
        if (category !== "All" && move.category !== category) return false;
        return !q || code.toLowerCase().includes(q) || move.name.toLowerCase().includes(q);
      })
      .sort(([ac, a], [bc, b]) => {
        if (sort === "power" && a.power !== b.power) return b.power - a.power;
        return a.name.localeCompare(b.name) || ac.localeCompare(bc);
      })
      .map(([code, move]) => ({ code, move, equipped: equipped.includes(code) }));
  });

  function choose(code: string) {
    onpick(code);
    open = false;
  }
  function clear() {
    search = "";
    scope = "species";
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
        <button class:on={scope === "species"} onclick={() => (scope = "species")}>This species</button>
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
            style="--c:{ELEMENT_COLOR[element]}"
            title={element}
            aria-label={element}
            aria-pressed={elements.has(element)}
            onclick={() => (elements = toggleIn(elements, element))}
          ><span></span></button>
        {/each}
      </div>
    </div>

    <div class="list">
      {#each rows as row (row.code)}
        <button
          class="move"
          class:equipped={row.equipped}
          disabled={row.equipped}
          style="--c:{ELEMENT_COLOR[row.move.element as ElementName] ?? 'var(--el-neutral)'}"
          onclick={() => choose(row.code)}
        >
          <span class="edia"></span>
          <span class="info">
            <span class="name">{row.move.name}</span>
            <span class="meta">{row.move.element || "Neutral"} · {row.move.category || "Move"}</span>
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
    width: min(820px, 92vw); height: min(78vh, 790px); display: flex; flex-direction: column;
    border-radius: 16px; overflow: hidden; border: 1px solid rgba(63,199,224,.32);
    background: linear-gradient(155deg, rgba(22,31,40,.99), rgba(14,18,24,.99));
    box-shadow: 0 30px 90px rgba(0,0,0,.62), 0 0 48px rgba(63,199,224,.12);
  }
  header { display: flex; align-items: center; gap: 12px; padding: 14px 18px; border-bottom: 1px solid rgba(63,199,224,.18); }
  .diamond { width: 10px; height: 10px; flex: none; transform: rotate(45deg); background: var(--accent-cyan); box-shadow: 0 0 8px var(--accent-cyan); }
  h2 { margin: 0; font: 700 17px var(--font-head); letter-spacing: .13em; color: #eafbff; }
  header p { margin: 2px 0 0; color: #71818d; font-size: 11.5px; }
  .count { margin-left: auto; color: #8797a4; font-size: 12px; font-variant-numeric: tabular-nums; }
  .x { width: 30px; height: 30px; border-radius: 8px; cursor: pointer; color: #b0bbc4; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.12); }
  .filters { padding: 13px 18px; display: flex; flex-direction: column; gap: 9px; border-bottom: 1px solid rgba(255,255,255,.07); }
  .row, .facets, .elements { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .searchbox { flex: 1; display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: 9px; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.09); }
  .searchbox input { flex: 1; min-width: 0; color: #e7edf2; background: transparent; border: 0; outline: 0; font-size: 14px; }
  .clear { color: #8e9ba5; background: transparent; border: 0; cursor: pointer; }
  .label { color: #71818d; font: 600 10px var(--font-head); letter-spacing: .08em; text-transform: uppercase; }
  .facets button {
    padding: 5px 9px; color: #aeb9c2; background: rgba(255,255,255,.03);
    border: 1px solid rgba(255,255,255,.1); border-radius: 13px; cursor: pointer; font-size: 11.5px;
  }
  .facets button.on { color: #bff3fb; border-color: rgba(63,199,224,.5); background: rgba(63,199,224,.13); }
  .divider { width: 1px; height: 22px; margin: 0 4px; background: rgba(255,255,255,.09); }
  select { color: #bdc7ce; background: #1c2630; border: 1px solid rgba(255,255,255,.12); border-radius: 7px; padding: 5px 8px; }
  .elements button { width: 27px; height: 27px; display: grid; place-items: center; cursor: pointer; border-radius: 7px; background: rgba(255,255,255,.03); border: 1px solid rgba(255,255,255,.1); }
  .elements button span { width: 10px; height: 10px; transform: rotate(45deg); background: var(--c); opacity: .48; }
  .elements button.on { border-color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); }
  .elements button.on span { opacity: 1; box-shadow: 0 0 6px var(--c); }
  .list { flex: 1; min-height: 0; overflow: auto; padding: 14px 18px 20px; display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: 8px; align-content: start; }
  .move {
    min-width: 0; display: flex; align-items: center; gap: 10px; padding: 10px 11px; cursor: pointer;
    color: #e9eef1; text-align: left; border-radius: 9px; background: rgba(255,255,255,.025);
    border: 1px solid rgba(255,255,255,.09);
  }
  .move:hover:not(:disabled) { border-color: color-mix(in srgb, var(--c) 60%, transparent); background: color-mix(in srgb, var(--c) 7%, transparent); }
  .move.equipped { opacity: .48; cursor: default; }
  .edia { width: 11px; height: 11px; flex: none; transform: rotate(45deg); background: var(--c); box-shadow: 0 0 6px var(--c); }
  .info { min-width: 0; flex: 1; display: flex; flex-direction: column; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font: 600 14px var(--font-cond); }
  .meta { color: #82919c; font-size: 10.5px; }
  .pcap { color: #71818d; font-size: 9.5px; }
  .power { min-width: 28px; text-align: right; color: #d5dde2; font: 700 16px var(--font-head); }
  .tag { color: #9ba8b0; padding: 3px 6px; border-radius: 8px; background: rgba(255,255,255,.06); font-size: 9.5px; }
  .empty { grid-column: 1 / -1; padding: 44px; text-align: center; color: var(--text-muted); }
  @media (max-width: 680px) { .list { grid-template-columns: 1fr; } }
</style>
