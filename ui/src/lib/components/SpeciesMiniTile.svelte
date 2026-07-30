<script lang="ts">
  import type { SpeciesRow } from "$lib/data/types";
  import { palIcon } from "$lib/data/icons";
  import { elementColor } from "$lib/data/palPresentation";
  import { ref } from "$lib/data/refdata.svelte";
  import ElementIcon from "./ElementIcon.svelte";
  import PalArtwork from "./PalArtwork.svelte";
  import WorkIcon from "./WorkIcon.svelte";

  let {
    species,
    current = false,
    onselect,
  }: {
    species: SpeciesRow;
    current?: boolean;
    onselect: (code: string) => void;
  } = $props();

  const primary = $derived(elementColor(species.elements[0]));
  const secondary = $derived(elementColor(species.elements[1] ?? species.elements[0]));
  const work = $derived(
    ref.workTypes.flatMap((definition) => {
      const level = species.work[definition.code] ?? 0;
      return level > 0 ? [{ ...definition, level }] : [];
    }),
  );
  const workSummary = $derived(
    work.length
      ? work.map((suitability) => `${suitability.name} Lv.${suitability.level}`).join(", ")
      : "No work suitabilities",
  );
</script>

<button
  type="button"
  class="tile"
  class:current
  style="--primary:{primary}; --secondary:{secondary}"
  data-species-code={species.code}
  onclick={() => onselect(species.code)}
  title={`${species.name} · ${workSummary}`}
  aria-label={`Select ${species.name}. ${workSummary}.`}
>
  <span class="portrait">
    <span class="portrait-inner">
      <PalArtwork src={palIcon(species.code)} shape="circle" />
    </span>
  </span>

  <span class="name">{species.name}</span>

  <span class="elements" aria-label={`Elements: ${species.elements.join(", ")}`}>
    {#each species.elements as element}
      <span class="element" title={element}>
        <ElementIcon {element} size={22} decorative={false} />
      </span>
    {/each}
  </span>

  <span class="work" aria-label={workSummary}>
    {#each work as suitability (suitability.name)}
      <span class="suitability" title={`${suitability.name} Lv.${suitability.level}`}>
        <WorkIcon
          icon={suitability.icon}
          name={suitability.name}
          level={suitability.level}
          size={20}
        />
        <b>{suitability.level}</b>
      </span>
    {/each}
    {#if !work.length}<span class="no-work">No work</span>{/if}
  </span>
</button>

<style>
  .tile {
    position: relative;
    min-width: 0;
    height: 250px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    padding: 14px 10px 12px;
    overflow: hidden;
    border-radius: 13px;
    cursor: pointer;
    color: var(--text-1);
    background:
      radial-gradient(circle at 14% 0%, color-mix(in srgb, var(--primary) 18%, transparent), transparent 44%),
      radial-gradient(circle at 90% 100%, color-mix(in srgb, var(--secondary) 15%, transparent), transparent 50%),
      rgba(18, 22, 31, 0.96);
    border: 1px solid color-mix(in srgb, var(--primary) 31%, rgba(255, 255, 255, 0.08));
    box-shadow: inset 0 1px rgba(255, 255, 255, 0.03);
    transition: transform 0.14s, border-color 0.14s, box-shadow 0.14s, background 0.14s;
  }
  .tile:hover {
    transform: translateY(-1px);
    border-color: color-mix(in srgb, var(--primary) 55%, var(--secondary));
    box-shadow:
      0 8px 20px rgba(0, 0, 0, 0.32),
      0 0 16px color-mix(in srgb, var(--primary) 17%, transparent);
  }
  .tile.current {
    border-color: var(--accent-cyan);
    box-shadow:
      0 0 0 1px rgba(63, 199, 224, 0.28),
      0 0 18px rgba(63, 199, 224, 0.32);
  }
  .tile.current::after {
    content: "CURRENT";
    position: absolute;
    top: 7px;
    right: 7px;
    padding: 3px 6px;
    border-radius: 7px;
    color: #b9f2fb;
    background: rgba(63, 199, 224, 0.16);
    border: 1px solid rgba(63, 199, 224, 0.34);
    font: 700 var(--type-micro) var(--font-head);
    letter-spacing: 0.08em;
  }
  .portrait {
    width: 82px;
    height: 82px;
    flex: none;
    display: grid;
    place-items: center;
    padding: 2px;
    border-radius: 50%;
    background: conic-gradient(from 180deg, var(--primary) 0 50%, var(--secondary) 50% 100%);
    box-shadow:
      0 0 13px color-mix(in srgb, var(--primary) 32%, transparent),
      0 0 16px color-mix(in srgb, var(--secondary) 22%, transparent);
  }
  .portrait-inner {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: inherit;
    background:
      radial-gradient(circle at 50% 30%, rgba(255, 255, 255, 0.13), transparent 52%),
      linear-gradient(
        145deg,
        color-mix(in srgb, var(--primary) 15%, #111722),
        color-mix(in srgb, var(--secondary) 12%, #0c1119)
      );
    box-shadow: inset 0 0 18px rgba(0, 0, 0, 0.48);
  }
  .name {
    display: block;
    flex: none;
    width: 100%;
    min-height: 20px;
    overflow: hidden;
    color: #eee9f2;
    font: 600 var(--type-title)/1.08 var(--font-cond);
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .elements {
    flex: none;
    min-height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
  }
  .element {
    display: grid;
    place-items: center;
    width: 27px;
    height: 27px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.045);
  }
  .work {
    flex: none;
    width: 100%;
    min-height: 31px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
    gap: 4px;
    padding-top: 7px;
    border-top: 1px solid rgba(255, 255, 255, 0.065);
  }
  .suitability {
    min-width: 34px;
    height: 29px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 3px 5px;
    border-radius: 7px;
    color: #d8c48e;
    background: rgba(245, 166, 35, 0.075);
    border: 1px solid rgba(245, 166, 35, 0.14);
  }
  .suitability b {
    font: 700 var(--type-label) var(--font-head);
    font-variant-numeric: tabular-nums;
  }
  .no-work {
    color: #6f7781;
    font-size: var(--type-label);
    font-style: italic;
  }
</style>
