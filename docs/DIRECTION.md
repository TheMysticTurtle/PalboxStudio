# Palbox Studio — Direction & Intent

> Working name: **Palbox Studio** (provisional — easy to rename before we publish).
> This document preserves the *why* and the *vision*. The stack is now **decided** — see
> "Locked Decisions" at the bottom and [decisions/0001-rust-core-tauri-svelte.md](decisions/0001-rust-core-tauri-svelte.md).

## What this is

A proper, ground-up application for editing the **global Pal box** in Palworld 1.0 —
taking everything we learned building/maintaining the PalEdit 1.0 fork and rebuilding it
as a real, well-engineered product with a beautiful GUI.

This is a rewrite in *intent*, not a patch on PalEdit.

## North-star principles

1. **Palworld's visual language.** Same color scheme, patterns, and feel as the game.
   Different layout (see below) — but it should feel like an official companion tool,
   not a modder's utility. Beautiful, clearly labeled, intuitive.
2. **Official names throughout.** Use Palworld's official terminology everywhere
   (stats, "IVs"/talents, the statue power level, passives, moves) — no fan slang in the UI.
3. **Modular / compartmentalized around a shared core engine.** Feature areas are
   independent modules that all depend on one headless core. The UI depends on the core;
   the core never depends on the UI.
4. **Safety first.** Never touch live saves. Always work on copies, always back up before
   write, atomic writes only. (Carried over from the PalEdit working rules.)
5. **Engineering discipline as a first-class feature.** Detailed commits that explain the
   *why*, bugs encountered / perceived benefit, testing approach and results. Decision
   records (ADRs) for anything architectural. Tests around the core from day one.

## Layout — the "tri-fold"

The main window is three vertical sections.

### Left third — Global Box Explorer
- **Tiles for each Pal**: thumbnail image, level, type, quick-view of available
  skills / passives.
- **Default state:** a vertical, scrollable column of *collapsed* tiles.
- **Expandable:** an arrow pulls out a side menu; the expanded view arranges Pals in a
  **matrix**, like peering into the actual Pal box and selecting from the grid.
- **Search & filter:** everything searchable and filterable, clearly labeled and intuitive.
  Similar *capability* to the species selector — but far cleaner/prettier than it.
- **Groups:** users can create groups to quickly edit the Pals they use for a given purpose
  without wading through the full box.
- **Tags:** freeform tagging for organization.
- **Operations:** add, clone, delete Pals.

### Center third — Pal Card (character display + editor)
- Laid out like the **Palworld 1.0 character card** (a reference source image will be
  provided — attach to the design handoff).
- Character display, stats, etc. Stats **clearly labeled and adjustable**.
- **Passives:** easy to add and filter.
- **Presets:** "apply preset" action, plus the ability to create/save presets.

### Right third — Talents / IVs · Statue Power Level · Moves
Split into thirds vertically:
1. **Talents / IVs** — the individual value stats.
2. **Statue power level** — (confirm the official in-game name and use it).
3. **Moves** — the Pal's move set.

## Cross-cutting features
- Presets (for passives / builds) — apply and author.
- Groups + tags for box organization.
- Add / clone / delete Pals.
- Consistent, prominent search + filter.

## Locked Decisions (2026-07-24)

See [decisions/0001-rust-core-tauri-svelte.md](decisions/0001-rust-core-tauri-svelte.md)
for the full rationale. Summary:

- **D1 — Core language: Rust.** Our own Palworld domain model, built fresh from our RE
  knowledge ([PalEdit notes](../../PalEdit/CLAUDE.md)) + the vendored PSP Rust source
  (`PalEdit/psp-reference/`, mechanics reference only — PSP is buggy, we don't copy blind),
  on top of a vendored general UE-save Rust crate for the byte-level GVAS/Oodle plumbing.
- **D2 — Frontend: Svelte + Tauri.** Web-tech UI (best for the Palworld look + clean design
  handoff) in a Tauri shell talking to the Rust core.
- **D3 — Distribution: single standalone binary.** Clean drop-in for the owner's Vortex tool
  tile — kills the cx_Freeze / copy-over-the-folder pain we live with in PalEdit today.
- **D4 — Reuse the *knowledge*, not the code.** We own the intelligence; fresh Rust
  implementation. Bonus: the world-save guild-tail logic we *couldn't* port to Python is
  native Rust in PSP — no longer a wall (relevant only if scope ever expands; see D5).
- **D5 — Scope: the Global Pal box, and only that.** A beautiful global-box editor is the
  whole v1 goal. **Explicitly OUT of scope for now:** world/`Level.sav` editing, bases,
  map, tech, party Pals. Rust leaves that door open for someday — we are not walking
  through it now. Don't scope-creep.

Rejected alternatives (see ADR 0001): Python core + web frontend (the KrisCris model —
would reuse our PalEdit code but keeps the packaging pain and the Rust→Python wall); C# / Go
(no maintained Palworld-aware save parser to build on).

## Reference material to gather
- Palworld 1.0 character-card screenshot (center panel reference).
- Palworld UI palette / fonts (for the design system).
- Official terminology list (stats, IV/talent label, statue power-level name, passive/move naming).
