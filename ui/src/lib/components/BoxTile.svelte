<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { ELEMENT_COLOR } from "$lib/data/constants";
  import { palIcon, onPalIconError } from "$lib/data/icons";

  let {
    pal,
    size = "sm",
    selected = false,
    onselect,
  }: {
    pal: BoxPal;
    size?: "sm" | "lg";
    selected?: boolean;
    onselect?: (id: string) => void;
  } = $props();

  const iconSrc = $derived(palIcon(pal.species));
</script>

<button class="tile {size}" class:selected onclick={() => onselect?.(pal.instanceId)} title={pal.name}>
  <div class="badges">
    {#if pal.alpha}<img class="emblem alpha" src="/icons/variants/alpha.webp" alt="Alpha" title="Alpha" />{/if}
    {#if pal.lucky}<img class="emblem lucky" src="/icons/variants/lucky.webp" alt="Lucky" title="Lucky" />{/if}
  </div>
  <div class="port"><img src={iconSrc} alt="" loading="lazy" decoding="async" onerror={onPalIconError} /></div>
  <div class="name">{pal.name}</div>
  <div class="meta">
    {#each pal.elements as el}<span class="dia" style="--c:{ELEMENT_COLOR[el]}"></span>{/each}
    <span class="lv">Lv.{pal.level}</span>
  </div>
  {#if size === "lg" && pal.groups?.length}
    <div class="groups">{#each pal.groups as g}<span class="gchip">{g}</span>{/each}</div>
  {/if}
</button>

<style>
  .tile {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 11px 9px 10px;
    border-radius: 12px;
    cursor: pointer;
    color: var(--text-1);
    background: rgba(176, 96, 224, 0.05);
    border: 1px solid rgba(176, 96, 224, 0.18);
    transition: border-color 0.15s, box-shadow 0.15s, background 0.15s;
  }
  .tile:hover { border-color: rgba(176, 96, 224, 0.5); background: rgba(176, 96, 224, 0.1); }
  .tile.selected {
    border-color: var(--accent-purple);
    box-shadow: 0 0 16px rgba(176, 96, 224, 0.4);
    background: rgba(176, 96, 224, 0.14);
  }
  .badges { position: absolute; z-index: 2; top: 5px; left: 5px; display: flex; align-items: center; gap: 2px; }
  .emblem {
    width: 25px;
    height: 25px;
    object-fit: contain;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.8));
  }
  .emblem.lucky { width: 22px; height: 22px; }
  .port {
    border-radius: 50%;
    background: radial-gradient(circle at 50% 35%, rgba(255, 255, 255, 0.1), rgba(0, 0, 0, 0.35));
    border: 1px solid rgba(176, 96, 224, 0.3);
    display: grid;
    place-items: center;
    overflow: hidden;
    margin-top: 4px;
  }
  .port img { width: 100%; height: 100%; object-fit: contain; }
  .name {
    font-family: var(--font-cond);
    font-weight: 600;
    color: #eae2f2;
    text-align: center;
    line-height: 1.05;
  }
  .meta { display: flex; align-items: center; gap: 6px; }
  .dia { width: 8px; height: 8px; transform: rotate(45deg); background: var(--c); box-shadow: 0 0 5px var(--c); }
  .lv { font-size: 11.5px; color: #a99bb8; font-variant-numeric: tabular-nums; }
  .groups { display: flex; flex-wrap: wrap; gap: 4px; justify-content: center; margin-top: 2px; }
  .gchip { font-size: 10px; color: #c9b4e0; padding: 2px 7px; border-radius: 10px; background: rgba(176, 96, 224, 0.14); border: 1px solid rgba(176, 96, 224, 0.3); }

  /* sizes */
  .sm .port { width: 56px; height: 56px; }
  .sm .name { font-size: 13.5px; }
  .lg { padding: 16px 12px 14px; gap: 8px; }
  .lg .port { width: 96px; height: 96px; }
  .lg .name { font-size: 16px; }
  .lg .lv { font-size: 13px; }
  .lg .emblem { width: 34px; height: 34px; }
  .lg .emblem.lucky { width: 30px; height: 30px; }
</style>
