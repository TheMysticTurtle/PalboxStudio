<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { resolveSpecies, speciesDisplayName } from "$lib/data/refdata.svelte";
  import { ui } from "$lib/stores/ui.svelte";
  import { box, openBoxFile, selectSlot, addPal, clonePal, deletePal } from "$lib/stores/box.svelte";
  import { palToBoxPal, tileDtoToBoxPal } from "$lib/data/mapper";
  import BoxTile from "./BoxTile.svelte";
  import SpeciesFilter from "./SpeciesFilter.svelte";
  import {
    createSpeciesFilter,
    speciesMatches,
    activeFilterCount,
  } from "$lib/data/speciesFilter.svelte";

  let search = $state("");
  // The shared species filter (elements / work / rideable / ranch / category),
  // applied to each tile's species row — same control as the species selector.
  const boxFilter = createSpeciesFilter();

  // Real box tiles joined to the cached species reference data.
  let source: BoxPal[] = $derived(
    box.open
      ? box.tiles.map((tile) =>
          tile.slot === box.selectedSlot && box.pal
            ? palToBoxPal(box.pal, tile.slot)
            : tileDtoToBoxPal(tile),
        )
      : [],
  );

  let filtered = $derived(
    source.filter((p) => {
      const query = search.trim().toLowerCase();
      if (
        query
        && !`${p.name} ${p.speciesName} ${p.nickname}`.toLowerCase().includes(query)
      ) return false;
      // Structured facets run against the tile's species reference row.
      const sp = resolveSpecies(p.species);
      if (sp) return speciesMatches(sp, boxFilter);
      return activeFilterCount(boxFilter) === 0; // no species row -> only the plain facets pass
    }),
  );

  async function openBoxClicked() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { localDataDir, join } = await import("@tauri-apps/api/path");
      const base = await join(await localDataDir(), "Pal", "Saved", "SaveGames");
      const file = await open({
        title: "Open GlobalPalStorage.sav",
        defaultPath: base,
        multiple: false,
        filters: [{ name: "Palworld Save", extensions: ["sav"] }],
      });
      if (typeof file === "string") await openBoxFile(file);
    } catch (e) {
      console.warn("File dialog is only available inside the app", e);
    }
  }

  function select(slot: number) {
    if (box.open) selectSlot(slot);
  }
  const isSelected = (slot: number) => box.open && box.selectedSlot === slot;

  // Reveal the selected tile (e.g. a just-added pal that landed off-screen).
  let matrixEl: HTMLDivElement | undefined = $state();
  $effect(() => {
    const s = box.selectedSlot;
    if (s < 0 || !matrixEl) return;
    requestAnimationFrame(() =>
      matrixEl?.querySelector(".tile.selected")?.scrollIntoView({ block: "nearest" }),
    );
  });

  async function onDelete() {
    if (!box.open || box.selectedSlot < 0) return;
    const t = box.tiles.find((x) => x.slot === box.selectedSlot);
    const name = t
      ? speciesDisplayName(t.characterId)
      : "this pal";
    try {
      const { ask } = await import("@tauri-apps/plugin-dialog");
      const ok = await ask(
        `Remove ${name} from the Global Palbox?\n\nTakes effect when you save; a backup is kept.`,
        { title: "Delete Pal", kind: "warning" },
      );
      if (!ok) return;
    } catch {
      // Dialog is app-only; edits are in-memory + backed up on save, so proceed.
    }
    await deletePal(box.selectedSlot);
  }
</script>

<div class="box">
  <button class="open" onclick={openBoxClicked} disabled={box.loading}>
    {box.loading ? "Opening…" : "⭳ Open Global Palbox"}
  </button>
  {#if box.open}
    <div class="picked">{box.tiles.length} pals · {box.slotCount} slots · <b>{box.path.split(/[\\/]/).pop()}</b></div>
  {/if}
  {#if box.error}<div class="err">{box.error}</div>{/if}

  <div class="controls">
    <div class="searchbox">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
      <input placeholder="Search pals…" bind:value={search} />
    </div>
    <span class="count">{filtered.length}</span>
    <button class="expand" onclick={() => (ui.boxExpanded = true)} title="Expand to full matrix">⤢ Expand</button>
  </div>

  <SpeciesFilter filter={boxFilter} showSearch={false} collapsible />

  <div class="matrix" bind:this={matrixEl}>
    {#each filtered as p (p.slot)}
      <BoxTile pal={p} size="sm" selected={isSelected(p.slot)} onselect={select} />
    {/each}
    {#if !filtered.length}<div class="empty">{box.open ? "No pals match." : "Open a box to load your pals."}</div>{/if}
  </div>

  {#if box.hint}<div class="hint">{box.hint}</div>{/if}

  <div class="footer">
    <button class="fa add" onclick={() => addPal()} disabled={!box.open} title="Add a new turtle to a free slot">+ Add</button>
    <button class="fa" onclick={() => clonePal(box.selectedSlot)} disabled={!box.open || box.selectedSlot < 0} title="Clone the selected pal">⧉ Clone</button>
    <button class="fa del" onclick={onDelete} disabled={!box.open || box.selectedSlot < 0} title="Remove the selected pal">🗑 Delete</button>
  </div>
</div>

<style>
  .box { height: 100%; display: flex; flex-direction: column; gap: 11px; }
  .open {
    padding: 11px; border-radius: 10px; cursor: pointer;
    font-family: var(--font-head); font-weight: 600; font-size: 14px; letter-spacing: 0.06em;
    color: #eafbff; border: 1px solid rgba(63, 199, 224, 0.45); background: rgba(63, 199, 224, 0.14);
  }
  .open:hover { background: rgba(63, 199, 224, 0.24); }
  .open:disabled { opacity: 0.6; cursor: default; }
  .picked { font-size: var(--type-caption); color: #a18caf; }
  .picked b { color: #c9b4e0; }
  .err { font-size: var(--type-caption); color: #e89090; }

  .controls { display: flex; align-items: center; gap: 8px; }
  .searchbox { flex: 1; display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: 9px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.09); }
  .searchbox input { flex: 1; min-width: 0; background: transparent; border: 0; outline: none; color: #e7daf4; font-size: var(--type-control); }
  .count { font-family: var(--font-head); font-weight: 700; color: #a18caf; font-size: var(--type-body); min-width: 20px; text-align: center; }
  .expand { min-height: var(--control-min); padding: 8px 12px; border-radius: 9px; border: 1px solid rgba(176, 96, 224, 0.4); background: rgba(176, 96, 224, 0.12); color: #d6bef2; cursor: pointer; font-size: var(--type-caption); white-space: nowrap; }
  .expand:hover { background: rgba(176, 96, 224, 0.22); }

  .matrix {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    grid-auto-rows: max-content;
    gap: 10px;
    align-content: start;
    align-items: start;
    padding-right: 4px;
  }
  .empty { grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 24px; font-size: var(--type-body); }

  .hint { font-size: var(--type-caption); line-height: 1.4; color: #c9b98a; padding: 9px 11px; border-radius: 8px; background: rgba(245, 166, 35, 0.1); border: 1px solid rgba(245, 166, 35, 0.28); }

  .footer { display: flex; gap: 9px; padding-top: 4px; }
  .fa { flex: 1; padding: 11px; border-radius: 9px; cursor: pointer; font-family: var(--font-cond); font-weight: 600; font-size: 14px; color: #cbd3db; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.12); }
  .fa.add { color: #9fd8e6; border-color: rgba(63, 199, 224, 0.4); background: rgba(63, 199, 224, 0.12); }
  .fa.del { color: #e89090; border-color: rgba(224, 90, 90, 0.35); background: rgba(224, 90, 90, 0.1); }
  .fa:hover:not(:disabled) { filter: brightness(1.15); }
  .fa:disabled { opacity: 0.4; cursor: default; }
</style>
