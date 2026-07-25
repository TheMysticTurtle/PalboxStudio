// The open Global Palbox session (real engine data). Holds the box tiles, the
// selected slot, and the currently-loaded editable pal. Edits mutate `box.pal`;
// they're flushed to the in-memory box (updatePal) on slot-switch and on save.
import {
  openBox,
  getPal,
  updatePal,
  saveBox,
  addBoxPal,
  cloneBoxPal,
  deleteBoxPal,
  type BoxTileDto,
} from "$lib/data/engine";
import { dtoToPal, palToDto } from "$lib/data/mapper";
import type { Pal } from "$lib/data/types";

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
  /** Transient hint after an add/clone/delete (e.g. the in-game "drag" caveat). */
  hint: "",
});

/** Persist the currently-loaded pal's edits into the in-memory box. */
async function flush() {
  if (box.pal && box.selectedSlot >= 0) {
    await updatePal(palToDto(box.pal, box.selectedSlot));
  }
}

export async function openBoxFile(path: string) {
  box.loading = true;
  box.error = "";
  box.saveMsg = "";
  try {
    const r = await openBox(path);
    box.path = r.path;
    box.slotCount = r.slotCount;
    box.tiles = r.pals;
    box.open = true;
    box.selectedSlot = -1;
    box.pal = null;
  } catch (e) {
    box.error = String(e);
  }
  box.loading = false;
}

export async function selectSlot(slot: number) {
  try {
    box.hint = "";
    await flush();
    const dto = await getPal(slot);
    box.pal = dtoToPal(dto);
    box.selectedSlot = slot;
  } catch (e) {
    box.error = String(e);
  }
}

/** Add a brand-new pal (default: a turtle) — selects + reveals it, unsaved. */
export async function addPal(species: string | null = null) {
  if (!box.open) return;
  try {
    box.error = "";
    await flush();
    const r = await addBoxPal(species);
    box.tiles = r.pals;
    if (r.slot != null) await selectSlot(r.slot);
    box.hint = "Added — save to keep it. Note: drag it onto an empty box slot in-game to place it.";
  } catch (e) {
    box.error = String(e);
  }
}

/** Clone the pal at `slot` — selects + reveals the copy, unsaved. */
export async function clonePal(slot: number) {
  if (!box.open || slot < 0) return;
  try {
    box.error = "";
    await flush();
    const r = await cloneBoxPal(slot);
    box.tiles = r.pals;
    if (r.slot != null) await selectSlot(r.slot);
    box.hint = "Cloned — save to keep it. Note: drag it onto an empty box slot in-game to place it.";
  } catch (e) {
    box.error = String(e);
  }
}

/** Delete the pal at `slot` (in-memory until save; the original is backed up). */
export async function deletePal(slot: number) {
  if (!box.open || slot < 0) return;
  try {
    box.error = "";
    const r = await deleteBoxPal(slot);
    box.tiles = r.pals;
    if (box.selectedSlot === slot) {
      box.selectedSlot = -1;
      box.pal = null;
    }
    box.hint = "Removed — takes effect when you save (a backup is kept).";
  } catch (e) {
    box.error = String(e);
  }
}

export async function saveToFile() {
  try {
    box.saveMsg = "saving…";
    await flush();
    const backup = await saveBox();
    box.saveMsg = "Saved ✓ (backup: " + backup.split(/[\\/]/).pop() + ")";
  } catch (e) {
    box.error = String(e);
    box.saveMsg = "";
  }
}
