// Map between the engine's raw PalDto (save values) and the UI's display `Pal`,
// joining the species reference table. Combat-stat numbers are placeholders until
// the stat formula lands; every editable field is faithful.
import type { Pal, ElementName, Gender } from "./types";
import type { PalDto } from "./engine";
import { ref } from "./refdata.svelte";
import { WORK_SUITS } from "./constants";

export function dtoToPal(dto: PalDto): Pal {
  const sp = ref.speciesByCode[dto.characterId];
  const elements = (sp?.elements ?? []) as ElementName[];
  const displayName = dto.nickname || sp?.name || dto.characterId;
  const workBase = sp?.work ?? {};
  const equipped = dto.equippedMoves;

  return {
    species: dto.characterId,
    name: displayName,
    paldexNo: sp && sp.deckIndex >= 0 ? `No. ${String(sp.deckIndex).padStart(3, "0")}` : "",
    gender: (dto.gender as Gender) || "Unknown",
    elements,
    level: dto.level,
    expToNext: dto.exp,
    expPct: 0.5, // placeholder until the exp table is wired
    favorite: false,
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
      hp: Math.round(dto.hp / 1000),
      hpMax: Math.round(dto.hp / 1000), // placeholder max (bar full) until formula
      attack: sp?.scaling.attack ?? 0, // placeholder = base scaling
      defense: sp?.scaling.defense ?? 0,
      workSpeed: 100,
      san: Math.round(dto.sanity),
      foodPct: sp?.maxStomach ? Math.min(1, dto.food / sp.maxStomach) : 0.5,
    },
    boosted: {},
    trust: { rank: Math.min(20, Math.floor(dto.friendship / 100)), pct: Math.min(1, dto.friendship / 2000) },
    partnerSkill: {
      name: sp?.partnerSkill?.name ?? "—",
      level: Math.min(5, dto.condensation + 1),
      description: sp?.partnerSkill?.description || "No Partner Skill reference is available for this species.",
      element: sp?.partnerSkill?.element ?? elements[0],
    },
    passives: dto.passives,
    activeSkills: dto.equippedMoves,
    // Bench = the species learnset minus what's equipped (real available moves).
    benchMoves: (sp?.moves ?? []).filter((m) => !equipped.includes(m)),
    workSuit: WORK_SUITS.map((w) => ({
      name: w.name,
      icon: w.icon,
      level: (workBase[w.name] ?? 0) + (dto.work[w.name] ?? 0),
    })),
  };
}

/** Build the editable DTO back from the display `Pal` for saving. */
export function palToDto(pal: Pal, slot: number): PalDto {
  const sp = ref.speciesByCode[pal.species];
  const workBase = sp?.work ?? {};
  const work: Record<string, number> = {};
  for (const w of pal.workSuit) {
    const addRank = w.level - (workBase[w.name] ?? 0);
    if (addRank !== 0) work[w.name] = addRank; // only non-zero bonuses
  }
  return {
    slot,
    characterId: pal.species,
    nickname: pal.name,
    gender: pal.gender,
    level: pal.level,
    exp: pal.expToNext,
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
    learnedMoves: [], // don't touch MasteredWaza
    isLucky: pal.lucky,
    isAlpha: pal.alpha,
    hp: 0,
    sanity: pal.stats.san,
    food: 0,
    friendship: 0,
  };
}
