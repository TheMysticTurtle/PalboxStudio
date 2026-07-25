<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { ELEMENT_COLOR } from "$lib/data/constants";
  import { sampleBox } from "$lib/data/sampleBox";
  import { ui } from "$lib/stores/ui.svelte";
  import BoxTile from "./BoxTile.svelte";

  const elements: ElementName[] = [
    "Neutral", "Fire", "Water", "Grass", "Electric", "Ice", "Ground", "Dark", "Dragon",
  ];
  const groups = ["All", ...Array.from(new Set(sampleBox.flatMap((p) => p.groups ?? [])))];

  let search = $state("");
  let selectedEls = $state<Set<ElementName>>(new Set());
  let activeGroup = $state("All");
  let pickedFile = $state<string | null>(null);

  function toggleEl(el: ElementName) {
    const s = new Set(selectedEls);
    if (s.has(el)) s.delete(el);
    else s.add(el);
    selectedEls = s;
  }

  let filtered = $derived(
    sampleBox.filter((p) => {
      if (search && !p.name.toLowerCase().includes(search.toLowerCase())) return false;
      if (selectedEls.size && !p.elements.some((e) => selectedEls.has(e))) return false;
      if (activeGroup !== "All" && !(p.groups ?? []).includes(activeGroup)) return false;
      return true;
    }),
  );

  // Browse for a GlobalPalStorage.sav, defaulting to Palworld's usual save location.
  // (Loading the file is the engine's job — this just picks it.)
  async function openBox() {
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
      if (typeof file === "string") pickedFile = file;
    } catch (e) {
      console.warn("File dialog is only available inside the app", e);
    }
  }

  const select = (id: string) => (ui.selectedId = id);
</script>

<div class="box">
  <button class="open" onclick={openBox}>⭳ Open Global Palbox</button>
  {#if pickedFile}
    <div class="picked" title={pickedFile}>Selected: <b>{pickedFile.split(/[\\/]/).pop()}</b> — loading comes with the engine.</div>
  {/if}

  <div class="controls">
    <div class="searchbox">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="7" stroke="#9782A8" stroke-width="1.8" /><path d="m20 20-3.5-3.5" stroke="#9782A8" stroke-width="1.8" stroke-linecap="round" /></svg>
      <input placeholder="Search pals…" bind:value={search} />
    </div>
    <span class="count">{filtered.length}</span>
    <button class="expand" onclick={() => (ui.boxExpanded = true)} title="Expand to full matrix">⤢ Expand</button>
  </div>

  <div class="elfilter">
    {#each elements as el (el)}
      <button class="eltog" class:on={selectedEls.has(el)} style="--c:{ELEMENT_COLOR[el]}" onclick={() => toggleEl(el)} title={el} aria-label={el} aria-pressed={selectedEls.has(el)}><span class="d"></span></button>
    {/each}
    {#if selectedEls.size}<button class="clear" onclick={() => (selectedEls = new Set())}>clear</button>{/if}
  </div>

  <div class="groupsrow">
    {#each groups as g (g)}
      <button class="gchip" class:on={activeGroup === g} onclick={() => (activeGroup = g)}>{g}</button>
    {/each}
    <button class="gchip add" title="Create a group (coming soon)">+ Group</button>
  </div>

  <div class="matrix">
    {#each filtered as p (p.instanceId)}
      <BoxTile pal={p} size="sm" selected={ui.selectedId === p.instanceId} onselect={select} />
    {/each}
    {#if !filtered.length}<div class="empty">No pals match.</div>{/if}
  </div>

  <div class="footer">
    <button class="fa add">+ Add</button>
    <button class="fa">⧉ Clone</button>
    <button class="fa del">🗑 Delete</button>
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
  .picked { font-size: 11.5px; color: #9782a8; }
  .picked b { color: #c9b4e0; }

  .controls { display: flex; align-items: center; gap: 8px; }
  .searchbox { flex: 1; display: flex; align-items: center; gap: 8px; padding: 9px 12px; border-radius: 9px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.09); }
  .searchbox input { flex: 1; min-width: 0; background: transparent; border: 0; outline: none; color: #e7daf4; font-size: 14px; }
  .count { font-family: var(--font-head); font-weight: 700; color: #9782a8; font-size: 13px; min-width: 18px; text-align: center; }
  .expand { padding: 8px 11px; border-radius: 9px; border: 1px solid rgba(176, 96, 224, 0.4); background: rgba(176, 96, 224, 0.12); color: #d6bef2; cursor: pointer; font-size: 12.5px; white-space: nowrap; }
  .expand:hover { background: rgba(176, 96, 224, 0.22); }

  .elfilter { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
  .eltog { width: 26px; height: 26px; display: grid; place-items: center; border-radius: 7px; cursor: pointer; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); }
  .eltog .d { width: 11px; height: 11px; transform: rotate(45deg); background: var(--c); opacity: 0.55; }
  .eltog.on { border-color: var(--c); background: color-mix(in srgb, var(--c) 18%, transparent); }
  .eltog.on .d { opacity: 1; box-shadow: 0 0 6px var(--c); }
  .clear { background: none; border: 0; color: #8b7c99; cursor: pointer; font-size: 12px; }

  .groupsrow { display: flex; align-items: center; gap: 7px; flex-wrap: wrap; }
  .gchip { padding: 6px 13px; border-radius: 16px; cursor: pointer; font-size: 12.5px; color: #b0a0be; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.1); }
  .gchip.on { color: #f5c97a; background: rgba(245, 166, 35, 0.16); border-color: rgba(245, 166, 35, 0.5); font-weight: 600; }
  .gchip.add { color: #8b7c99; border-style: dashed; }

  .matrix { flex: 1; overflow: auto; display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; align-content: start; padding-right: 4px; }
  .empty { grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 24px; font-size: 13px; }

  .footer { display: flex; gap: 9px; padding-top: 4px; }
  .fa { flex: 1; padding: 11px; border-radius: 9px; cursor: pointer; font-family: var(--font-cond); font-weight: 600; font-size: 14px; color: #cbd3db; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.12); }
  .fa.add { color: #9fd8e6; border-color: rgba(63, 199, 224, 0.4); background: rgba(63, 199, 224, 0.12); }
  .fa.del { color: #e89090; border-color: rgba(224, 90, 90, 0.35); background: rgba(224, 90, 90, 0.1); }
  .fa:hover { filter: brightness(1.15); }
</style>
