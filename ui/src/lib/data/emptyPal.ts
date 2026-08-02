import type { Pal } from "./types";

/**
 * Read-only rendering value for the normal card when no save Pal is selected.
 * It never enters the box store and therefore cannot be flushed to a save.
 */
export const EMPTY_PAL: Pal = {
  instanceId: "",
  species: "",
  name: "",
  paldexNo: "No. 000",
  gender: "Unknown",
  elements: [],
  level: 0,
  exp: 0,
  alpha: false,
  lucky: false,
  condensation: 0,
  awakened: false,
  ivs: { hp: 0, shot: 0, defense: 0 },
  soulRanks: { hp: 0, attack: 0, defense: 0, craftSpeed: 0 },
  stats: { hp: 0, hpMax: 0, attack: 0, defense: 0, san: 0, foodPct: 0 },
  trust: { rank: 0, minRank: 0, maxRank: 0, progress: 0 },
  partnerSkill: { name: "", level: 0, description: "" },
  passives: [],
  activeSkills: [],
  learnedMoves: [],
  benchMoves: [],
  workSuit: [],
};
