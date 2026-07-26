import type { BoxPal } from "./types";
import { soulBonusPercent } from "./constants";
import { ref, resolveSpecies } from "./refdata.svelte";

export interface CombatStats {
  hp: number;
  attack: number;
  defense: number;
}

type CombatStatSource = Pick<
  BoxPal,
  "species" | "level" | "condensation" | "ivs" | "soulRanks" | "passives"
> & {
  alpha?: boolean;
};

type PassiveStat = "MaxHP" | "ShotAttack" | "Defense";

function passivePercent(codes: string[], stat: PassiveStat): number {
  return codes.reduce((total, code) => {
    const passive = ref.passives[code];
    if (!passive) return total;
    return total + passive.effects.reduce((sum, effect) => {
      const affectsPal =
        effect.target == null ||
        effect.target === "None" ||
        effect.target === "ToSelf" ||
        effect.target === "ToSelfAndTrainer";
      return effect.type === stat && affectsPal ? sum + (effect.value ?? 0) : sum;
    }, 0);
  }, 0);
}

function finishStat(
  base: number,
  soulRank: number,
  condensation: number,
  passiveBonusPercent: number,
): number {
  const trained = Math.floor(
    base * (1 + soulBonusPercent(soulRank) / 100) * (1 + condensation * 0.05),
  );
  return Math.max(1, Math.floor(trained * (1 + passiveBonusPercent / 100)));
}

/**
 * Calculate the stable combat stats visible for a boxed Pal.
 *
 * Mirrors PalEdit's in-game stat formula and uses only context carried by the
 * Global Palbox plus the cached reference bundle: species scaling, level, IVs,
 * Soul ranks, condensation, and self-targeted passive effects. Party, rider,
 * equipment, food, and server modifiers are intentionally excluded because
 * they are runtime context rather than properties of the boxed Pal.
 */
export function calculateCombatStats(pal: CombatStatSource): CombatStats | null {
  const species = resolveSpecies(pal.species);
  if (!species) return null;

  const hpIv = 1 + pal.ivs.hp * 0.003;
  const attackIv = 1 + pal.ivs.shot * 0.003;
  const defenseIv = 1 + pal.ivs.defense * 0.003;
  // Captured Alphas retain their HP bonus. Lucky is represented by its Rare
  // passive and does not receive a second implicit HP multiplier.
  const alphaHpRate = pal.alpha ? 1.2 : 1;

  const hpBase = Math.floor(
    500 + 5 * pal.level + species.scaling.hp * 0.5 * pal.level * hpIv * alphaHpRate,
  );
  const attackBase = Math.floor(
    100 + species.scaling.attack * 0.075 * pal.level * attackIv,
  );
  const defenseBase = Math.floor(
    50 + species.scaling.defense * 0.075 * pal.level * defenseIv,
  );

  return {
    hp: finishStat(
      hpBase,
      pal.soulRanks.hp,
      pal.condensation,
      passivePercent(pal.passives, "MaxHP"),
    ),
    attack: finishStat(
      attackBase,
      pal.soulRanks.attack,
      pal.condensation,
      passivePercent(pal.passives, "ShotAttack"),
    ),
    defense: finishStat(
      defenseBase,
      pal.soulRanks.defense,
      pal.condensation,
      passivePercent(pal.passives, "Defense"),
    ),
  };
}
