// The open Global Palbox session (real engine data). Holds the box tiles, the
// selected slot, and the currently-loaded editable pal. Edits mutate `box.pal`;
// they're flushed to the in-memory box (updatePal) on slot-switch and on save.
import {
  openBox,
  getPal,
  getBoxSessionStatus,
  updatePal,
  saveBox,
  addBoxPal,
  cloneBoxPal,
  deleteBoxPal,
  type BoxTileDto,
} from "$lib/data/engine";
import { dtoToPal, palToDto } from "$lib/data/mapper";
import {
  classifySourceConflict,
  type BoxConflictKind,
} from "$lib/data/sourceMonitor";
import type { Pal } from "$lib/data/types";
import { ref } from "$lib/data/refdata.svelte";
import {
  boxPreferences,
  rememberBoxPath,
  setAutoReopen,
  setVitalMaxPreference,
  type VitalMaxPreference,
} from "$lib/stores/boxPreferences.svelte";

type SourceState = "idle" | "unchanged" | "changed" | "unavailable";

export const box = $state({
  open: false,
  path: "",
  slotCount: 0,
  tiles: [] as BoxTileDto[],
  selectedSlot: -1,
  pal: null as Pal | null,
  loading: false,
  error: "",
  saveMsg: "",
  lastBackupPath: "",
  dirty: false,
  sourceState: "idle" as SourceState,
  sourceDetail: "",
  conflict: "" as BoxConflictKind,
  lastSavedAt: 0,
  /** Transient hint after an add/clone/delete (e.g. the in-game "drag" caveat). */
  hint: "",
});

let selectedBaseline = "";
let monitorInFlight = false;
let unavailableStreak = 0;

function applyVitalMaximum(pal: Pal, preference: VitalMaxPreference) {
  if (preference === "maxHp") pal.stats.hp = pal.stats.hpMax;
  if (preference === "maxSanity") pal.stats.san = ref.limits.sanityMax;
  if (preference === "maxFood") pal.stats.foodPct = 1;
  if (preference === "maxTrust") {
    pal.trust.rank = pal.trust.maxRank;
    pal.trust.progress = 1;
  }
}

export function applySelectedVitalMaxPreferences() {
  if (!box.pal) return;
  for (const preference of [
    "maxHp",
    "maxSanity",
    "maxFood",
    "maxTrust",
  ] as const) {
    if (boxPreferences[preference]) applyVitalMaximum(box.pal, preference);
  }
}

export async function setSelectedVitalMax(
  preference: VitalMaxPreference,
  enabled: boolean,
): Promise<boolean> {
  if (!(await setVitalMaxPreference(preference, enabled))) return false;
  if (enabled && box.pal) applyVitalMaximum(box.pal, preference);
  return true;
}

function selectedSnapshot(): string {
  if (!box.pal || box.selectedSlot < 0) return "";
  return JSON.stringify(palToDto(box.pal, box.selectedSlot));
}

export function hasPendingUiEdits(): boolean {
  return Boolean(box.pal && selectedBaseline && selectedSnapshot() !== selectedBaseline);
}

export function hasUnsavedChanges(): boolean {
  return box.dirty || hasPendingUiEdits();
}

/** Persist the currently-loaded Pal's edits into the in-memory engine session. */
async function flush() {
  if (!box.pal || box.selectedSlot < 0) return;

  const submitted = palToDto(box.pal, box.selectedSlot);
  const snapshot = JSON.stringify(submitted);
  if (snapshot === selectedBaseline) return;

  const updated = await updatePal(submitted);
  box.pal = dtoToPal(updated);
  selectedBaseline = selectedSnapshot();
  box.dirty = true;

  // Keep both box explorers in sync with unsaved main-card edits. The engine
  // returns the canonical BOSS_ representation after Alpha/Lucky changes.
  const tile = box.tiles.find((value) => value.slot === box.selectedSlot);
  if (tile) {
    const editable = updated.editable;
    tile.characterId = editable.characterId;
    tile.nickname = editable.nickname;
    tile.gender = editable.gender;
    tile.level = editable.level;
    tile.condensation = editable.condensation;
    tile.ivs = editable.ivs;
    tile.souls = editable.souls;
    tile.isLucky = editable.isLucky;
    tile.isAlpha = editable.isAlpha;
    tile.passives = editable.passives;
    tile.equippedMoves = editable.equippedMoves;
    tile.learnedMoves = editable.learnedMoves;
    tile.projection = updated.projection;
  }
}

/** Commit pending edits, then let the engine change species and re-project every
 * dependent value while preserving the save-backed Work bonuses. */
export async function changeSelectedSpecies(characterId: string) {
  if (!box.pal || box.selectedSlot < 0) return;
  try {
    box.error = "";
    await flush();
    const submitted = palToDto(box.pal, box.selectedSlot);
    submitted.characterId = characterId;
    const updated = await updatePal(submitted);
    box.pal = dtoToPal(updated);
    selectedBaseline = selectedSnapshot();
    applySelectedVitalMaxPreferences();
    box.dirty = true;
    const tile = box.tiles.find((value) => value.slot === box.selectedSlot);
    if (tile) {
      tile.characterId = updated.editable.characterId;
      tile.projection = updated.projection;
    }
  } catch (error) {
    box.error = String(error);
  }
}

export async function openBoxFile(
  path: string,
  options: { automatic?: boolean; remember?: boolean } = {},
): Promise<boolean> {
  box.loading = true;
  box.error = "";
  box.saveMsg = "";
  try {
    const result = await openBox(path);
    box.path = result.path;
    box.slotCount = result.slotCount;
    box.tiles = result.pals;
    box.open = true;
    box.lastBackupPath = "";
    box.selectedSlot = -1;
    box.pal = null;
    box.dirty = false;
    box.sourceState = "unchanged";
    box.sourceDetail = "";
    box.conflict = "";
    box.lastSavedAt = 0;
    box.hint = "";
    selectedBaseline = "";
    unavailableStreak = 0;
    if (
      options.remember !== false
      && !(await rememberBoxPath(result.path))
    ) {
      box.error =
        "The Global Palbox opened, but Studio could not remember it in palbox-user.db.";
    }
    return true;
  } catch (error) {
    if (options.automatic) {
      await setAutoReopen(false);
      box.error =
        `Could not reopen the last Global Palbox. Auto-open was turned off; choose the file again. ${String(error)}`;
    } else {
      box.error = String(error);
    }
    return false;
  } finally {
    box.loading = false;
  }
}

export async function selectSlot(slot: number) {
  try {
    box.hint = "";
    await flush();
    const dto = await getPal(slot);
    box.pal = dtoToPal(dto);
    box.selectedSlot = slot;
    selectedBaseline = selectedSnapshot();
    applySelectedVitalMaxPreferences();
  } catch (error) {
    box.error = String(error);
  }
}

/** Add a brand-new Pal (default: a turtle) — selects + reveals it, unsaved. */
export async function addPal(species: string | null = null) {
  if (!box.open) return;
  try {
    box.error = "";
    await flush();
    const result = await addBoxPal(species);
    box.tiles = result.pals;
    box.dirty = true;
    if (result.slot != null) await selectSlot(result.slot);
    box.hint = "Added — save to keep it. Note: drag it onto an empty box slot in-game to place it.";
  } catch (error) {
    box.error = String(error);
  }
}

/** Clone the Pal at `slot` — selects + reveals the copy, unsaved. */
export async function clonePal(slot: number) {
  if (!box.open || slot < 0) return;
  try {
    box.error = "";
    await flush();
    const result = await cloneBoxPal(slot);
    box.tiles = result.pals;
    box.dirty = true;
    if (result.slot != null) await selectSlot(result.slot);
    box.hint = "Cloned — save to keep it. Note: drag it onto an empty box slot in-game to place it.";
  } catch (error) {
    box.error = String(error);
  }
}

/** Delete the Pal at `slot` (in-memory until save; the original is backed up). */
export async function deletePal(slot: number) {
  if (!box.open || slot < 0) return;
  try {
    box.error = "";
    const result = await deleteBoxPal(slot);
    box.tiles = result.pals;
    box.dirty = true;
    if (box.selectedSlot === slot) {
      box.selectedSlot = -1;
      box.pal = null;
      selectedBaseline = "";
    }
    box.hint = "Removed — takes effect when you save (a backup is kept).";
  } catch (error) {
    box.error = String(error);
  }
}

export async function saveToFile() {
  if (box.conflict) {
    box.error = "Save is blocked because the Global Palbox changed on disk. Reload it before saving.";
    return;
  }
  try {
    box.error = "";
    box.saveMsg = "saving…";
    await flush();
    const backup = await saveBox();
    box.lastBackupPath = backup;
    box.dirty = false;
    box.sourceState = "unchanged";
    box.sourceDetail = "";
    box.conflict = "";
    box.lastSavedAt = Date.now();
    selectedBaseline = selectedSnapshot();
    box.saveMsg = "Saved ✓ (backup: " + backup.split(/[\\/]/).pop() + ")";
    void refreshBoxSessionStatus();
  } catch (error) {
    box.error = String(error);
    box.saveMsg = "";
  }
}

export async function refreshBoxSessionStatus() {
  if (!box.open || monitorInFlight) return;
  monitorInFlight = true;
  try {
    const status = await getBoxSessionStatus();
    box.dirty = status.dirty || hasPendingUiEdits();
    box.sourceState = status.sourceState;
    box.sourceDetail = status.detail ?? "";

    unavailableStreak = status.sourceState === "unavailable" ? unavailableStreak + 1 : 0;
    const confirmedConflict =
      status.sourceState === "changed"
      || (status.sourceState === "unavailable" && unavailableStreak >= 2);
    if (confirmedConflict && !box.conflict) {
      box.conflict = classifySourceConflict(
        status.sourceState,
        box.lastSavedAt,
        Date.now(),
      );
    }
  } catch (error) {
    // Opening/replacing a session can race one monitor tick. The core's
    // persist-time fingerprint still remains authoritative.
    if (box.open && !box.loading) {
      box.sourceState = "unavailable";
      box.sourceDetail = String(error);
    }
  } finally {
    monitorInFlight = false;
  }
}

export function startBoxSourceMonitor(intervalMs = 1_500): () => void {
  void refreshBoxSessionStatus();
  const timer = window.setInterval(refreshBoxSessionStatus, intervalMs);
  return () => window.clearInterval(timer);
}

export async function reloadBoxFromDisk(): Promise<boolean> {
  if (!box.path) return false;
  return openBoxFile(box.path);
}
