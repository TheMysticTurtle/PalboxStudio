// Domain types.
//
// Design rule: a pal (editable, per-instance, from the save) stores **codes**.
// Human-readable info (passive name/rating/description, move element/power, species
// stats) is *resolved* from the static reference tables (see refdata.svelte.ts) —
// never hand-written onto the pal. That keeps one source of truth and makes the
// display always match the game data.

export type ElementName =
  | "Neutral" | "Fire" | "Water" | "Grass" | "Electric"
  | "Ice" | "Ground" | "Dark" | "Dragon";

export type Gender = "Male" | "Female" | "Unknown";
export type Category = "Natural" | "TowerBoss" | "Unobtainable";

// ---- Reference tables (static; from the SQLite reference DB via get_reference_data) ----

/** passives.json: code -> this. */
export interface PassiveEffectRef {
  type: string;
  value: number | null;
  target: string | null;
}

export interface PassiveRef {
  name: string;
  /** -3..5 (no 0); 5 is the 1.0 top tier. */
  rating: number;
  description: string;
  disabled: boolean;
  availableNormalPal: boolean;
  availableLuckyPal: boolean;
  effects: PassiveEffectRef[];
}

/** moves.json: code -> this. */
export interface MoveRef {
  name: string;
  element: ElementName | "";
  power: number;
  category: string; // Shot / Melee / Status / Unique
  /** A non-disabled Skill Fruit item exists for this move. */
  skillFruit: boolean;
}

/** elements.json: element name -> this. */
export interface ElementInfo {
  color: string;
  /** Canonical basename for the bundled element badge. */
  icon: string;
}

/** schema.json: one per species column; drives filter/display generation. */
export interface SchemaColumn {
  key: string;
  label: string;
  type: "text" | "enum" | "multi" | "number" | "bool" | "map" | "relation";
  filterable: boolean;
  displayable: boolean;
  values?: string[];
}

export interface PartnerSkillRef {
  name: string;
  description: string;
  category: string | null;
  element: ElementName | null;
  gearName: string | null;
  technologyLevel: number | null;
}

export interface RanchDropRef {
  itemCode: string | null;
  itemName: string;
  notes: string | null;
}

/** species.json: one row per box-storable pal. */
export interface SpeciesRow {
  code: string;
  name: string;
  elements: ElementName[];
  category: Category;
  disabled: boolean;
  rarity: number;
  size: string;
  genus: string;
  nocturnal: boolean;
  alpha: boolean;
  deckIndex: number;
  combiRank: number;
  captureRate: number;
  price: number;
  foodAmount: number;
  maxStomach: number;
  maleProbability: number;
  runSpeed: number;
  rideSpeed: number;
  scaling: { hp: number; attack: number; defense: number };
  work: Record<string, number>;
  moves: string[];
  passives: string[];
  partnerSkill: PartnerSkillRef | null;
  farmDrops: RanchDropRef[];
}

export interface ReferenceBundle {
  passives: Record<string, PassiveRef>;
  moves: Record<string, MoveRef>;
  species: SpeciesRow[];
  elements: Record<string, ElementInfo>;
  /** Trust rank -> total FriendshipPoint required. */
  friendshipRanks: Record<string, number>;
  schema: SchemaColumn[];
}

// ---- Editable per-instance data (from the save) ----

export interface WorkSuit {
  name: string;
  /** Icon basename in /icons/work — active = `<icon>.png`, level 0 = `no_<icon>.png`. */
  icon: string;
  /** 0..10 in 1.0. */
  level: number;
}

export interface Pal {
  /** Stable GUID from the save; used only for app-owned metadata such as groups. */
  instanceId: string;
  species: string; // CodeName -> joins SpeciesRow + icon
  name: string; // nickname (editable)
  paldexNo: string;
  gender: Gender;
  elements: ElementName[];
  level: number;
  /** Raw total Exp from the save; preserved until a verified editor is exposed. */
  exp: number;
  alpha: boolean;
  lucky: boolean;
  condensation: number;
  ivs: { hp: number; shot: number; defense: number };
  soulRanks: { hp: number; attack: number; defense: number; craftSpeed: number };
  stats: {
    hp: number; hpMax: number; san: number; foodPct: number;
  };
  /** Trust rank 0..10 plus progress toward the next rank (0..1). */
  trust: { rank: number; progress: number };
  partnerSkill: { name: string; level: number; description: string; element?: ElementName };
  /** Passive **codes** (up to 4) — resolved against passives.json. */
  passives: string[];
  /** Equipped Active Skill (move) **codes** (up to 3) — resolved against moves.json. */
  activeSkills: string[];
  /** Explicit MasteredWaza entries. Natural learnset moves stay out of this list. */
  learnedMoves: string[];
  /** Bench / learnset move **codes** — resolved against moves.json. */
  benchMoves: string[];
  /** Species-supported work suitabilities, in canonical order. */
  workSuit: WorkSuit[];
}

/** Lightweight summary for the Global Box explorer tiles (storable pals only). */
export interface BoxPal {
  instanceId: string;
  slot: number;
  species: string;
  speciesName: string;
  nickname: string;
  /** Nickname when present, otherwise the localized species name. */
  name: string;
  gender: Gender;
  level: number;
  condensation: number;
  ivs: { hp: number; shot: number; defense: number };
  soulRanks: { hp: number; attack: number; defense: number; craftSpeed: number };
  elements: ElementName[];
  alpha?: boolean;
  lucky?: boolean;
  groups?: string[];
  workSuit: WorkSuit[];
  passives: string[];
  activeSkills: string[];
  moves: string[];
}
