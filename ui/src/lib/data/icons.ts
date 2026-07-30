import type { ElementName } from "./types";
import { ref } from "./refdata.svelte";

// Pal portrait icon path. Mirrors PalEdit's GetImage lookup rule (see
// docs/DATA-AND-ASSETS.md): the bundled files are `T_<CodeName>_icon_normal.png`;
// strip a RAID_/BOSS_ prefix and a trailing _2, and special-case PlantSlime.
// Alpha pals store `BOSS_<Code>` but share the base icon, so we strip BOSS_ too.
export const PAL_ICON_FALLBACK = "/pals/%23ERROR.png";
export const APP_LOGO_ART = "/logo.png";

// Save/reference codes and PalEdit asset basenames are normally identical.
// Lamball is the known exception: `Sheepball` in game data, `SheepBall` on disk.
const PAL_ICON_CODE_ALIASES: Record<string, string> = {
  Sheepball: "SheepBall",
};

export function palIcon(code: string): string {
  const withoutVariant = code.replace(/^BOSS_/i, "");
  const canonical = ref.speciesAliases[withoutVariant] ?? withoutVariant;
  const normalized = (canonical.includes("PlantSlime") ? "PlantSlime" : canonical)
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

export function elementIcon(element: ElementName): string {
  const basename = ref.elements[element]?.icon;
  return basename ? `/icons/elements/${basename}.webp` : APP_LOGO_ART;
}

export type PalVariant = "alpha" | "lucky";

const VARIANT_ICON_PATHS: Record<PalVariant, string> = {
  alpha: "/icons/variants/alpha.webp",
  lucky: "/icons/variants/lucky.webp",
};

export function variantIcon(variant: PalVariant): string {
  return VARIANT_ICON_PATHS[variant];
}

/** Shared interface artwork; components never own the deployed asset path. */
export const STATUE_OF_POWER_ART = "/icons/statue-of-power.webp";

/** Resolve all Work Suitability artwork through one path contract. */
export function workIcon(basename: string, active = true): string {
  return `/icons/work/${active ? basename : `no_${basename}`}.png`;
}
