// Thin presentation mapping over engine-owned semantic inputs and projections.
// No save encoding, game formula, Work base/bonus, Trust, EXP, or progression
// calculation belongs in this layer.
import type { Pal, BoxPal, ElementName, Gender } from "./types";
import type { BoxTileDto, PalDto, PalView } from "./engine";
import { resolveSpecies } from "./refdata.svelte";

export function maxHpForPal(pal: Pal): number {
  return Math.max(1, pal.stats.hpMax);
}

export function dtoToPal(view: PalView): Pal {
  const dto = view.editable;
  const projection = view.projection;
  const species = resolveSpecies(dto.characterId);
  const elements = projection.elements as ElementName[];
  const displayName = dto.nickname || projection.speciesName || dto.characterId;
  const equipped = dto.equippedMoves;
  const learned = dto.learnedMoves;
  const activeRank = projection.partnerSkill?.activeRank;

  return {
    instanceId: dto.instanceId,
    species: dto.characterId,
    name: displayName,
    paldexNo:
      species && species.deckIndex >= 0
        ? `No. ${String(species.deckIndex).padStart(3, "0")}`
        : "",
    gender: (dto.gender as Gender) || "Unknown",
    elements,
    level: dto.level,
    exp: dto.exp,
    alpha: dto.isAlpha,
    lucky: dto.isLucky,
    condensation: dto.condensation,
    awakened: dto.isAwakened,
    ivs: { hp: dto.ivs.hp, shot: dto.ivs.shot, defense: dto.ivs.defense },
    soulRanks: {
      hp: dto.souls.hp,
      attack: dto.souls.attack,
      defense: dto.souls.defense,
      craftSpeed: dto.souls.craftSpeed,
    },
    stats: {
      hp: dto.hp,
      hpMax: projection.stats.hp,
      attack: projection.stats.attack,
      defense: projection.stats.defense,
      san: Math.round(dto.sanity),
      foodPct: dto.foodPercent,
    },
    trust: {
      rank: projection.trust.rank,
      minRank: projection.trust.minRank,
      maxRank: projection.trust.maxRank,
      progress: projection.trust.progress,
    },
    partnerSkill: {
      name: projection.partnerSkill?.name ?? "—",
      level: projection.partnerSkill?.level ?? 0,
      description:
        projection.partnerSkill?.description
        || "No Partner Skill reference is available for this species.",
      element:
        (projection.partnerSkill?.element as ElementName | null) ?? elements[0],
      rankEffect: activeRank?.valueText,
    },
    passives: dto.passives,
    activeSkills: equipped,
    learnedMoves: learned,
    // Explicitly mastered moves and the DB-backed natural learnset are display
    // inputs only; natural moves are never written into MasteredWaza.
    benchMoves: [...new Set([...learned, ...(species?.moves ?? [])])].filter(
      (move) => !equipped.includes(move),
    ),
    workSuit: projection.work
      .filter((work) => work.available)
      .map((work) => ({
        code: work.code,
        name: work.name,
        icon: work.icon,
        level: work.totalLevel,
      })),
  };
}

/** Build semantic edit input. The engine translates these user-facing values
 * into Palworld's save-only encodings and returns a fresh canonical view. */
export function palToDto(pal: Pal, slot: number): PalDto {
  const speciesName = resolveSpecies(pal.species)?.name ?? pal.species;
  const nickname = pal.name.trim();
  return {
    slot,
    instanceId: pal.instanceId,
    characterId: pal.species,
    nickname: nickname && nickname !== speciesName ? nickname : null,
    gender: pal.gender,
    level: pal.level,
    exp: pal.exp,
    condensation: pal.condensation,
    isAwakened: pal.awakened,
    souls: {
      hp: pal.soulRanks.hp,
      attack: pal.soulRanks.attack,
      defense: pal.soulRanks.defense,
      craftSpeed: pal.soulRanks.craftSpeed,
    },
    ivs: { hp: pal.ivs.hp, shot: pal.ivs.shot, defense: pal.ivs.defense },
    work: Object.fromEntries(
      pal.workSuit.map((work) => [work.code, work.level]),
    ),
    passives: pal.passives,
    equippedMoves: pal.activeSkills,
    learnedMoves: pal.learnedMoves,
    isLucky: pal.lucky,
    isAlpha: pal.alpha,
    hp: Math.round(Math.max(0, pal.stats.hp)),
    sanity: pal.stats.san,
    foodPercent: Math.max(0, Math.min(1, pal.stats.foodPct)),
    trust: {
      rank: pal.trust.rank,
      progress: pal.trust.progress,
    },
  };
}

/** Engine-projected lightweight row for either Global Palbox density. */
export function tileDtoToBoxPal(
  tile: BoxTileDto,
  groups: string[] = [],
): BoxPal {
  const species = resolveSpecies(tile.characterId);
  const projection = tile.projection;
  const speciesName =
    projection?.speciesName ?? species?.name ?? tile.characterId;
  const nickname = tile.nickname?.trim() ?? "";
  return {
    instanceId: tile.instanceId,
    slot: tile.slot,
    species: tile.characterId,
    speciesName,
    nickname,
    name: nickname || speciesName,
    gender: (tile.gender as Gender) || "Unknown",
    level: tile.level,
    condensation: tile.condensation,
    ivs: { ...tile.ivs },
    soulRanks: { ...tile.souls },
    elements: (projection?.elements ?? species?.elements ?? []) as ElementName[],
    alpha: tile.isAlpha,
    lucky: tile.isLucky,
    groups,
    stats: projection?.stats ?? { hp: 0, attack: 0, defense: 0 },
    workSuit:
      projection?.work
        .filter((work) => work.totalLevel > 0)
        .map((work) => ({
          code: work.code,
          name: work.name,
          icon: work.icon,
          level: work.totalLevel,
        })) ?? [],
    passives: tile.passives,
    activeSkills: tile.equippedMoves,
    moves: [...new Set([...tile.equippedMoves, ...tile.learnedMoves])],
  };
}

/** Live card row for the selected Pal, already projected by the engine. */
export function palToBoxPal(
  pal: Pal,
  slot: number,
  groups: string[] = [],
): BoxPal {
  const species = resolveSpecies(pal.species);
  const speciesName = species?.name ?? pal.species;
  const nickname = pal.name !== speciesName ? pal.name : "";
  return {
    instanceId: pal.instanceId,
    slot,
    species: pal.species,
    speciesName,
    nickname,
    name: nickname || speciesName,
    gender: pal.gender,
    level: pal.level,
    condensation: pal.condensation,
    ivs: { ...pal.ivs },
    soulRanks: { ...pal.soulRanks },
    elements: pal.elements,
    alpha: pal.alpha,
    lucky: pal.lucky,
    groups,
    stats: {
      hp: pal.stats.hpMax,
      attack: pal.stats.attack,
      defense: pal.stats.defense,
    },
    workSuit: pal.workSuit,
    passives: pal.passives,
    activeSkills: pal.activeSkills,
    moves: [...new Set([...pal.activeSkills, ...pal.learnedMoves])],
  };
}
