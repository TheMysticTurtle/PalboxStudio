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

## Layout — center-stage card + two retractable drawers

Revised 2026-07-24 (from the earlier static "tri-fold"): the window is a **big editable Pal
card in the center**, always visible, flanked by **two side drawers that pop out and retract
into the edges**. With both drawers retracted you see the whole card, clean and unobstructed.
Both drawers are **clearly labeled** at their edge tabs. Visual anchor: the Party "Pal Stats"
screen + the Global Palbox screen (see `docs/reference/`).

### Center — the Pal Card (always visible, fully editable)
Modeled on Palworld 1.0's **Party "Pal Stats"** full card. Everything on it is editable and
clearly labeled; use **official in-game terminology** throughout:
- Portrait/render, **LEVEL**, name (+ **Edit**), gender, **element** pills, `NEXT` exp,
  **Level Sync**, **Trust**, **Favorite**.
- **Stats** — Attack / Defense / Work Speed (+ HP, SAN, hunger/food) — adjustable, with the
  in-game "boosted" up-arrow treatment where relevant.
- **Partner Skill**, **Passive Skills** (2×2 chips, rating-colored; easy add/filter).
- **Work Suitability** (all 12 jobs, greyed when N/A, level shown when active).
- **Moves = Active Skills** live on the **right side of the card** (element-colored rows with
  power values, like in-game).
- **Presets:** "apply preset" + author/save presets (passives / builds).

### Left drawer — Global Box Explorer (pop-out, retractable)
- Pops out from the left, clearly labeled; retracts into the edge to reveal the full card.
- **Themed tiles, not bare dots:** each Pal is the round portrait **inside a tile that
  matches the theme**, showing a brief data overview (level, element, quick skills/passives).
- Collapsed = scrollable list of tiles; expanded = **matrix** grid, like peering into the box.
- **Search & filter** — same capability as the in-game Sort/Filter modal, far cleaner.
- **Operations:** add, clone, delete Pals.

### Right drawer — Advanced / "hidden settings" (pop-out, retractable)
- Pops out from the right, clearly labeled.
- **IV / breeding traits** (the individual talent values) — clearly labeled + adjustable.
- **Statue power level** = the **Pal Souls** rank (raised at the *Statue of Power*; confirm
  the exact official label). **Include an image of the Statue of Power** in this window.

## Groups & tags (customizable, filter-like)
- **Groups are user-named, fully customizable collections** — the user names them anything.
- A group **behaves like a filter**: selecting it filters the box to its members.
- **Tags bind pals to groups:** you *tag* the pals you want into a group; a pal can belong to
  multiple groups. This is the easy-editing workflow — tag your "breeders" or "base workers"
  and jump straight to editing just them.
- **These are Palbox Studio metadata, NOT save data.** Stored app-side (our own local
  db/file), keyed by each Pal's `InstanceId` — never written into the `.sav`. Must survive
  clone/add/delete sensibly (new InstanceId on clone → not auto-tagged unless we choose to).

## Cross-cutting features
- Presets (for passives / builds) — apply and author.
- Groups + tags for box organization (see above).
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
