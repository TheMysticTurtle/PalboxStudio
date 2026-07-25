import type { Pal } from "./types";
import { WORK_SUITS } from "./constants";

// Placeholder pal so the card has something real to render before save-loading
// exists. Modeled loosely on the in-game Incineram reference (Fire/Dark).
const workSuitLevels: Record<string, number> = {
  Kindling: 4,
  Handiwork: 2,
  Gathering: 3,
  Mining: 1,
  Transporting: 2,
};

export const samplePal: Pal = {
  species: "Incineram",
  name: "Incineram",
  paldexNo: "No. 099",
  gender: "Male",
  elements: ["Fire", "Dark"],
  level: 38,
  expToNext: 60121,
  expPct: 0.62,
  favorite: false,
  alpha: true,
  lucky: true,
  condensation: 2,
  souls: 3,
  stats: { hp: 2450, hpMax: 2787, attack: 452, defense: 318, workSpeed: 100, san: 92, foodPct: 0.74 },
  boosted: { attack: true, workSpeed: true },
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
  activeSkills: [
    { id: "FireBall", name: "Fire Ball", element: "Fire", power: 45 },
    { id: "DarkLaser", name: "Dark Laser", element: "Dark", power: 450 },
    { id: "PowerBomb", name: "Power Bomb", element: "Neutral", power: 90 },
  ],
  benchMoves: [
    { id: "FlareStorm", name: "Flare Storm", element: "Fire", power: 90 },
    { id: "SpiritFire", name: "Spirit Fire", element: "Fire", power: 55 },
    { id: "IcicleThrow", name: "Icicle Throw", element: "Ice", power: 40 },
    { id: "SandBlast", name: "Sand Blast", element: "Ground", power: 35 },
    { id: "TriLightning", name: "Tri-Lightning", element: "Electric", power: 90 },
    { id: "GrassTornado", name: "Grass Tornado", element: "Grass", power: 75 },
  ],
  workSuit: WORK_SUITS.map((name) => ({ name, level: workSuitLevels[name] ?? 0 })),
};
