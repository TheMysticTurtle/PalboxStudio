// The open Global Palbox session (real engine data). Holds the box tiles, the
// selected slot, and the currently-loaded editable pal. Edits mutate `box.pal`;
// they're flushed to the in-memory box (updatePal) on slot-switch and on save.
import { openBox, getPal, updatePal, saveBox, type BoxTileDto } from "$lib/data/engine";
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
    await flush();
    const dto = await getPal(slot);
    box.pal = dtoToPal(dto);
    box.selectedSlot = slot;
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
