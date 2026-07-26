<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { presentBoxPal } from "$lib/data/palPresentation";
  import ElementIcon from "./ElementIcon.svelte";
  import PalPortrait from "./PalPortrait.svelte";

  let {
    pal,
    selected = false,
    onselect,
  }: {
    pal: BoxPal;
    selected?: boolean;
    onselect?: (slot: number) => void;
  } = $props();

  const card = $derived(presentBoxPal(pal));
</script>

<button
  type="button"
  class="tile"
  class:selected
  style="--primary:{card.primaryColor}; --secondary:{card.secondaryColor}"
  onclick={() => onselect?.(card.slot)}
  title={`${card.title} · ${card.speciesName} · Level ${card.level}`}
  aria-label={`Select ${card.title}, ${card.speciesName}, level ${card.level}`}
>
  <PalPortrait {card} size={58} />
  <span class="name">{card.title}</span>
  {#if card.showSpeciesSubtitle}<span class="species">{card.speciesName}</span>{/if}
  <span class="meta">
    <span class="elements">
      {#each card.elements as element}<ElementIcon {element} size={15} />{/each}
    </span>
    <span class="level">Lv.{card.level}</span>
    <span class="gender" class:male={card.gender === "Male"} class:female={card.gender === "Female"}>{card.genderSymbol}</span>
  </span>
  {#if card.condensation > 0 || card.groups.length}
    <span class="foot">
      {#if card.condensation > 0}<span class="cond" title="Condensation">★{card.condensation}</span>{/if}
      {#if card.groups.length}<span class="group">{card.groups[0]}</span>{/if}
    </span>
  {/if}
</button>

<style>
  .tile {
    position: relative;
    min-width: 0;
    min-height: 126px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 12px 7px 9px;
    border-radius: 12px;
    cursor: pointer;
    font: inherit;
    color: var(--text-1);
    background:
      radial-gradient(circle at 16% 0%, color-mix(in srgb, var(--primary) 15%, transparent), transparent 45%),
      radial-gradient(circle at 88% 100%, color-mix(in srgb, var(--secondary) 12%, transparent), transparent 48%),
      rgba(19, 24, 33, 0.92);
    border: 1px solid color-mix(in srgb, var(--primary) 28%, rgba(255, 255, 255, 0.08));
    box-shadow: inset 0 1px rgba(255, 255, 255, 0.025);
    transition: border-color 0.15s, box-shadow 0.15s, background 0.15s, transform 0.15s;
  }
  .tile:hover {
    transform: translateY(-1px);
    border-color: color-mix(in srgb, var(--primary) 55%, var(--secondary));
    box-shadow: 0 8px 18px rgba(0, 0, 0, 0.28);
  }
  .tile.selected {
    border-color: color-mix(in srgb, var(--primary) 55%, var(--secondary));
    box-shadow:
      0 0 0 1px color-mix(in srgb, var(--secondary) 34%, transparent),
      0 0 17px color-mix(in srgb, var(--primary) 28%, transparent);
  }
  .name {
    display: block;
    width: 100%;
    overflow: hidden;
    color: #eee9f2;
    font: 600 13.5px/1.05 var(--font-cond);
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .species {
    display: block;
    width: 100%;
    overflow: hidden;
    color: #8f829c;
    font-size: 10px;
    line-height: 1;
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta { display: flex; align-items: center; justify-content: center; gap: 5px; min-height: 17px; }
  .elements { display: flex; align-items: center; gap: 1px; }
  .level { color: #b1a6bc; font-size: 11px; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .gender {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    color: #9099a3;
    font: 700 11px var(--font-head);
    background: rgba(255, 255, 255, 0.055);
  }
  .gender.male { color: #8fc8ff; background: rgba(63, 143, 224, 0.16); }
  .gender.female { color: #f2a0d8; background: rgba(224, 95, 192, 0.16); }
  .foot { display: flex; align-items: center; justify-content: center; gap: 4px; max-width: 100%; min-height: 16px; }
  .cond { color: #f2c968; font: 700 10px var(--font-head); }
  .group {
    max-width: 78px;
    overflow: hidden;
    padding: 2px 6px;
    border-radius: 8px;
    color: #cbb6de;
    background: rgba(176, 96, 224, 0.13);
    font-size: 9px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
