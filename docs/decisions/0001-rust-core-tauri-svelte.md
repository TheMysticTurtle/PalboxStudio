# ADR 0001 — Rust core + Tauri + Svelte

- **Status:** Accepted
- **Date:** 2026-07-24
- **Deciders:** owner (direction/UX) + Claude (implementation)
- **Supersedes:** the initial "Python because the ecosystem lives there" leaning

## Context

We are rebuilding, from scratch, a beautiful Palworld 1.0 **Global Pal box** editor,
carrying forward everything we learned maintaining the PalEdit 1.0 fork (Python/Tkinter).
The rebuild has to answer two forks: what language is the save-parsing/domain **core**, and
how do we get a **Palworld-grade GUI** on top of it.

Findings from research (see [../RESEARCH-landscape.md](../RESEARCH-landscape.md)):

1. **"Format knowledge lives in Python" is not a durable moat.** The canonical Python parser
   `cheahjs/palworld-save-tools` last had a code push 2024-10; every serious editor bundles
   and maintains its own parser fork regardless of language. We will own a fork either way.
2. **The realistic core languages are Python or Rust.** C#, Go, and pure-JS have no
   maintained Palworld-aware save parser to build on.
3. **Palworld Save Pal (PSP), the strongest modern editor, migrated Python → Rust** and ships
   **Rust core + Tauri + Svelte** as a single binary. A mature project walked this exact path.
4. We already **vendor the full PSP Rust source** in `PalEdit/psp-reference/` for study.
5. The **one capability we could never bring into PalEdit** — world/`Level.sav` edits — was
   blocked by 1.0's guild-tail trailing bytes (`GroupSaveDataMap` "EOF not reached"). PSP
   solves it in Rust (`guild.rs` + `guild_tail.rs`); a Rust→Python port was the wall.

## Decision

- **Core: Rust.** Build our own Palworld domain model from our RE knowledge + PSP as a
  mechanics reference, on top of a **vendored** general UE-save Rust crate (uesave-rs
  lineage) for the byte-level GVAS/Oodle plumbing. We do **not** reinvent the universal UE
  container layer (that's like writing our own zip lib); we **do** own 100% of the
  Palworld-specific intelligence, vendored so we're never blocked by an upstream.
- **Frontend: Svelte in a Tauri shell.** Web-tech gives the best shot at the Palworld look
  and the cleanest handoff to the design pass; Tauri keeps it a small native app.
- **Distribution: one standalone signed binary** — a clean drop-in for the owner's Vortex
  tool tile.
- **Scope: the Global Pal box only** (see D5 in DIRECTION.md). World saves are explicitly
  out for v1.

## Why not the alternatives

- **Python core + Svelte (KrisCris model).** Tempting because it reuses our existing PalEdit
  1.0 parsing. Rejected: keeps the cx_Freeze packaging pain we already fight every release,
  and keeps the Rust→Python wall for any future world-save work. The reuse win is smaller
  than it looks — our real asset is the *knowledge*, which ports to Rust fine.
- **C# / Go.** No maintained Palworld-aware parser; we'd pioneer the byte format alone.
- **The learning-curve argument against Rust** evaporated: the owner guides/designs and does
  not code; Claude writes the code and is equally at home in Rust.

## Consequences

**Positive:** clean single-binary distribution; performance and memory safety; a codebase
that still builds cleanly years out; the guild-tail wall is gone if scope ever grows; we
own our parser fork outright.

**Negative / to manage:** larger up-front build than a Python port; we maintain a vendored
UE-save crate and our own domain layer; must keep our RE honest — **PSP is buggy, use it for
mechanics only, never mimic blindly** (owner hit real PSP load errors).

**Carry-over discipline (non-negotiable):** never touch live saves (scratchpad copies only);
prove correctness with the **no-edit round-trip diff = zero fields changed**; one feature per
branch; commits explain the *why* + bug/benefit/test notes; ADRs for architecture.
