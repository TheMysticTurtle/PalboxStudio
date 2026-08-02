# Palbox Studio 1.2.0 (Draft)

This feature release adds Palworld 1.0 Awakening support and focused editor improvements without
changing Palbox Studio's Global-Palbox-only scope or verified-backup save workflow.

## Highlights

- **Awakening support** — the Progression drawer has a labeled Awakening Crystal after the four
  condensation stars. It writes Palworld's separate `bIsAwakening` Boolean and automatically sets
  condensation to four displayed stars (`Rank = 5`).
- **Safer new Pals** — newly created Pals explicitly receive 50/50/50 HP, Attack, and Defense IVs,
  zero displayed condensation stars (`Rank = 1`), and `bIsAwakening = false`. The engine registers
  every required save schema before encoding.
- **Live combat-stat updates** — level, IV, Pal Soul, condensation/Awakening, Alpha/Lucky, and
  passive edits immediately refresh HP, Attack, and Defense through the Rust engine.
- **Persistent MAX controls** — HP, SAN, Food, and Trust each have a MAX toggle. Preferences are
  stored in `palbox-user.db` and applied as Pals are selected. Trust MAX sets rank 10 and full
  progress; toggles can be turned off for custom values.
- **Focused Work Suitabilities** — the editor shows only jobs supported by the species while
  preserving any nonzero saved exception. Levels can be typed from 1–10 or adjusted with arrows.
- **Faster passive-preset creation** — the preset builder can copy the selected Pal's current
  passives into its draft. Nothing is persisted until Save Preset is clicked.

## Fixes and polish

- Reconfirmed and regression-tested the condensation mapping: displayed stars 0–4 are stored as
  `Rank` 1–5. Existing four-star Pals no longer risk appearing one star low after editing.
- Save status and Open Backup now sit to the left of Save Box, keeping Save Box anchored on the
  right side of the title bar.
- Larger element readouts now occupy the open space beside Alpha/Lucky, Trust Rank and MAX share a
  row, and portrait/empty-logo framing has been refined.
- Small labels and secondary readouts are one step larger across the interface for easier reading.
- MAX HP follows a newly calculated maximum after progression edits instead of retaining the
  previous level's current HP.

## Save safety

Palbox Studio still creates and verifies a backup before replacing `GlobalPalStorage.sav`, refuses
stale writes if the source changed after opening, and keeps all edits in memory until Save Box is
clicked.

This is a pre-release draft for testing. The application version and download filenames will be
updated when the release is finalized.
