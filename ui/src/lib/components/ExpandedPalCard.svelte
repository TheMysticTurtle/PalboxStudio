<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import { presentBoxPal } from "$lib/data/palPresentation";
  import { ratingTone } from "$lib/data/constants";
  import ElementIcon from "./ElementIcon.svelte";
  import PalPortrait from "./PalPortrait.svelte";
  import WorkIcon from "./WorkIcon.svelte";

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
  class="card"
  class:selected
  style="--primary:{card.primaryColor}; --secondary:{card.secondaryColor}"
  onclick={() => onselect?.(card.slot)}
  aria-label={`Select ${card.title}, ${card.speciesName}, level ${card.level}`}
>
  <span class="accent"></span>
  <span class="identity">
    <PalPortrait {card} size={76} />
    <span class="copy">
      <strong>{card.title}</strong>
      {#if card.showSpeciesSubtitle}<span class="species">{card.speciesName}</span>{/if}
      <span class="meta">
        <span class="elements">{#each card.elements as element}<ElementIcon {element} size={16} />{/each}</span>
        <span>Lv.{card.level}</span>
        <span class="gender" class:male={card.gender === "Male"} class:female={card.gender === "Female"}>
          {card.genderSymbol}
        </span>
        {#if card.condensation > 0}<span class="cond">★{card.condensation}</span>{/if}
      </span>
    </span>
    {#if card.alpha || card.lucky}
      <span class="variants">
        {#if card.alpha}<span class="alpha">ALPHA</span>{/if}
        {#if card.lucky}<span class="lucky">LUCKY</span>{/if}
      </span>
    {/if}
  </span>

  <span class="section ivs" aria-label="Individual values">
    <span><small>HP IV</small><b>{card.ivs.hp}</b></span>
    <span><small>ATK IV</small><b>{card.ivs.shot}</b></span>
    <span><small>DEF IV</small><b>{card.ivs.defense}</b></span>
  </span>

  <span class="section">
    <span class="label">WORK SUITABILITIES</span>
    <span class="work-list">
      {#each card.workSuit as work (work.name)}
        <span class="work" title="{work.name} Lv.{work.level}">
          <WorkIcon icon={work.icon} name={work.name} level={work.level} size={18} />
          <b>Lv.{work.level}</b>
        </span>
      {/each}
      {#if !card.workSuit.length}<span class="none">None</span>{/if}
    </span>
  </span>

  <span class="section">
    <span class="label">EQUIPPED MOVES</span>
    <span class="move-list">
      {#each card.moves as move (move.code)}
        <span class="move" title="{move.name} · {move.element} · Power {move.power}">
          <ElementIcon element={move.element} size={15} />
          <span>{move.name}</span>
          <b>{move.power}</b>
        </span>
      {/each}
      {#if !card.moves.length}<span class="none">No moves equipped</span>{/if}
    </span>
  </span>

  <span class="section passive-section">
    <span class="label">PASSIVES</span>
    <span class="passive-list">
      {#each card.passives as passive (passive.code)}
        <span class="passive" style="--tone:{ratingTone(passive.rating)}" title={passive.name}>{passive.name}</span>
      {/each}
      {#if !card.passives.length}<span class="none">No passives</span>{/if}
    </span>
  </span>

  {#if card.groups.length}
    <span class="groups">
      {#each card.groups as group}<span>{group}</span>{/each}
    </span>
  {/if}
</button>

<style>
  .card {
    position: relative;
    min-width: 0;
    min-height: 348px;
    display: flex;
    flex-direction: column;
    gap: 9px;
    padding: 13px;
    overflow: hidden;
    color: #e8e4ec;
    text-align: left;
    cursor: pointer;
    font: inherit;
    border-radius: 13px;
    border: 1px solid color-mix(in srgb, var(--primary) 26%, rgba(255, 255, 255, 0.09));
    background:
      radial-gradient(circle at 0% 0%, color-mix(in srgb, var(--primary) 13%, transparent), transparent 37%),
      radial-gradient(circle at 100% 100%, color-mix(in srgb, var(--secondary) 11%, transparent), transparent 42%),
      linear-gradient(155deg, rgba(24, 28, 39, 0.97), rgba(14, 18, 26, 0.98));
    box-shadow: inset 0 1px rgba(255, 255, 255, 0.025), 0 8px 24px rgba(0, 0, 0, 0.2);
    transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
  }
  .card:hover { transform: translateY(-2px); border-color: color-mix(in srgb, var(--primary) 56%, var(--secondary)); }
  .card.selected {
    border-color: color-mix(in srgb, var(--primary) 55%, var(--secondary));
    box-shadow:
      0 0 0 1px color-mix(in srgb, var(--secondary) 30%, transparent),
      0 0 22px color-mix(in srgb, var(--primary) 24%, transparent);
  }
  .accent {
    position: absolute;
    inset: 0 0 auto;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--secondary));
    box-shadow: 0 0 10px color-mix(in srgb, var(--primary) 38%, transparent);
  }
  .identity { position: relative; display: flex; align-items: center; gap: 11px; min-width: 0; }
  .copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .copy strong {
    overflow: hidden;
    color: #f2edf5;
    font: 600 17px/1.05 var(--font-cond);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .species { color: #91869c; font-size: 11px; }
  .meta { display: flex; align-items: center; gap: 5px; color: #aba1b5; font-size: 11px; }
  .elements { display: flex; align-items: center; gap: 1px; }
  .gender { font: 700 12px var(--font-head); color: #929ca7; }
  .gender.male { color: #8fc8ff; }
  .gender.female { color: #f2a0d8; }
  .cond { color: #f1c95e; font: 700 10px var(--font-head); }
  .variants { position: absolute; top: 0; right: 0; display: flex; flex-direction: column; align-items: flex-end; gap: 3px; }
  .variants span { padding: 2px 5px; border-radius: 7px; font: 700 8px var(--font-head); letter-spacing: .08em; }
  .variants .alpha { color: #ff9a9a; background: rgba(255, 64, 64, 0.14); border: 1px solid rgba(255, 64, 64, 0.3); }
  .variants .lucky { color: #9fddff; background: rgba(70, 170, 255, 0.14); border: 1px solid rgba(70, 170, 255, 0.3); }
  .section { display: flex; flex-direction: column; gap: 5px; padding-top: 8px; border-top: 1px solid rgba(255, 255, 255, 0.065); }
  .label { color: #716b7a; font: 600 9px var(--font-head); letter-spacing: .1em; }
  .ivs { display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; }
  .ivs > span { display: flex; align-items: baseline; justify-content: space-between; padding: 5px 7px; border-radius: 7px; background: rgba(255, 255, 255, 0.035); }
  .ivs small { color: #756e7f; font: 600 8px var(--font-head); letter-spacing: .05em; }
  .ivs b { color: #d8d1de; font: 700 13px var(--font-head); }
  .work-list { display: flex; flex-wrap: wrap; gap: 4px; }
  .work { display: flex; align-items: center; gap: 3px; padding: 3px 5px; border-radius: 7px; background: rgba(245, 166, 35, 0.07); border: 1px solid rgba(245, 166, 35, 0.12); }
  .work b { color: #d5c8ab; font: 600 9px var(--font-head); white-space: nowrap; }
  .move-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 4px; }
  .move { min-width: 0; display: flex; align-items: center; gap: 4px; padding: 4px 5px; border-radius: 7px; background: rgba(255, 255, 255, 0.032); }
  .move span { min-width: 0; flex: 1; overflow: hidden; color: #c8c1cf; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .move b { color: #827b8b; font: 600 9px var(--font-head); }
  .passive-section { margin-top: auto; }
  .passive-list { display: flex; flex-wrap: wrap; gap: 4px; }
  .passive {
    max-width: 100%;
    overflow: hidden;
    padding: 3px 7px;
    color: color-mix(in srgb, var(--tone) 38%, #e9e2ee);
    font-size: 9.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--tone) 25%, transparent);
    background: color-mix(in srgb, var(--tone) 9%, transparent);
  }
  .none { color: #625d69; font-size: 10px; font-style: italic; }
  .groups { display: flex; flex-wrap: wrap; gap: 4px; }
  .groups span { padding: 3px 7px; border-radius: 8px; color: #cab7dc; background: rgba(176, 96, 224, 0.11); border: 1px solid rgba(176, 96, 224, 0.2); font-size: 9px; }
</style>
