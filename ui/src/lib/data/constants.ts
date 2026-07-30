import { ref } from "./refdata.svelte";

// Compatibility-shaped accessors over the engine's cached, DB-backed limits.
// Components never own a second copy of these values.
export const LIMITS = {
  get levelMin() { return ref.limits.levelMin; },
  get levelMax() { return ref.limits.levelMax; },
  get ivMin() { return ref.limits.ivMin; },
  get ivMax() { return ref.limits.ivMax; },
  get workSuitMin() { return ref.limits.workSuitabilityMin; },
  get workSuitMax() { return ref.limits.workSuitabilityMax; },
  get soulsMin() { return ref.limits.soulRankMin; },
  get soulsMax() { return ref.limits.soulRankMax; },
  get condensationMin() { return ref.limits.condensationMin; },
  get condensationMax() { return ref.limits.condensationMax; },
  get equippedMovesMax() { return ref.limits.equippedMovesMax; },
  get passivesMax() { return ref.limits.passivesMax; },
};

/**
 * Default keyboard/mouse controls for the three active-skill slots while
 * mounted. Palworld allows rebinding, so the UI presents these as defaults.
 */
export const ACTIVE_SKILL_DEFAULT_CONTROLS = [
  { short: "RMB", label: "Right Mouse Button", action: "Mounted Skill 1" },
  { short: "E", label: "E", action: "Mounted Skill 2" },
  { short: "C", label: "C", action: "Mounted Skill 3" },
] as const;

export function activeSkillDefaultControl(index: number) {
  return ACTIVE_SKILL_DEFAULT_CONTROLS[index] ?? {
    short: String(index + 1),
    label: `Skill slot ${index + 1}`,
    action: `Mounted Skill ${index + 1}`,
  };
}

export function soulBonusPercent(rank: number): number {
  return Math.max(LIMITS.soulsMin, Math.min(LIMITS.soulsMax, rank))
    * ref.calculationRules.soulBonusPercentPerRank;
}

/** Presentation color for a DB-authored passive rating. */
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
