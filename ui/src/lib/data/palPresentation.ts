import type { BoxPal, ElementName, Gender, WorkSuit } from "./types";
import { palIcon } from "./icons";
import { ref, resolveMove, resolvePassive } from "./refdata.svelte";

export interface CombatStats {
  hp: number;
  attack: number;
  defense: number;
}

export interface PresentedPassive {
  code: string;
  name: string;
  rating: number;
}

export interface PresentedMove {
  code: string;
  name: string;
  element: ElementName;
  power: number;
}

/**
 * Read-only projection shared by every Global Palbox card density. Codes stay
 * canonical on BoxPal; localized names and asset/theme metadata resolve here.
 */
export interface PalCardPresentation {
  slot: number;
  title: string;
  speciesName: string;
  showSpeciesSubtitle: boolean;
  gender: Gender;
  level: number;
  condensation: number;
  stats: CombatStats;
  elements: ElementName[];
  primaryColor: string;
  secondaryColor: string;
  portrait: string;
  alpha: boolean;
  lucky: boolean;
  groups: string[];
  workSuit: WorkSuit[];
  passives: PresentedPassive[];
  moves: PresentedMove[];
}

export function nextGender(gender: Gender): Gender {
  return gender === "Male" ? "Female" : "Male";
}

export function elementColor(element: ElementName | undefined): string {
  return (element && ref.elements[element]?.color) || "var(--el-neutral)";
}

export function normalizeElement(element: string | undefined): ElementName {
  return element && element in ref.elements
    ? element
    : (Object.keys(ref.elements)[0] ?? "");
}

export function presentBoxPal(pal: BoxPal): PalCardPresentation {
  return {
    slot: pal.slot,
    title: pal.name,
    speciesName: pal.speciesName,
    showSpeciesSubtitle: !!pal.nickname && pal.nickname !== pal.speciesName,
    gender: pal.gender,
    level: pal.level,
    condensation: pal.condensation,
    stats: pal.stats,
    elements: pal.elements,
    primaryColor: elementColor(pal.elements[0]),
    secondaryColor: elementColor(pal.elements[1] ?? pal.elements[0]),
    portrait: palIcon(pal.species),
    alpha: !!pal.alpha,
    lucky: !!pal.lucky,
    groups: pal.groups ?? [],
    workSuit: pal.workSuit.filter((work) => work.level > 0),
    passives: pal.passives.map((code) => {
      const passive = resolvePassive(code);
      return { code, name: passive?.name ?? code, rating: passive?.rating ?? 0 };
    }),
    moves: pal.activeSkills.map((code) => {
      const move = resolveMove(code);
      return {
        code,
        name: move?.name ?? code,
        element: move?.element || "Neutral",
        power: move?.power ?? 0,
      };
    }),
  };
}
