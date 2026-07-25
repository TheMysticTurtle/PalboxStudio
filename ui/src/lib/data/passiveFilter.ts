import type { PassiveRef } from "./types";

export type PassiveScope = "species" | "normal" | "lucky" | "all";
export type PassiveTone = "all" | "positive" | "negative";
export type PassiveSort = "rating" | "name";

export const PASSIVE_GROUPS = [
  "Combat",
  "Element Damage",
  "Element Resistance",
  "Work",
  "Movement",
  "Survival",
  "Economy",
  "Utility",
] as const;
export type PassiveGroup = (typeof PASSIVE_GROUPS)[number];

/** Turn the engine's exact passive-effect identifiers into useful UI facets. */
export function passiveGroups(passive: PassiveRef): Set<PassiveGroup> {
  const groups = new Set<PassiveGroup>();
  for (const effect of passive.effects) {
    const type = effect.type;
    if (/^ElementBoost_/.test(type)) groups.add("Element Damage");
    else if (/^ElementResist_/.test(type)) groups.add("Element Resistance");
    else if (/ShotAttack|Defense|MaxHP|LifeSteal|ActiveSkill|NonKilling/.test(type)) groups.add("Combat");
    else if (/CraftSpeed|Logging|Mining|CollectItem|BreedSpeed/.test(type)) groups.add("Work");
    else if (/MoveSpeed|JumpCount|JumpPower/.test(type)) groups.add("Movement");
    else if (/FullStomatch|Sanity|TemperatureResist|Nocturnal/.test(type)) groups.add("Survival");
    else if (/ShopBuy|ShopSell/.test(type)) groups.add("Economy");
    else groups.add("Utility");
  }
  if (!groups.size) groups.add("Utility");
  return groups;
}

export function passiveMatches(
  code: string,
  passive: PassiveRef,
  search: string,
  scope: PassiveScope,
  tone: PassiveTone,
  group: PassiveGroup | "all",
  includeDisabled: boolean,
  speciesPassives: Set<string>,
): boolean {
  if (!includeDisabled && passive.disabled) return false;
  if (scope === "species" && !passive.availableNormalPal && !speciesPassives.has(code)) return false;
  if (scope === "normal" && !passive.availableNormalPal) return false;
  if (scope === "lucky" && !passive.availableLuckyPal) return false;
  if (tone === "positive" && passive.rating <= 0) return false;
  if (tone === "negative" && passive.rating >= 0) return false;
  if (group !== "all" && !passiveGroups(passive).has(group)) return false;
  const q = search.trim().toLowerCase();
  if (!q) return true;
  return (
    code.toLowerCase().includes(q) ||
    passive.name.toLowerCase().includes(q) ||
    passive.description.toLowerCase().includes(q) ||
    passive.effects.some((effect) => effect.type.toLowerCase().includes(q))
  );
}
