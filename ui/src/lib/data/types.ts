// Domain types for the UI. These mirror what the Rust core will hand across the
// bridge once save-loading is wired; for now we render a sample pal against them.

export type ElementName =
  | "Neutral"
  | "Fire"
  | "Water"
  | "Grass"
  | "Electric"
  | "Ice"
  | "Ground"
  | "Dark"
  | "Dragon";

export type Gender = "Male" | "Female" | "Unknown";

export interface Move {
  id: string;
  name: string;
  element: ElementName;
  power: number;
}

export interface Passive {
  id: string;
  name: string;
  /** -3..5 (no 0); 5 is the 1.0 top tier. */
  rating: number;
  /** Short human blurb of what it adjusts. */
  effects: string;
}

export interface WorkSuit {
  name: string;
  /** Icon basename in /icons/work — active = `<icon>.png`, level 0 = `no_<icon>.png`. */
  icon: string;
  /** 0..10 in 1.0. */
  level: number;
}

export interface Pal {
  species: string;
  /** Display name (nickname if set, else species). */
  name: string;
  paldexNo: string;
  gender: Gender;
  elements: ElementName[];
  level: number;
  /** Exp remaining to next level (display only for now). */
  expToNext: number;
  /** 0..1 progress within the current level. */
  expPct: number;
  favorite: boolean;
  alpha: boolean;
  lucky: boolean;
  /** Condensation rank, 0..4 stars (save: `rank`). */
  condensation: number;
  /** IV / breeding talents, 0..100 display (save: Talent_HP/Shot/Defense; raw byte 0..255). */
  ivs: { hp: number; shot: number; defense: number };
  /** Pal Souls rank per stat, 0..10 (Statue of Power; save: Rank_HP/Attack/Defence/CraftSpeed). */
  soulRanks: { hp: number; attack: number; defense: number; craftSpeed: number };
  stats: {
    hp: number;
    hpMax: number;
    attack: number;
    defense: number;
    workSpeed: number;
    /** SAN 0..100. */
    san: number;
    /** Food fullness 0..1. */
    foodPct: number;
  };
  /** Which stats show the in-game "boosted" up-arrow. */
  boosted: Partial<Record<"attack" | "defense" | "workSpeed", boolean>>;
  trust: { rank: number; pct: number };
  partnerSkill: { name: string; level: number; description: string; element?: ElementName };
  /** Up to 4. */
  passives: Passive[];
  /** Equipped Active Skills (moves), up to 3. */
  activeSkills: Move[];
  /** Unequipped / available moves (bench). Placeholder until the core reads the pal's real learnset. */
  benchMoves: Move[];
  /** All 13 work suitabilities, in canonical order. */
  workSuit: WorkSuit[];
}

/** Lightweight summary for a pal shown in the Global Box explorer (tiles).
 *  Only species that CAN live in the global box appear — humans/NPCs are excluded. */
export interface BoxPal {
  instanceId: string;
  /** CodeName → icon T_<species>_icon_normal.png */
  species: string;
  name: string;
  level: number;
  elements: ElementName[];
  alpha?: boolean;
  lucky?: boolean;
  /** Editor-side group tags (app metadata — NOT written to the save). */
  groups?: string[];
}
