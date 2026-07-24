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
  /** Condensation rank, 0..4 stars. */
  condensation: number;
  /** Pal Souls rank (per-stat later), 0..10. */
  souls: number;
  stats: { hp: number; hpMax: number; attack: number; defense: number; workSpeed: number };
  /** Which stats show the in-game "boosted" up-arrow. */
  boosted: Partial<Record<"attack" | "defense" | "workSpeed", boolean>>;
  partnerSkill: { name: string; level: number; description: string; element?: ElementName };
  /** Up to 4. */
  passives: Passive[];
  /** Equipped Active Skills (moves), up to 3. */
  activeSkills: Move[];
  /** All 12 work suitabilities, in canonical order. */
  workSuit: WorkSuit[];
}
