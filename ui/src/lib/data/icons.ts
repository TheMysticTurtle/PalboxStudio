// Pal portrait icon path. Mirrors PalEdit's GetImage lookup rule (see
// docs/DATA-AND-ASSETS.md): the bundled files are `T_<CodeName>_icon_normal.png`;
// strip a RAID_/BOSS_ prefix and a trailing _2, and special-case PlantSlime.
// Alpha pals store `BOSS_<Code>` but share the base icon, so we strip BOSS_ too.
export const PAL_ICON_FALLBACK = "/pals/%23ERROR.png";

export function palIcon(code: string): string {
  const c = (code.includes("PlantSlime") ? "PlantSlime" : code)
    .replace(/^(RAID_|BOSS_)/i, "")
    .replace(/_2$/i, "");
  return `/pals/T_${c}_icon_normal.png`;
}

/** onerror handler: swap a missing portrait for the shared placeholder. */
export function onPalIconError(e: Event): void {
  const img = e.currentTarget as HTMLImageElement;
  if (!img.src.endsWith("%23ERROR.png")) img.src = PAL_ICON_FALLBACK;
}
