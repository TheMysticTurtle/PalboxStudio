import type { ElementName, Category, SpeciesRow } from "./types";

export interface SpeciesFilterState {
  search: string;
  elements: Set<ElementName>;
  /** Official work-suitability names (all selected jobs must have a positive base level). */
  work: Set<string>;
  /** Can be ridden as a mount (has saddle Partner-Skill gear). */
  rideable: boolean;
  /** Ranch/farm drop item names (all selected drops must be present). */
  ranchDrops: Set<string>;
  /** Mutually exclusive species source category. */
  categories: Set<Category>;
}

/** Immutably toggle a value in a Set (returns a new Set so $state re-renders). */
export function toggleIn<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

/** Toggle one mutually-exclusive value, or clear it by clicking it again. */
export function toggleOnly<T>(set: Set<T>, value: T): Set<T> {
  return set.has(value) ? new Set<T>() : new Set<T>([value]);
}

/** A saddle in the Partner-Skill gear = a rideable mount (data-derived, exact). */
export function isRideable(sp: SpeciesRow): boolean {
  return /saddle/i.test(sp.partnerSkill?.gearName ?? "");
}

/**
 * Intersection semantics: a row must satisfy every selected value in every
 * multi-select facet, and it must also satisfy every other active facet.
 */
export function speciesMatches(sp: SpeciesRow, f: SpeciesFilterState): boolean {
  if (f.search) {
    const q = f.search.toLowerCase();
    if (!sp.name.toLowerCase().includes(q) && !sp.code.toLowerCase().includes(q)) return false;
  }
  if (f.categories.size && !f.categories.has(sp.category)) return false;
  if (
    f.elements.size
    && !Array.from(f.elements).every((element) => sp.elements.includes(element))
  ) return false;
  if (
    f.work.size
    && !Array.from(f.work).every((work) => (sp.work[work] ?? 0) > 0)
  ) return false;
  if (f.rideable && !isRideable(sp)) return false;
  if (f.ranchDrops.size) {
    const drops = new Set(sp.farmDrops.map((drop) => drop.itemName));
    if (!Array.from(f.ranchDrops).every((drop) => drops.has(drop))) return false;
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

export const CATEGORY_LABELS: { value: Category; label: string }[] = [
  { value: "Natural", label: "Natural" },
  { value: "TowerBoss", label: "Tower Bosses" },
  { value: "Unobtainable", label: "Unobtainable" },
];
