import type { Pal } from "./types";
import { WORK_SUITS } from "./constants";

// Real Incineram (CodeName "Baphomet"), built from PalEdit's game data
// (resources/data/pals/Baphomet.json) so the card shows real structure + icon.
// Stats (hp/attack/…) are still placeholder numbers until the engine computes them
// from scaling + level + IV + souls + condensation.
const workLevels: Record<string, number> = {
  Kindling: 3, // EmitFlame
  Handiwork: 2, // Handcraft
  Mining: 3,
  Transporting: 2, // Transport
};

export const samplePal: Pal = {
  species: "Baphomet", // CodeName → icon T_Baphomet_icon_normal.png
  name: "Incineram", // display name (en-GB)
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
  passives: [
    { id: "Legend", name: "Legend", rating: 4, effects: "+20% Attack · +20% Defense · +15% Move Speed" },
    { id: "Ferocious", name: "Ferocious", rating: 3, effects: "+20% Attack" },
    { id: "Workaholic", name: "Workaholic", rating: 2, effects: "Slows SAN loss while working" },
  ],
  // Equipped (max 3) + bench, drawn from Baphomet's real Moveset. Powers are placeholder.
  activeSkills: [
    { id: "FireBall", name: "Fire Ball", element: "Fire", power: 45 },
    { id: "Inferno", name: "Inferno", element: "Fire", power: 150 },
    { id: "DarkLegion", name: "Dark Legion", element: "Dark", power: 150 },
  ],
  benchMoves: [
    { id: "FireBlast", name: "Fire Blast", element: "Fire", power: 30 },
    { id: "FireSeed", name: "Fire Seed", element: "Fire", power: 30 },
    { id: "FlareArrow", name: "Flare Arrow", element: "Fire", power: 55 },
    { id: "SwallowKite", name: "Swallow Kite", element: "Fire", power: 90 },
    { id: "DarkWave", name: "Dark Wave", element: "Dark", power: 110 },
  ],
  workSuit: WORK_SUITS.map((w) => ({ name: w.name, icon: w.icon, level: workLevels[w.name] ?? 0 })),
};
