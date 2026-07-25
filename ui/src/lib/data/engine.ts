// Bridge to the Rust engine (Tauri commands). Only works inside the app; in a
// plain browser these reject (caught by callers).
import { invoke } from "@tauri-apps/api/core";
import type { ReferenceBundle } from "./types";

export interface PalDtoSouls { hp: number; attack: number; defense: number; craftSpeed: number }
export interface PalDtoIvs { hp: number; shot: number; defense: number }

/** Mirrors palbox_core::pal::PalDto (raw editable save values). */
export interface PalDto {
  slot: number;
  characterId: string;
  nickname: string | null;
  gender: string;
  level: number;
  exp: number;
  condensation: number;
  souls: PalDtoSouls;
  ivs: PalDtoIvs;
  /** official work name -> AddRank bonus */
  work: Record<string, number>;
  passives: string[];
  equippedMoves: string[];
  learnedMoves: string[];
  isLucky: boolean;
  isAlpha: boolean;
  hp: number;
  sanity: number;
  food: number;
  friendship: number;
}

export interface BoxTileDto {
  slot: number;
  characterId: string;
  level: number;
  isLucky: boolean;
  isAlpha: boolean;
}

export interface OpenResult {
  path: string;
  slotCount: number;
  pals: BoxTileDto[];
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

export const openBox = (path: string) => invoke<OpenResult>("open_box", { path });
export const getPal = (slot: number) => invoke<PalDto>("get_pal", { slot });
export const updatePal = (dto: PalDto) => invoke<PalDto>("update_pal", { dto });
/** Backup the original + atomic-write the edited box. Returns the backup path. */
export const saveBox = () => invoke<string>("save_box");

export const getReferenceData = () =>
  invoke<ReferenceBundle>("get_reference_data");

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
  invoke<PalDto>("apply_passive_preset", { slot, presetId });
