import type { Pal } from "./types";
import { WORK_SUITS } from "./constants";

// Real Incineram (CodeName "Baphomet"), from PalEdit's game data. Passives and
// moves are stored as **codes** and resolved for display via the reference tables
// (passives.json / moves.json). Stats are placeholder until the engine computes them.
const workLevels: Record<string, number> = {
  Kindling: 3, // EmitFlame
  Handiwork: 2, // Handcraft
  Mining: 3,
  Transporting: 2, // Transport
};

export const samplePal: Pal = {
  species: "Baphomet",
  name: "Incineram",
  paldexNo: "No. 095",
  gender: "Male",
  elements: ["Fire", "Dark"],
  level: 38,
  expToNext: 60121,
  expPct: 0.62,
  favorite: false,
  alpha: false,
  lucky: false,
  condensation: 2,
  ivs: { hp: 80, shot: 84, defense: 60 },
  soulRanks: { hp: 3, attack: 2, defense: 1, craftSpeed: 0 },
  stats: { hp: 2450, hpMax: 2787, attack: 452, defense: 318, workSpeed: 100, san: 92, foodPct: 0.74 },
  boosted: { attack: true },
  trust: { rank: 8, pct: 0.78 },
  partnerSkill: {
    name: "Flameclaw Hunter",
    level: 1,
    description:
      "When activated, attacks the targeted enemy with Hellfire Claw. When this Pal uses Hellfire Claw, it deals bonus damage.",
    element: "Fire",
  },
  // Passive CODES (resolved via passives.json): Legend, Ferocious, Workaholic.
  passives: ["Legend", "PAL_ALLAttack_up2", "PAL_Sanity_Down_2"],
  // Move CODES from Baphomet's real learnset (resolved via moves.json).
  activeSkills: ["FireBall", "Inferno", "DarkLegion"],
  benchMoves: ["FireBlast", "FireSeed", "FlareArrow", "Unique_Baphomet_SwallowKite", "DarkWave"],
  workSuit: WORK_SUITS.map((w) => ({ name: w.name, icon: w.icon, level: workLevels[w.name] ?? 0 })),
};
