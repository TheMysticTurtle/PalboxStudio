# Palbox Studio 1.1.2 🐢

Palbox Studio is a desktop editor for the Palworld 1.0 Global Palbox. Open your box, shape your Pals
exactly how you want them, and save with confidence — Palbox Studio creates and verifies a backup
before every write.

Global Palbox only: Palbox Studio does not edit world saves, bases, or party Pals.

Version 1.1.2 adds Palworld 1.0 Awakening support alongside a focused collection of quality-of-life,
save-safety, and interface improvements. Have an idea or find an issue? Feedback is always welcome
on GitHub.

## ✨ Key features

- 🐾 **Comprehensive Pal editing** — Edit species, nickname, gender, level, IVs, Pal Souls,
  condensation, Awakening, passive skills, active and learned moves, Work Suitabilities, Trust,
  vitals, and the Lucky and Alpha flags.
- 📦 **Box explorer** — Browse the entire Global Palbox from the side panel or expand it into a full
  gallery.
- 🔍 **Powerful search and filtering** — Filter by element, Work Suitability, ride or mount type,
  ranch drops, obtainability, or name.
- 🏷️ **Groups and tags** — Organize the box with custom groups and tags.
- 🔄 **Species, passive, and move pickers** — Find exactly what you need through searchable,
  filterable selectors.
- ➕ **Add, clone, and remove Pals.**
- 💾 **Verified backups** — Every save begins by creating and verifying a backup before safely
  replacing the original file.

## 🆕 New in 1.1.2

- **Pal Awakening** — A new Awakening Crystal control appears after the four condensation stars.
  Awakening uses Palworld's separate save value and automatically fully condenses the Pal first.
- **Live combat-stat updates** — HP, Attack, and Defense now refresh while editing level, IVs, Pal
  Souls, condensation, Awakening, passives, or Alpha/Lucky status.
- **Remembered MAX controls** — HP, SAN, Food, and Trust can be set to MAX as a saved preference.
  Disable any toggle whenever you want to enter a custom value instead.
- **Focused Work Suitabilities** — The editor shows the jobs a species actually supports while
  preserving existing nonzero exceptions. Values can now be typed directly from 1–10 or adjusted
  with the arrow buttons.
- **Safer new Pals** — Newly created Pals start with 50/50/50 HP, Attack, and Defense IVs and receive
  the complete save schema expected by Palworld.
- **Faster passive presets** — The preset builder can copy the selected Pal's current passive skills
  directly into a new preset draft.

## 🛠️ Fixes and polish

- **Correct condensation display** — The 0–4 displayed stars now consistently map to Palworld's
  stored ranks 1–5, preventing four-star Pals from appearing one star short.
- **MAX HP follows progression** — A Pal using MAX HP now remains full when level or progression
  changes increase its calculated maximum.
- **Cleaner save header** — Save status and Open Backup stay to the left while Save Box remains
  anchored on the right.
- **Roomier, clearer editor** — Element badges sit near the species selector, Trust Rank and MAX
  share a row, and the Pal portrait and empty-state logo are better aligned.
- **Improved readability** — Small labels and secondary readouts are larger, condensation stars are
  better proportioned, and the Awakened control is easier to see and select.

## ⬇️ Downloads

- **Windows installer** — `PalboxStudio-1.1.2-setup.exe`
- **Windows portable** — `PalboxStudio-1.1.2-portable.zip` — extract and run; also suitable for use
  as a Vortex tool.
- **Linux AppImage** — `PalboxStudio-1.1.2-linux.AppImage`
- **Linux Debian package** — `PalboxStudio-1.1.2-linux.deb`

Linux builds remain experimental. Windows builds are unsigned, so Windows SmartScreen or antivirus
software may show a warning on first launch. This is expected for an independent unsigned
application; the complete source is available on GitHub for inspection.

## 🛡️ Before editing

Palbox Studio keeps edits in memory until Save Box is clicked, creates and verifies a backup before
replacing `GlobalPalStorage.sav`, and refuses stale writes if the source file changed after opening.
Keeping an additional copy of any important save is always recommended.

---

Source-available under the **PolyForm Strict License 1.0.0**. Palbox Studio is an unofficial,
fan-made tool. Palworld is a trademark of Pocketpair, Inc.
