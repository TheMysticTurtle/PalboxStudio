// Map between the engine's raw PalDto (save values) and the UI's display `Pal`,
// joining the species reference table. Only verified save-backed or computed
// values are surfaced by the UI.
import type { Pal, BoxPal, ElementName, Gender } from "./types";
import type { BoxTileDto, PalDto } from "./engine";
import { ref, resolveSpecies } from "./refdata.svelte";
import { WORK_SUITS } from "./constants";

const FALLBACK_TRUST_RANKS = [
  0, 6_000, 13_000, 21_000, 30_000, 40_000, 55_000, 80_000, 110_000, 150_000, 200_000,
];

function trustThreshold(rank: number): number {
  return ref.friendshipRanks[String(rank)] ?? FALLBACK_TRUST_RANKS[rank] ?? 0;
}

export function friendshipToTrust(points: number): Pal["trust"] {
  const clamped = Math.max(0, Math.min(trustThreshold(10), Math.round(points)));
  let rank = 0;
  for (let value = 1; value <= 10; value += 1) {
    if (clamped < trustThreshold(value)) break;
    rank = value;
  }
  if (rank >= 10) return { rank: 10, progress: 1 };
  const start = trustThreshold(rank);
  const end = trustThreshold(rank + 1);
  return { rank, progress: end > start ? (clamped - start) / (end - start) : 0 };
}

export function trustToFriendship(trust: Pal["trust"]): number {
  const rank = Math.max(0, Math.min(10, Math.round(trust.rank)));
  if (rank >= 10) return trustThreshold(10);
  const start = trustThreshold(rank);
  const end = trustThreshold(rank + 1);
  const progress = Math.max(0, Math.min(1, trust.progress));
  return Math.round(start + (end - start) * progress);
}

function calculateMaxHp(
  hpScaling: number,
  level: number,
  hpIv: number,
  hpSoulRank: number,
  condensation: number,
  boosted: boolean,
): number {
  const alphaRate = boosted ? 1.2 : 1;
  const base = Math.floor(
    500 + 5 * level + hpScaling * 0.5 * level * (1 + hpIv * 0.003) * alphaRate,
  );
  return Math.max(
    1,
    Math.floor(base * (1 + condensation * 0.05) * (1 + hpSoulRank * 0.03)),
  );
}

export function maxHpForPal(pal: Pal): number {
  const species = resolveSpecies(pal.species);
  // The plain browser preview has no Tauri bridge/reference bundle. Preserve the
  // mapped/sample maximum there instead of collapsing the slider to a base-only value.
  if (!species) return Math.max(1, pal.stats.hpMax);
  return calculateMaxHp(
    species.scaling.hp,
    pal.level,
    pal.ivs.hp,
    pal.soulRanks.hp,
    pal.condensation,
    pal.alpha || pal.lucky,
  );
}

export function dtoToPal(dto: PalDto): Pal {
  const sp = resolveSpecies(dto.characterId);
  const elements = (sp?.elements ?? []) as ElementName[];
  const displayName = dto.nickname || sp?.name || dto.characterId;
  const workBase = sp?.work ?? {};
  const equipped = dto.equippedMoves;
  const learned = dto.learnedMoves;
  const hp = Math.max(0, Math.round(dto.hp / 1000));
  const hpMax = calculateMaxHp(
    sp?.scaling.hp ?? 0,
    dto.level,
    dto.ivs.hp,
    dto.souls.hp,
    dto.condensation,
    dto.isAlpha || dto.isLucky,
  );

  return {
    instanceId: dto.instanceId,
    species: dto.characterId,
    name: displayName,
    paldexNo: sp && sp.deckIndex >= 0 ? `No. ${String(sp.deckIndex).padStart(3, "0")}` : "",
    gender: (dto.gender as Gender) || "Unknown",
    elements,
    level: dto.level,
    exp: dto.exp,
    alpha: dto.isAlpha,
    lucky: dto.isLucky,
    condensation: dto.condensation,
    ivs: { hp: dto.ivs.hp, shot: dto.ivs.shot, defense: dto.ivs.defense },
    soulRanks: {
      hp: dto.souls.hp,
      attack: dto.souls.attack,
      defense: dto.souls.defense,
      craftSpeed: dto.souls.craftSpeed,
    },
    stats: {
      hp,
      hpMax,
      san: Math.round(dto.sanity),
      foodPct: sp?.maxStomach ? Math.min(1, dto.food / sp.maxStomach) : 0.5,
    },
    trust: friendshipToTrust(dto.friendship),
    partnerSkill: {
      name: sp?.partnerSkill?.name ?? "—",
      level: Math.min(5, dto.condensation + 1),
      description: sp?.partnerSkill?.description || "No Partner Skill reference is available for this species.",
      element: sp?.partnerSkill?.element ?? elements[0],
    },
    passives: dto.passives,
    activeSkills: dto.equippedMoves,
    learnedMoves: learned,
    // Keep explicit mastered moves separate so natural moves are never written
    // into MasteredWaza merely because the UI displayed them.
    benchMoves: [...new Set([...learned, ...(sp?.moves ?? [])])].filter(
      (move) => !equipped.includes(move),
    ),
    workSuit: WORK_SUITS
      .filter((work) => (workBase[work.name] ?? 0) > 0 || (dto.work[work.name] ?? 0) !== 0)
      .map((work) => ({
        name: work.name,
        icon: work.icon,
        level: (workBase[work.name] ?? 0) + (dto.work[work.name] ?? 0),
      })),
  };
}

/** Change a loaded pal's species in place: rewrite the species code and re-derive
 *  the species-dependent display fields (elements, Paldex no., Partner Skill, bench
 *  learnset) from the reference table. Per-instance edits (level, IVs, souls,
 *  passives, equipped moves) are kept. A pal whose name still matches its old
 *  species (i.e. no custom nickname) follows to the new species name. Persisted on
 *  save: palToDto sends the new characterId, and the engine's set_species writes it. */
export function reSpecies(pal: Pal, code: string): void {
  const oldSp = resolveSpecies(pal.species);
  const sp = resolveSpecies(code);
  const oldWorkBase = oldSp?.work ?? {};
  const workBonus = Object.fromEntries(
    pal.workSuit.map((work) => [work.name, work.level - (oldWorkBase[work.name] ?? 0)]),
  );
  if (!pal.name || pal.name === oldSp?.name) pal.name = sp?.name ?? code;
  pal.species = code;
  pal.elements = (sp?.elements ?? []) as ElementName[];
  pal.paldexNo = sp && sp.deckIndex >= 0 ? `No. ${String(sp.deckIndex).padStart(3, "0")}` : "";
  pal.partnerSkill = {
    name: sp?.partnerSkill?.name ?? "—",
    level: pal.partnerSkill.level,
    description:
      sp?.partnerSkill?.description || "No Partner Skill reference is available for this species.",
    element: sp?.partnerSkill?.element ?? pal.elements[0],
  };
  // Bench = explicit mastered moves + the new natural learnset, excluding active.
  pal.benchMoves = [...new Set([...pal.learnedMoves, ...(sp?.moves ?? [])])].filter(
    (move) => !pal.activeSkills.includes(move),
  );
  const workBase = sp?.work ?? {};
  pal.workSuit = WORK_SUITS
    .filter((work) => (workBase[work.name] ?? 0) > 0 || (workBonus[work.name] ?? 0) !== 0)
    .map((work) => ({
      name: work.name,
      icon: work.icon,
      level: Math.max(0, Math.min(10, (workBase[work.name] ?? 0) + (workBonus[work.name] ?? 0))),
    }));
}

/** Build the editable DTO back from the display `Pal` for saving. */
export function palToDto(pal: Pal, slot: number): PalDto {
  const sp = resolveSpecies(pal.species);
  const workBase = sp?.work ?? {};
  const work: Record<string, number> = {};
  for (const w of pal.workSuit) {
    const addRank = w.level - (workBase[w.name] ?? 0);
    if (addRank !== 0) work[w.name] = addRank; // only non-zero bonuses
  }
  return {
    slot,
    instanceId: pal.instanceId,
    characterId: pal.species,
    nickname: pal.name,
    gender: pal.gender,
    level: pal.level,
    exp: pal.exp,
    condensation: pal.condensation,
    souls: {
      hp: pal.soulRanks.hp,
      attack: pal.soulRanks.attack,
      defense: pal.soulRanks.defense,
      craftSpeed: pal.soulRanks.craftSpeed,
    },
    ivs: { hp: pal.ivs.hp, shot: pal.ivs.shot, defense: pal.ivs.defense },
    work,
    passives: pal.passives,
    equippedMoves: pal.activeSkills,
    learnedMoves: pal.learnedMoves,
    isLucky: pal.lucky,
    isAlpha: pal.alpha,
    hp: Math.round(Math.max(0, pal.stats.hp) * 1000),
    sanity: pal.stats.san,
    food: Math.max(0, Math.min(1, pal.stats.foodPct)) * (sp?.maxStomach || 300),
    friendship: trustToFriendship(pal.trust),
  };
}

/** Join a lightweight engine tile to the in-memory species reference table.
 * `resolveSpecies` strips the BOSS_ storage prefix, so Alpha/Lucky tiles always
 * show the real species name, elements, and portrait instead of a code name. */
export function tileDtoToBoxPal(tile: BoxTileDto, groups: string[] = []): BoxPal {
  const sp = resolveSpecies(tile.characterId);
  return {
    instanceId: tile.instanceId,
    slot: tile.slot,
    species: tile.characterId,
    name: sp?.name ?? tile.characterId,
    level: tile.level,
    elements: (sp?.elements ?? []) as ElementName[],
    alpha: tile.isAlpha,
    lucky: tile.isLucky,
    groups,
    passives: tile.passives,
    moves: [...new Set([...tile.equippedMoves, ...tile.learnedMoves])],
  };
}

/** Live tile projection for the Pal currently open on the main card. */
export function palToBoxPal(pal: Pal, slot: number, groups: string[] = []): BoxPal {
  const sp = resolveSpecies(pal.species);
  return {
    instanceId: pal.instanceId,
    slot,
    species: pal.species,
    name: sp?.name ?? pal.species,
    level: pal.level,
    elements: pal.elements,
    alpha: pal.alpha,
    lucky: pal.lucky,
    groups,
    passives: pal.passives,
    moves: [...new Set([...pal.activeSkills, ...pal.learnedMoves])],
  };
}
