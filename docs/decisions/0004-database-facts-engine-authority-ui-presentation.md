# ADR 0004: Database facts, engine authority, and presentation-only UI

Date: 2026-07-30  
Status: Accepted

## Context

The original desktop bridge returned raw save values and the Svelte UI separately
implemented combat-stat formulas, Work Suitability base/bonus arithmetic, Trust
thresholds, Partner Skill progression, element catalogs, and several editor
limits. That made a future Palworld data refresh unsafe: a regenerated reference
database could disagree with facts still embedded in Rust or TypeScript.

The headless core is also intended to support more than one frontend. A second UI
or CLI should not need to reproduce Palbox Studio's calculations or save encodings.

## Decision

Use the following authority boundary:

1. **`palbox-reference.db` owns patch-sensitive facts.** Species, codes, names,
   ordering, icons, base Work levels, limits, progression tables, and numeric
   calculation operands live in normalized, typed tables. The generated database
   is loaded once into a validated, indexed `ReferenceCatalog`.
2. **`palbox-core` owns behavior.** The engine decodes raw save fields, validates
   semantic edits, computes display projections, translates Work totals into
   `AddRank` bonuses, translates Trust ranks/progress into friendship points,
   preserves binary-format conventions, and encodes the output.
3. **The Tauri layer owns session orchestration only.** It keeps the parsed save
   and cached catalog alive, marshals commands, and delegates mutations and
   projections to the headless engine.
4. **The UI owns presentation and transient interaction only.** Layout, animation,
   selection state, drag gestures, percentage formatting, and visual fallbacks
   stay in Svelte. It does not reproduce game formulas or save encodings.
5. **`palbox-user.db` owns durable user metadata only.** Presets, groups,
   memberships, and app preferences survive upgrades through forward migrations.
   Patch-sensitive limits are not duplicated in its schema.

The public Pal edit contract is semantic: frontends receive editable user-facing
values plus an engine-computed projection. Stable `InstanceId` is checked on
mutation so stale frontends cannot edit a different Pal after slot contents change.

## Human inspection

The reference database remains normalized for engine use and also provides named
SQLite views for common inspection tasks: species summaries, species Work levels,
Partner Skill ranks, moves/effects, passives/effects, and source provenance.
Internal codes and human-readable names appear together.

## Consequences

- Updating verified data and rebuilding the reference DB updates every frontend
  without hunting for mirrored constants or formulas.
- Invalid or incomplete reference data fails during catalog construction instead
  of producing late UI inconsistencies.
- Box tiles and the full editor use the same projections.
- Negative Trust ranks are preserved and exposed because the DB defines the full
  range; unrelated edits cannot silently normalize negative Friendship to zero.
- Formula structure and Unreal/save-property conventions remain versioned engine
  behavior, while patch-sensitive operands remain replaceable data.
- The semantic payload is larger than a single-field command, but the mutation is
  transactional, identity-checked, and reusable without Tauri or Svelte.
