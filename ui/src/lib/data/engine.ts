// Bridge to the Rust engine (Tauri commands). Only works inside the app; in a
// plain browser these reject (caught by callers).
import { invoke } from "@tauri-apps/api/core";
import type { ReferenceBundle } from "./types";

export interface PalDtoSouls { hp: number; attack: number; defense: number; craftSpeed: number }
export interface PalDtoIvs { hp: number; shot: number; defense: number }

export interface TrustInput { rank: number; progress: number }

/** Engine-owned semantic edit input. No save-only encodings cross this boundary. */
export interface PalDto {
  slot: number;
  instanceId: string;
  characterId: string;
  nickname: string | null;
  gender: string;
  level: number;
  exp: number;
  condensation: number;
  isAwakened: boolean;
  souls: PalDtoSouls;
  ivs: PalDtoIvs;
  /** Internal Work Suitability code -> desired effective total. */
  work: Record<string, number>;
  passives: string[];
  equippedMoves: string[];
  learnedMoves: string[];
  isLucky: boolean;
  isAlpha: boolean;
  /** User-facing whole HP. */
  hp: number;
  sanity: number;
  foodPercent: number;
  trust: TrustInput;
}

export interface WorkSuitabilityView {
  code: string;
  name: string;
  icon: string;
  baseLevel: number;
  bonusLevel: number;
  totalLevel: number;
  available: boolean;
}

export interface PalProjection {
  speciesName: string;
  elements: string[];
  maxStomach: number;
  work: WorkSuitabilityView[];
  stats: { hp: number; attack: number; defense: number };
  trust: {
    rank: number;
    minRank: number;
    maxRank: number;
    progress: number;
    points: number;
    rankStartPoints: number;
    nextRankPoints: number;
  };
  exp: {
    points: number;
    levelStartPoints: number;
    nextLevelPoints: number;
    toNextLevel: number;
    progress: number;
  };
  partnerSkill: {
    name: string;
    description: string;
    category: string | null;
    element: string | null;
    gearName: string | null;
    technologyLevel: number | null;
    level: number;
    activeRank: { rank: number; valueText: string; valueNumber: number | null } | null;
  } | null;
}

export interface PalView {
  editable: PalDto;
  projection: PalProjection;
}

export interface BoxTileDto {
  slot: number;
  instanceId: string;
  characterId: string;
  nickname: string | null;
  gender: string;
  level: number;
  condensation: number;
  ivs: PalDtoIvs;
  souls: PalDtoSouls;
  /** Internal Work Suitability code -> per-instance AddRank bonus (engine-only detail). */
  work: Record<string, number>;
  isLucky: boolean;
  isAlpha: boolean;
  passives: string[];
  equippedMoves: string[];
  learnedMoves: string[];
  projection: PalProjection | null;
}

export interface OpenResult {
  path: string;
  slotCount: number;
  pals: BoxTileDto[];
}

export interface BoxSessionStatus {
  dirty: boolean;
  sourceState: "unchanged" | "changed" | "unavailable";
  detail: string | null;
}

export interface PassiveOption {
  code: string;
  name: string;
  description: string;
  rating: number;
  disabled: boolean;
  availableNormalPal: boolean;
}

export interface PassivePreset {
  id: number;
  name: string;
  passiveCodes: string[];
}

export interface UserGroup {
  id: number;
  name: string;
}

export interface PalGroupMembership {
  instanceId: string;
  groupIds: number[];
}

/** Result of a box add/clone/delete: refreshed tiles + the slot to select. */
export interface BoxMutation {
  pals: BoxTileDto[];
  slot: number | null;
}

export const openBox = (path: string) => invoke<OpenResult>("open_box", { path });
export const getPal = (slot: number) => invoke<PalView>("get_pal", { slot });
export const getBoxSessionStatus = () =>
  invoke<BoxSessionStatus>("box_session_status");
export const updatePal = (dto: PalDto) => invoke<PalView>("update_pal", { dto });

/** Add a new pal (default: the turtle CubeTurtle) to a free slot. */
export const addBoxPal = (species: string | null = null) =>
  invoke<BoxMutation>("add_box_pal", { species });
/** Deep-copy the pal at `slot` into a free slot with a fresh identity. */
export const cloneBoxPal = (slot: number) => invoke<BoxMutation>("clone_box_pal", { slot });
/** Remove the pal at `slot`, restoring a vacancy. */
export const deleteBoxPal = (slot: number) => invoke<BoxMutation>("delete_box_pal", { slot });
/** Backup the original + atomic-write the edited box. Returns the backup path. */
export const saveBox = () => invoke<string>("save_box");

export const getReferenceData = () =>
  invoke<ReferenceBundle>("get_reference_data");

export interface AppPreferences {
  lastBoxPath: string;
  autoReopen: boolean;
}

export const getAppPreferences = () =>
  invoke<AppPreferences>("get_app_preferences");

export const saveAppPreferences = (preferences: AppPreferences) =>
  invoke<AppPreferences>("save_app_preferences", { preferences });

export const listPassiveOptions = (
  search = "",
  includeDisabled = false,
  includeUnavailable = false,
) => invoke<PassiveOption[]>("list_passive_options", {
  search,
  includeDisabled,
  includeUnavailable,
});

export const listPassivePresets = () =>
  invoke<PassivePreset[]>("list_passive_presets");

export const savePassivePreset = (
  name: string,
  passiveCodes: string[],
  id: number | null = null,
) => invoke<PassivePreset>("save_passive_preset", { id, name, passiveCodes });

export const deletePassivePreset = (id: number) =>
  invoke<boolean>("delete_passive_preset", { id });

/** Applies the preset to the in-memory Pal at slot; persistence still requires saveBox(). */
export const applyPassivePreset = (slot: number, presetId: number) =>
  invoke<PalView>("apply_passive_preset", { slot, presetId });

export const listGroups = () => invoke<UserGroup[]>("list_groups");
export const createGroup = (name: string) =>
  invoke<UserGroup>("create_group", { name });
export const renameGroup = (id: number, name: string) =>
  invoke<UserGroup>("rename_group", { id, name });
export const deleteGroup = (id: number) =>
  invoke<boolean>("delete_group", { id });
export const listGroupMemberships = () =>
  invoke<PalGroupMembership[]>("list_group_memberships");
export const setPalGroups = (instanceId: string, groupIds: number[]) =>
  invoke<number[]>("set_pal_groups", { instanceId, groupIds });
