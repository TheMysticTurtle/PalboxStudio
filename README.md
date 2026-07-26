<h1 align="center">Palbox Studio</h1>

<p align="center"><strong>A gorgeous, purpose-built editor for your Palworld&nbsp;1.0 Global Palbox.</strong></p>

<p align="center">
  <img alt="Windows" src="https://img.shields.io/badge/Windows-10%20%2F%2011-0a7bbd">
  <img alt="Version" src="https://img.shields.io/badge/release-1.0.0-b060e0">
  <img alt="Built with" src="https://img.shields.io/badge/built%20with-Rust%20%C2%B7%20Tauri%20%C2%B7%20Svelte-3fc7e0">
  <img alt="License" src="https://img.shields.io/badge/license-PolyForm%20Strict%201.0.0-6e7a86">
</p>

<p align="center">
  <!-- SCREENSHOT: the main editor card, a pal loaded -->
  <img src="docs/screenshots/01-main-editor.png" alt="Palbox Studio — the main pal editor" width="900">
</p>

Palbox Studio reads your real `GlobalPalStorage.sav`, shows every Pal in it, and lets you edit
them through an interface that actually looks like it belongs next to Palworld — not a spreadsheet,
not a form from 2005. It's a **standalone desktop app**, not a game mod: nothing is installed into
Palworld and nothing is deployed into the game folder. You open your box, make it exactly what you
want, and save — with a verified backup taken every single time.

> **Focused by design:** Palbox Studio edits the **Global Palbox only**. No world saves, no bases,
> no party — just your box, done properly.

---

## ✦ What you can do

- **Edit a Pal completely** — species, nickname, gender, level, IVs, Pal Souls, condensation,
  passive skills, active & learned moves, work suitabilities, and the Lucky / Alpha flags.
- **Browse your whole box** at a glance, or blow it up into a full gallery with live stats.
- **Filter by anything** — element, work suitability, "rideable as a mount," ranch drops, and
  obtainability, plus a name/species search. The same filter drives the box *and* the species picker.
- **Organize with tags** — create your own groups, tag Pals, and filter to them.
- **Change a Pal's species** from a searchable, icon-rich picker — the card re-derives its type,
  Partner Skill, and learnset instantly.
- **Add, clone, and delete** Pals; a new one defaults to a turtle 🐢 and jumps straight into view.
- **Never lose data** — every save writes a byte-verified backup first, then replaces atomically.

Everything you see is rendered from an authoritative, bundled game-data database (species,
movesets, passives, Partner Skills, ranch products, work suitabilities), so the display always
matches Palworld 1.0.

---

## ✦ A quick tour

### The editor
The centerpiece: one clean card for the selected Pal. Portrait, typing, calculated combat stats,
Partner Skill, passives, active/bench moves, and work suitabilities — all editable, all live.

<p align="center">
  <!-- SCREENSHOT: main editor card (can reuse the hero or a different Pal) -->
  <img src="docs/screenshots/01-main-editor.png" alt="The main editor card" width="820">
</p>

### The Global Palbox explorer
A compact side drawer lists every Pal in your box with search, filters, and tags — right next to
the editor so you can hop between Pals fast.

<p align="center">
  <!-- SCREENSHOT: the Global Box drawer (compact explorer) -->
  <img src="docs/screenshots/02-palbox-explorer.png" alt="The Global Palbox explorer drawer" width="360">
</p>

### Expanded explorer
Need the big picture? Expand the box into a full gallery — every Pal as a rich card with stats,
equipped moves, passives, and work suitabilities at a glance.

<p align="center">
  <!-- SCREENSHOT: expanded full-matrix gallery -->
  <img src="docs/screenshots/03-expanded-explorer.png" alt="The expanded Palbox gallery" width="900">
</p>

### Filters that actually help
Element groups, the 13 work suitabilities, "rideable mount," ranch-drop products, obtainability
buckets, and a live search — collapsible so it never crowds the view. It filters your box and the
species picker with the same controls.

<p align="center">
  <!-- SCREENSHOT: filters expanded -->
  <img src="docs/screenshots/04-filters.png" alt="The shared species filter" width="900">
</p>

### Your own groups & tags
Make groups (favorites, breeders, projects — whatever), tag Pals, and filter down to them. Your
tags are stored per-user and survive updates.

<p align="center">
  <!-- SCREENSHOT: groups / tag management -->
  <img src="docs/screenshots/05-groups.png" alt="Groups and tags" width="900">
</p>

### The stats you can edit
IVs, Pal Souls, condensation, level, passives, moves, work suitabilities and more — with clear
readouts and sensible ranges (plus room for the power-user values Palworld can reach).

<p align="center">
  <!-- SCREENSHOT: stats / advanced editing (IVs, souls, condensation, work suits) -->
  <img src="docs/screenshots/06-stats.png" alt="Editable stats" width="900">
</p>

### Backups you can trust
Every **Save Box** encodes and verifies the edited data, writes a byte-checked backup beside your
save, then replaces the original atomically. A backup failure aborts the save before your file is
ever touched — and **Open backup** reveals the exact file it kept.

<p align="center">
  <!-- SCREENSHOT: save confirmation / backup handling -->
  <img src="docs/screenshots/07-backups.png" alt="Backup handling" width="900">
</p>

---

## ✦ Download & install

Grab the latest build from the [**Releases**](../../releases) page (or from Nexus Mods).

**Installer** — download `PalboxStudio-<version>-setup.exe` and run it. It creates a Start-menu
shortcut and sets up the WebView2 runtime if your Windows doesn't already have it.

**Portable** — download `PalboxStudio-<version>-portable.zip`, unzip it anywhere, and double-click
`Palbox Studio.exe`. Nothing to install; keep the folder together.

**With Vortex** (optional) — the portable folder doubles as a Vortex *tool*: on the Palworld
dashboard, **+ Add Tool → New…**, point **Target** at `Palbox Studio.exe`, and it launches from a
tile like your other tools. (Full steps are in the portable zip's `READ ME FIRST.txt`.)

> Windows 11 already includes the WebView2 runtime Palbox Studio uses. On older Windows, if the app
> won't start, install the free **Microsoft Edge WebView2 Runtime** and try again.

### Safety first — always
- Palbox Studio **never edits a live save in place** without first taking a verified backup.
- **Close Palworld completely** before saving edits.
- Your box lives at `%LOCALAPPDATA%\Pal\Saved\SaveGames\<your-id>\GlobalPalStorage.sav`.

---

## ✦ Build from source

Prereqs: Rust (stable-msvc) + Node.

```bash
npm install && npm --prefix ui install   # one-time
npm run tauri dev                          # run the app (hot-reloads the UI)
```

`npm run build` builds the frontend, `cargo test` runs the engine tests, and
`python scripts/build_reference_db.py --check` verifies the reference database.

### Release build

One command produces every distribution artifact into `dist/`:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

It runs the pre-bundle gate (type-check / format / tests / reference-DB validation), compiles the
Tauri release binary, and emits the **NSIS installer** and the **portable / Vortex zip** (with its
`READ ME FIRST.txt`). Both share one writable user database in `%APPDATA%` (tags / groups / presets)
so that data survives upgrades and switching between builds; the read-only reference DB rides along
inside each. Flags: `-SkipGate`, `-InstallerOnly`, `-PortableOnly`.

---

## ✦ Under the hood

- **`core/`** — a headless Rust engine (`palbox-core`): the Palworld 1.0 save model, load/write,
  and mutations, with verified 1.0 limits. No UI; unit-tested against real box data.
- **`src-tauri/`** — the Tauri 2 desktop shell that bridges the engine to the UI.
- **`ui/`** — the Svelte 5 + Vite frontend, styled in Palworld's visual language.
- **`database/` + `data/`** — a normalized, bundled SQLite **reference** database (species, moves,
  passives, Partner Skills, ranch products, localization) loaded into memory at startup, plus the
  writable **user** database (groups / tags / presets).

Design decisions live in [`docs/decisions/`](docs/decisions); the vision and specs are in
[`docs/DIRECTION.md`](docs/DIRECTION.md) and [`docs/SPECS-1.0.md`](docs/SPECS-1.0.md).

## ✦ Principles

- Palworld's visual language; official in-game terminology throughout.
- Safety first: never touch live saves — copies, verified backups, atomic writes only.
- One authoritative data source; the UI resolves everything from it, never hand-written strings.
- Engineering discipline: commits explain *why*, ADRs for architecture, tests around the core.

## ✦ Credits

A ground-up successor in spirit to the PalEdit 1.0 fork. Save decode/encode is powered by the
[`uesave`](https://github.com/oMaN-Rod/uesave-rs) fork used by the Palworld save-editing community;
1.0 data facts are cross-checked against the open reference work of that community. Palworld is a
trademark of Pocketpair, Inc. — Palbox Studio is an unofficial, fan-made tool.

## ✦ License

Palbox Studio is **source-available, not open source.** It's licensed under the
[PolyForm Strict License 1.0.0](LICENSE): you're free to download and use the app for personal,
noncommercial purposes, and the source is here so you can inspect exactly what it does — but you may
not redistribute it, or create or share modified/forked versions, without permission. Please don't
re-host it or publish bastardized builds; if you want to collaborate or reuse part of it, just ask.
