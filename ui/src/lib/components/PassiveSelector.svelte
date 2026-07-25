<script lang="ts">
  import { ref, resolveSpecies } from "$lib/data/refdata.svelte";
  import { ratingTone } from "$lib/data/constants";
  import {
    PASSIVE_GROUPS,
    passiveMatches,
    type PassiveGroup,
    type PassiveScope,
    type PassiveSort,
    type PassiveTone,
  } from "$lib/data/passiveFilter";

  let {
    open = $bindable(false),
    species,
    selected,
    editing = null,
    onpick,
    onremove,
  }: {
    open?: boolean;
    species: string;
    selected: string[];
    editing?: number | null;
    onpick: (code: string) => void;
    onremove?: () => void;
  } = $props();

  let search = $state("");
  let scope = $state<PassiveScope>("species");
  let tone = $state<PassiveTone>("all");
  let group = $state<PassiveGroup | "all">("all");
  let sort = $state<PassiveSort>("rating");
  let includeDisabled = $state(false);

  const current = $derived(editing == null ? "" : (selected[editing] ?? ""));
  let speciesPassives = $derived(new Set(resolveSpecies(species)?.passives ?? []));
  let rows = $derived.by(() => {
    const used = new Set(selected.filter((_, i) => i !== editing));
    return Object.entries(ref.passives)
      .filter(([code, passive]) =>
        passiveMatches(code, passive, search, scope, tone, group, includeDisabled, speciesPassives),
      )
      .sort(([ac, a], [bc, b]) => {
        if (sort === "rating" && a.rating !== b.rating) return b.rating - a.rating;
        return a.name.localeCompare(b.name) || ac.localeCompare(bc);
      })
      .map(([code, passive]) => ({ code, passive, used: used.has(code) }));
  });

  function choose(code: string) {
    onpick(code);
    open = false;
  }
  function remove() {
    onremove?.();
    open = false;
  }
  function clear() {
    search = "";
    scope = "species";
    tone = "all";
    group = "all";
    sort = "rating";
    includeDisabled = false;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") open = false;
  }
</script>

<svelte:window onkeydown={open ? onKey : undefined} />

{#if open}
  <div class="scrim" role="presentation" onclick={() => (open = false)}></div>
  <div class="panel" role="dialog" aria-label="Select passive skill" aria-modal="true">
    <header>
      <span class="diamond"></span>
      <div>
        <h2>{editing == null ? "ADD PASSIVE SKILL" : "CHANGE PASSIVE SKILL"}</h2>
        <p>Filtering the reference data already loaded in memory</p>
      </div>
      <span class="count">{rows.length} / {Object.keys(ref.passives).length}</span>
      {#if editing != null}<button class="remove" onclick={remove}>Remove current</button>{/if}
      <button class="x" onclick={() => (open = false)} aria-label="Close">✕</button>
    </header>

    <div class="filters">
      <div class="searchbox">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
        <input placeholder="Search name, effect, or code…" bind:value={search} spellcheck="false" />
      </div>
      <button class="clear" onclick={clear}>Clear filters</button>

      <div class="facets">
        <div class="facet">
          <span>Pool</span>
          {#each [["species", "This species"], ["normal", "Normal pool"], ["lucky", "Lucky pool"], ["all", "All"]] as option}
            <button class:on={scope === option[0]} onclick={() => (scope = option[0] as PassiveScope)}>{option[1]}</button>
          {/each}
        </div>
        <div class="facet">
          <span>Rating</span>
          {#each [["all", "All"], ["positive", "Positive"], ["negative", "Negative"]] as option}
            <button class:on={tone === option[0]} onclick={() => (tone = option[0] as PassiveTone)}>{option[1]}</button>
          {/each}
        </div>
        <div class="facet">
          <span>Sort</span>
          <button class:on={sort === "rating"} onclick={() => (sort = "rating")}>Rating</button>
          <button class:on={sort === "name"} onclick={() => (sort = "name")}>Name</button>
        </div>
        <label class="disabled"><input type="checkbox" bind:checked={includeDisabled} /> Include disabled</label>
      </div>

      <div class="groups">
        <button class:on={group === "all"} onclick={() => (group = "all")}>All effects</button>
        {#each PASSIVE_GROUPS as value}
          <button class:on={group === value} onclick={() => (group = value)}>{value}</button>
        {/each}
      </div>
    </div>

    <div class="list">
      {#each rows as row (row.code)}
        <button
          class="passive"
          class:current={row.code === current}
          class:used={row.used}
          disabled={row.used}
          style="--c:{ratingTone(row.passive.rating)}"
          onclick={() => choose(row.code)}
        >
          <span class="rank">{row.passive.rating > 0 ? "+" : ""}{row.passive.rating}</span>
          <span class="info">
            <span class="name">{row.passive.name}</span>
            <span class="desc">{row.passive.description || "No description available."}</span>
          </span>
          {#if row.passive.disabled}<span class="tag warn">Disabled</span>{/if}
          {#if row.used}<span class="tag">Equipped</span>{/if}
        </button>
      {/each}
      {#if !rows.length}<div class="empty">No passive skills match these filters.</div>{/if}
    </div>
  </div>
{/if}

<style>
  .scrim { position: fixed; inset: 0; z-index: 94; background: rgba(10, 8, 15, 0.66); backdrop-filter: blur(4px); }
  .panel {
    position: fixed; z-index: 95; top: 50%; left: 50%; transform: translate(-50%, -50%);
    width: min(900px, 92vw); height: min(80vh, 820px); display: flex; flex-direction: column;
    border-radius: 16px; overflow: hidden; border: 1px solid rgba(63, 199, 224, 0.32);
    background: linear-gradient(155deg, rgba(22, 31, 40, 0.99), rgba(14, 18, 24, 0.99));
    box-shadow: 0 30px 90px rgba(0, 0, 0, 0.62), 0 0 48px rgba(63, 199, 224, 0.12);
  }
  header { display: flex; align-items: center; gap: 12px; padding: 14px 18px; border-bottom: 1px solid rgba(63, 199, 224, 0.18); }
  .diamond { width: 10px; height: 10px; flex: none; transform: rotate(45deg); background: var(--accent-cyan); box-shadow: 0 0 8px var(--accent-cyan); }
  h2 { margin: 0; font: 700 17px var(--font-head); letter-spacing: 0.13em; color: #eafbff; }
  header p { margin: 2px 0 0; color: #71818d; font-size: 11.5px; }
  .count { margin-left: auto; color: #8797a4; font-size: 12px; font-variant-numeric: tabular-nums; }
  .remove { color: #e9a1a1; background: rgba(224, 90, 90, 0.1); border: 1px solid rgba(224, 90, 90, 0.32); border-radius: 8px; padding: 7px 10px; cursor: pointer; }
  .x { width: 30px; height: 30px; border-radius: 8px; cursor: pointer; color: #b0bbc4; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.12); }

  .filters { padding: 13px 18px; display: flex; flex-wrap: wrap; gap: 9px; border-bottom: 1px solid rgba(255,255,255,.07); }
  .searchbox { flex: 1 1 420px; display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: 9px; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.09); }
  .searchbox input { flex: 1; min-width: 0; color: #e7edf2; background: transparent; border: 0; outline: 0; font-size: 14px; }
  .clear { color: #8e9ba5; background: transparent; border: 0; cursor: pointer; }
  .facets, .groups { flex: 1 1 100%; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .facet { display: flex; align-items: center; gap: 4px; padding-right: 9px; border-right: 1px solid rgba(255,255,255,.08); }
  .facet > span { margin-right: 3px; color: #71818d; font: 600 10px var(--font-head); letter-spacing: .08em; text-transform: uppercase; }
  .facet button, .groups button {
    padding: 5px 9px; color: #aeb9c2; background: rgba(255,255,255,.03);
    border: 1px solid rgba(255,255,255,.1); border-radius: 13px; cursor: pointer; font-size: 11.5px;
  }
  .facet button.on, .groups button.on { color: #bff3fb; border-color: rgba(63,199,224,.5); background: rgba(63,199,224,.13); }
  .disabled { margin-left: auto; display: flex; align-items: center; gap: 6px; color: #82919c; font-size: 11.5px; }
  .groups button.on { color: #e0ccf5; border-color: rgba(176,96,224,.5); background: rgba(176,96,224,.14); }

  .list { flex: 1; min-height: 0; overflow: auto; padding: 14px 18px 20px; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 9px; align-content: start; }
  .passive {
    min-width: 0; display: flex; align-items: center; gap: 11px; padding: 11px 12px;
    color: #edf2f4; text-align: left; cursor: pointer; border-radius: 10px;
    background: linear-gradient(90deg, color-mix(in srgb, var(--c) 11%, transparent), rgba(255,255,255,.025));
    border: 1px solid color-mix(in srgb, var(--c) 30%, transparent); border-left: 3px solid var(--c);
  }
  .passive:hover:not(:disabled) { border-color: color-mix(in srgb, var(--c) 68%, transparent); background-color: rgba(255,255,255,.035); }
  .passive.current { box-shadow: 0 0 12px color-mix(in srgb, var(--c) 28%, transparent); }
  .passive.used { opacity: .48; cursor: default; }
  .rank { flex: none; min-width: 29px; color: var(--c); font: 700 17px var(--font-head); text-align: center; }
  .info { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .name { font: 600 14px var(--font-cond); color: #edf2f4; }
  .desc { color: #9ba8b0; font-size: 11.5px; line-height: 1.25; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; line-clamp: 2; overflow: hidden; }
  .tag { flex: none; padding: 3px 6px; border-radius: 8px; color: #aab5bd; background: rgba(255,255,255,.06); font-size: 9.5px; }
  .tag.warn { color: #e8a0a0; background: rgba(224,90,90,.12); }
  .empty { grid-column: 1 / -1; padding: 44px; text-align: center; color: var(--text-muted); }
  @media (max-width: 720px) { .list { grid-template-columns: 1fr; } }
</style>
