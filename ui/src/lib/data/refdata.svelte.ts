// Static reference tables, loaded once from the engine's SQLite reference DB and
// shared. Components resolve codes -> info through here so display always matches
// the game data and is never hand-written onto a pal.
import type { PassiveRef, MoveRef, SpeciesRow, ElementInfo, SchemaColumn } from "./types";
import { getReferenceData } from "./engine";

interface RefData {
  loaded: boolean;
  passives: Record<string, PassiveRef>;
  moves: Record<string, MoveRef>;
  species: SpeciesRow[];
  speciesByCode: Record<string, SpeciesRow>;
  speciesAliases: Record<string, string>;
  elements: Record<string, ElementInfo>;
  friendshipRanks: Record<string, number>;
  schema: SchemaColumn[];
}

export const ref = $state<RefData>({
  loaded: false,
  passives: {},
  moves: {},
  species: [],
  speciesByCode: {},
  speciesAliases: {},
  elements: {},
  friendshipRanks: {},
  schema: [],
});

let started = false;

/** Load the reference bundle from the engine (SQLite, in-memory cached). Idempotent.
 *  Only resolves inside the app; a plain browser has no engine bridge. */
export async function loadRefData(): Promise<void> {
  if (started) return;
  started = true;
  try {
    const bundle = await getReferenceData();
    ref.passives = bundle.passives;
    ref.moves = bundle.moves;
    ref.species = bundle.species;
    ref.speciesByCode = Object.fromEntries(bundle.species.map((s) => [s.code, s]));
    ref.speciesAliases = bundle.speciesAliases;
    ref.elements = bundle.elements;
    ref.friendshipRanks = bundle.friendshipRanks;
    ref.schema = bundle.schema;
    ref.loaded = true;
  } catch (e) {
    started = false; // allow a retry
    console.error("Failed to load reference data from the engine", e);
  }
}

// Resolvers — read `ref` so they stay reactive inside $derived once data loads.
export const resolvePassive = (code: string): PassiveRef | undefined => ref.passives[code];
export const resolveMove = (code: string): MoveRef | undefined => ref.moves[code];

/** Normalize a save/encounter code to the canonical owned-species reference row. */
export const baseSpeciesCode = (code: string): string => {
  const withoutVariant = code.replace(/^BOSS_/i, "");
  return ref.speciesAliases[withoutVariant] ?? withoutVariant;
};
export const resolveSpecies = (code: string): SpeciesRow | undefined =>
  ref.speciesByCode[baseSpeciesCode(code)];
export const speciesDisplayName = (code: string): string => resolveSpecies(code)?.name ?? code;
