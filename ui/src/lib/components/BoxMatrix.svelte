<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { sampleBox } from "$lib/data/sampleBox";
  import { ui } from "$lib/stores/ui.svelte";
  import { box, selectSlot } from "$lib/stores/box.svelte";
  import { palToBoxPal, tileDtoToBoxPal } from "$lib/data/mapper";
  import BoxTile from "./BoxTile.svelte";

  let source: BoxPal[] = $derived(
    box.open
      ? box.tiles.map((tile) =>
          tile.slot === box.selectedSlot && box.pal
            ? palToBoxPal(box.pal, tile.slot)
            : tileDtoToBoxPal(tile),
        )
      : sampleBox,
  );

  function select(id: string) {
    if (box.open) selectSlot(Number(id));
    else ui.selectedId = id;
  }
  const isSelected = (id: string) =>
    box.open ? box.selectedSlot === Number(id) : ui.selectedId === id;
</script>

<div class="overlay">
  <div class="head">
    <span class="diamond"></span>
    <h2>GLOBAL PALBOX</h2>
    <span class="count">{source.length} pals</span>
    <button class="collapse" onclick={() => (ui.boxExpanded = false)} aria-label="Collapse to drawer">⤡ Collapse</button>
  </div>
  <div class="grid">
    {#each source as p (p.instanceId)}
      <BoxTile pal={p} size="lg" selected={isSelected(p.instanceId)} onselect={select} />
    {/each}
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
  .count { color: #9782a8; font-size: 13px; }
  .collapse {
    margin-left: auto;
    padding: 9px 15px;
    border-radius: 9px;
    border: 1px solid rgba(176, 96, 224, 0.4);
    background: rgba(176, 96, 224, 0.12);
    color: #d6bef2;
    cursor: pointer;
    font-size: 13px;
  }
  .collapse:hover { background: rgba(176, 96, 224, 0.22); }
  .grid {
    flex: 1;
    overflow: auto;
    padding: 24px 30px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 14px;
    align-content: start;
  }
</style>
