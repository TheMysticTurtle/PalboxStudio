// Static reference tables (generated into ui/static/data/) loaded once and shared.
// Components resolve codes -> info through here so display always matches the game
// data and is never hand-written onto a pal.
import type { PassiveRef, MoveRef, SpeciesRow, ElementInfo, SchemaColumn } from "./types";

interface RefData {
  loaded: boolean;
  passives: Record<string, PassiveRef>;
  moves: Record<string, MoveRef>;
  species: SpeciesRow[];
  speciesByCode: Record<string, SpeciesRow>;
  elements: Record<string, ElementInfo>;
  schema: SchemaColumn[];
}

export const ref = $state<RefData>({
  loaded: false,
  passives: {},
  moves: {},
  species: [],
  speciesByCode: {},
  elements: {},
  schema: [],
});

let started = false;

/** Fetch all reference tables from /data (idempotent). Call once at app start. */
export async function loadRefData(): Promise<void> {
  if (started) return;
  started = true;
  try {
    const get = (f: string) => fetch(`/data/${f}`).then((r) => r.json());
    const [passives, moves, species, elements, schema] = await Promise.all([
      get("passives.json"),
      get("moves.json"),
      get("species.json"),
      get("elements.json"),
      get("schema.json"),
    ]);
    ref.passives = passives.passives;
    ref.moves = moves.moves;
    ref.species = species.species;
    ref.speciesByCode = Object.fromEntries((species.species as SpeciesRow[]).map((s) => [s.code, s]));
    ref.elements = elements.elements;
    ref.schema = schema.schema;
    ref.loaded = true;
  } catch (e) {
    started = false; // allow a retry
    console.error("Failed to load reference data", e);
  }
}

// Resolvers — read `ref` so they stay reactive inside $derived once data loads.
export const resolvePassive = (code: string): PassiveRef | undefined => ref.passives[code];
export const resolveMove = (code: string): MoveRef | undefined => ref.moves[code];
export const resolveSpecies = (code: string): SpeciesRow | undefined => ref.speciesByCode[code];
