// One data-driven filter over the species reference table, shared by the species
// selector and the Global Box explorer (the box maps each tile to its species row
// and reuses `speciesMatches`). Everything is derived from the reference data — no
// hardcoded per-species lists. Categories mirror the DB buckets (Natural / Tower
// Bosses / Unobtainable); "rideable" is derived from the Partner Skill saddle gear.
import type { Category, ElementName } from "./types";
import { ref } from "./refdata.svelte";
import type { SpeciesFilterState } from "./speciesFilter";

export * from "./speciesFilter";

/** A fresh, reactive (empty) filter. Sets are reassigned on toggle for reactivity. */
export function createSpeciesFilter(): SpeciesFilterState {
  const state = $state<SpeciesFilterState>({
    search: "",
    elements: new Set<ElementName>(),
    work: new Set<string>(),
    rideable: false,
    ranchDrops: new Set<string>(),
    categories: new Set<Category>(),
  });
  return state;
}

/** Distinct ranch/farm drop item names across every species, sorted for the picker. */
export function ranchDropOptions(): string[] {
  const seen = new Set<string>();
  for (const sp of ref.species) for (const d of sp.farmDrops) seen.add(d.itemName);
  return Array.from(seen).sort((a, b) => a.localeCompare(b));
}
