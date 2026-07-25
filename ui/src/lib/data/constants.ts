import type { ElementName } from "./types";

// Verified Palworld 1.0 editing limits — mirrors `palbox-core::limits`
// (docs/SPECS-1.0.md). TODO: source these from the core via a Tauri command so
// there's a single source of truth across the boundary.
export const LIMITS = {
  levelMin: 1,
  levelMax: 80,
  ivMin: 0,
  ivMax: 100,
  workSuitMin: 0,
  workSuitMax: 10,
  soulsMin: 0,
  soulsMax: 10,
  condensationMin: 0,
  condensationMax: 4,
  equippedMovesMax: 3,
  passivesMax: 4,
} as const;

// Element codename note (SPECS-1.0.md): the save uses Normal/Leaf/Earth/Electricity;
// we always display the official UI names below.
export const ELEMENT_COLOR: Record<ElementName, string> = {
  Neutral: "var(--el-neutral)",
  Fire: "var(--el-fire)",
  Water: "var(--el-water)",
  Grass: "var(--el-grass)",
  Electric: "var(--el-electric)",
  Ice: "var(--el-ice)",
  Ground: "var(--el-ground)",
  Dark: "var(--el-dark)",
  Dragon: "var(--el-dragon)",
};

// The 13 Work Suitabilities in canonical order (official UI name + icon basename in
// /icons/work). NOTE: 13, not 12 — the data + icons include Crude Oil Extraction. See
// docs/DATA-AND-ASSETS.md for the internal-codename mapping.
export const WORK_SUITS = [
  { name: "Kindling", icon: "kindling" },
  { name: "Watering", icon: "watering" },
  { name: "Planting", icon: "planting" },
  { name: "Generating Electricity", icon: "generating" },
  { name: "Handiwork", icon: "handiwork" },
  { name: "Gathering", icon: "gathering" },
  { name: "Lumbering", icon: "deforesting" },
  { name: "Mining", icon: "mining" },
  { name: "Crude Oil Extraction", icon: "extracting" },
  { name: "Medicine Production", icon: "production" },
  { name: "Cooling", icon: "cooling" },
  { name: "Transporting", icon: "transporting" },
  { name: "Farming", icon: "farming" },
] as const;

/** Rating (-3..5) -> chip color token. */
export function ratingColor(rating: number): string {
  if (rating >= 1) return "var(--rate-good)";
  if (rating <= -1) return "var(--rate-bad)";
  return "var(--rate-ok)";
}

/** Positive ratings show gold; the strongest (>=4) show teal, per the in-game chips. */
export function ratingTone(rating: number): string {
  if (rating >= 4) return "var(--rate-good)";
  if (rating >= 1) return "var(--rate-ok)";
  return "var(--rate-bad)";
}
