// One data-driven filter over the species reference table, shared by the species
// selector and the Global Box explorer (the box maps each tile to its species row
// and reuses `speciesMatches`). Everything is derived from the reference data — no
// hardcoded per-species lists. Categories mirror the DB buckets (Natural / Tower
// Bosses / Unobtainable); "rideable" is derived from the Partner Skill saddle gear.
import type { ElementName, Category, SpeciesRow } from "./types";
import { ref } from "./refdata.svelte";

export interface SpeciesFilterState {
  search: string;
  elements: Set<ElementName>;
  /** Official work-suitability names (any-of: pal has a positive base level). */
  work: Set<string>;
  /** Can be ridden as a mount (has a saddle Partner-Skill gear). */
  rideable: boolean;
  /** Ranch/farm drop item names (any-of). */
  ranchDrops: Set<string>;
  categories: Set<Category>;
}

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

/** Immutably toggle a value in a Set (returns a new Set so $state re-renders). */
export function toggleIn<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

/** A saddle in the Partner-Skill gear = a rideable mount (data-derived, exact). */
export function isRideable(sp: SpeciesRow): boolean {
  return /saddle/i.test(sp.partnerSkill?.gearName ?? "");
}

export function speciesMatches(sp: SpeciesRow, f: SpeciesFilterState): boolean {
  if (f.search) {
    const q = f.search.toLowerCase();
    if (!sp.name.toLowerCase().includes(q) && !sp.code.toLowerCase().includes(q)) return false;
  }
  if (f.categories.size && !f.categories.has(sp.category)) return false;
  if (f.elements.size && !sp.elements.some((e) => f.elements.has(e))) return false;
  if (f.work.size) {
    let ok = false;
    for (const w of f.work) if ((sp.work[w] ?? 0) > 0) { ok = true; break; }
    if (!ok) return false;
  }
  if (f.rideable && !isRideable(sp)) return false;
  if (f.ranchDrops.size) {
    const drops = new Set(sp.farmDrops.map((d) => d.itemName));
    let ok = false;
    for (const d of f.ranchDrops) if (drops.has(d)) { ok = true; break; }
    if (!ok) return false;
  }
  return true;
}

export function filterSpecies(rows: SpeciesRow[], f: SpeciesFilterState): SpeciesRow[] {
  return rows.filter((sp) => speciesMatches(sp, f));
}

export function activeFilterCount(f: SpeciesFilterState): number {
  return (
    (f.search ? 1 : 0) +
    f.elements.size +
    f.work.size +
    f.categories.size +
    f.ranchDrops.size +
    (f.rideable ? 1 : 0)
  );
}

export function clearFilter(f: SpeciesFilterState): void {
  f.search = "";
  f.elements = new Set();
  f.work = new Set();
  f.rideable = false;
  f.ranchDrops = new Set();
  f.categories = new Set();
}

/** Distinct ranch/farm drop item names across every species, sorted for the picker. */
export function ranchDropOptions(): string[] {
  const seen = new Set<string>();
  for (const sp of ref.species) for (const d of sp.farmDrops) seen.add(d.itemName);
  return Array.from(seen).sort((a, b) => a.localeCompare(b));
}

export const CATEGORY_LABELS: { value: Category; label: string }[] = [
  { value: "Natural", label: "Natural" },
  { value: "TowerBoss", label: "Tower Bosses" },
  { value: "Unobtainable", label: "Unobtainable" },
];
