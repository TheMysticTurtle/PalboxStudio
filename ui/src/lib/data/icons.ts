import type { ElementName } from "./types";

// Pal portrait icon path. Mirrors PalEdit's GetImage lookup rule (see
// docs/DATA-AND-ASSETS.md): the bundled files are `T_<CodeName>_icon_normal.png`;
// strip a RAID_/BOSS_ prefix and a trailing _2, and special-case PlantSlime.
// Alpha pals store `BOSS_<Code>` but share the base icon, so we strip BOSS_ too.
export const PAL_ICON_FALLBACK = "/pals/%23ERROR.png";

// Save/reference codes and PalEdit asset basenames are normally identical.
// Lamball is the known exception: `Sheepball` in game data, `SheepBall` on disk.
const PAL_ICON_CODE_ALIASES: Record<string, string> = {
  Sheepball: "SheepBall",
};

export function palIcon(code: string): string {
  const normalized = (code.includes("PlantSlime") ? "PlantSlime" : code)
    .replace(/^(RAID_|BOSS_)/i, "")
    .replace(/_2$/i, "");
  const c = PAL_ICON_CODE_ALIASES[normalized] ?? normalized;
  return `/pals/T_${c}_icon_normal.png`;
}

/** onerror handler: swap a missing portrait for the shared placeholder. */
export function onPalIconError(e: Event): void {
  const img = e.currentTarget as HTMLImageElement;
  if (!img.src.endsWith("%23ERROR.png")) img.src = PAL_ICON_FALLBACK;
}

/** Bundled game-style badges shared by Pal species and move element displays. */
const ELEMENT_ICON_PATHS: Record<ElementName, string> = {
  Neutral: "/icons/elements/neutral.webp",
  Fire: "/icons/elements/fire.webp",
  Water: "/icons/elements/water.webp",
  Grass: "/icons/elements/grass.webp",
  Electric: "/icons/elements/electric.webp",
  Ice: "/icons/elements/ice.webp",
  Ground: "/icons/elements/ground.webp",
  Dark: "/icons/elements/dark.webp",
  Dragon: "/icons/elements/dragon.webp",
};

export function elementIcon(element: ElementName): string {
  return ELEMENT_ICON_PATHS[element];
}
