import type { BoxPal, ElementName, Gender, WorkSuit } from "./types";
import { ELEMENT_COLOR } from "./constants";
import { palIcon } from "./icons";
import { calculateCombatStats, type CombatStats } from "./palStats";
import { ref, resolveMove, resolvePassive } from "./refdata.svelte";

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
  genderSymbol: string;
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

export function genderSymbol(gender: Gender): string {
  return gender === "Male" ? "♂" : gender === "Female" ? "♀" : "–";
}

export function nextGender(gender: Gender): Gender {
  return gender === "Male" ? "Female" : "Male";
}

export function elementColor(element: ElementName | undefined): string {
  if (!element) return ELEMENT_COLOR.Neutral;
  return ref.elements[element]?.color || ELEMENT_COLOR[element];
}

export function normalizeElement(element: string | undefined): ElementName {
  return element && element in ELEMENT_COLOR ? element as ElementName : "Neutral";
}

export function presentBoxPal(pal: BoxPal): PalCardPresentation {
  const stats = calculateCombatStats(pal) ?? { hp: 0, attack: 0, defense: 0 };
  return {
    slot: pal.slot,
    title: pal.name,
    speciesName: pal.speciesName,
    showSpeciesSubtitle: !!pal.nickname && pal.nickname !== pal.speciesName,
    gender: pal.gender,
    genderSymbol: genderSymbol(pal.gender),
    level: pal.level,
    condensation: pal.condensation,
    stats,
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
