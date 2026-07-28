# Palbox Studio — engineering backlog

A prioritized backlog of hardening and refinement work, drawn from two engineering audits of the
project. Items are grouped by tier — save integrity first, then correctness, then trust and
hardening, then deliberate refinements. Each item earns its place by reducing real risk, improving
testability, or making future changes cheaper.

Items come from the audits and should be confirmed against the current code before implementation.

Status key: `[ ]` open · `[x]` done · `[~]` in progress · `[?]` needs investigation first.

## Decisions already made

- **Remove the speculative user-database migration machinery.** No released user has a legacy
  schema to migrate, and the user database holds only app metadata (groups, tags, presets), never
  save data. On an incompatible schema, recreate the user database rather than migrate. Confirm the
  database-open logic and baseline schema first so nothing is lost.
- **Adopt core-authoritative validation.** The Rust core is the single source of truth for every
  editable limit; the UI restricts values for convenience only. This removes the editing limits
  that are currently mirrored between the core and the frontend.
- **Generated Rust→TS bindings: revisit later.** Worth adopting once the DTO surface is stable and
  the maintenance benefit is clear; not adopted preemptively.

## Tier 0 — save integrity (do first)

- [ ] **Detect external save changes.** Record a fingerprint of the source file when a box is
  opened; before saving, compare it and decline to overwrite if the file changed since (for example
  if Palworld or another tool wrote it), with a clear reopen/reload prompt.
- [ ] **Test the write-recovery paths.** Cover backup creation, staged-write failure, decode
  failure, replacement failure, cleanup, and manual restore, on Windows and Linux. Every failure
  path should leave either the untouched original or a verified backup, with no orphaned temp file.
- [ ] **Commit a safe save fixture and run it in CI.** Add a sanitized or synthetic
  `GlobalPalStorage.sav` that is safe to distribute, so the real-save round-trip test runs on every
  build instead of self-skipping when `PALBOX_TEST_SAV` is unset.
- [ ] **Add property / fuzz tests for the parser.** Exercise truncated files, unknown properties,
  duplicate fields, and unusual slot states; the engine should fail in bounded, typed ways rather
  than panic.

## Tier 1 — correctness

- [ ] **Derive Partner Skill level from condensation.** Partner Skill level is a function of
  condensation rank; derive it live (`min(5, condensation + 1)`) rather than storing a separate
  copy that can go stale when condensation or species changes.
- [ ] **Surface rank-specific Partner Skill effects.** The reference schema already models
  `partner_skill_rank` (levels 1–5). Carry those rows into the UI bundle and show the active rank's
  effect, so changing condensation also updates the displayed Partner Skill effect.
- [?] **Confirm condensation ↔ Work Suitability coupling.** In 1.0, each rank-up also raises a Work
  Suitability, and reaching max rank raises all of them. Capture controlled before/after saves for
  ranks 0→4, diff `Rank` and `GotWorkSuitabilityAddRankList`, and decide how the editor should
  present the two controls. Do not guess which job increments.

## Tier 2 — trust and hardening (low cost)

- [ ] **Bundle third-party license notices in every release artifact.** Include the required notice
  for `uesave` (and any other bundled dependency), and check for its presence in the release gate.
- [ ] **Synchronize version metadata.** Align the version across `tauri.conf.json`, the READMEs, and
  the crate/package files (currently 1.1.0 vs 1.0.0), or add a release check that fails on a
  mismatch.
- [ ] **Run the UI unit tests in CI and the release gate.** The `test:unit` script exists but is not
  run automatically.
- [ ] **Add dirty-state and close/open guards.** Mark the session dirty after the first edit; warn
  before closing or opening another box with unsaved changes; do not prompt when nothing changed.
- [ ] **Make an explicit CSP decision.** Either define a restrictive Content Security Policy for the
  UI or document why the local threat model makes the current setting acceptable.

## Tier 3 — deliberate refinements (when the area is next touched)

- [ ] **Remove the migration machinery** (per the decision above): delete
  `database/migrations/user-v2-groups.sql`, the `USER_MIGRATION_V2` path and the runtime schema
  migration branch, and the legacy-schema migration tests; recreate the user DB on an incompatible
  schema.
- [ ] **Move domain rules into the core.** New-Pal initialization and DTO application belong behind
  core operations (for example `create_initialized_pal`, `apply_pal_dto`); the Tauri layer manages
  the session and marshals commands.
- [ ] **Single source of truth for limits and computed stats.** Have the core expose the editing
  limits and computed display stats the UI currently mirrors or recomputes; consider generated
  Rust→TS bindings once the DTO surface is stable.
- [ ] **Release provenance.** Publish checksums, a software bill of materials, and dependency-audit
  evidence; consider code signing when the audience and cost justify it.
- [ ] **Performance profile with a near-full box.** Measure startup, reference load, filtering,
  rendering, slot switching, and save time with a roughly 960-slot box; optimize from measurements.
- [ ] **Accessibility, typography, and contrast pass.** Keyboard navigation and focus order, a
  minimum readable type floor, and contrast for small or muted text; test at 1920×1080 with 100%
  and 125% scaling, and on Linux/WebKitGTK.
- [ ] **Contributor / extension guide.** A short data-flow diagram and notes on where new save
  fields, reference data, commands, and UI panels belong.

## Future projects

- [ ] **Self-owned game-data extractor.** Build the reference database directly from the game's
  packaged assets (for example a small CUE4Parse-based utility against `Pal-Windows.pak`),
  replacing the local generator with a repeatable, self-contained pipeline.

## Explicitly out of scope

Kept out on purpose, per the audits and the project's direction: world/`Level.sav`, base, party,
map, or technology editing; a large Tauri/core rewrite; moving app-only groups or tags into the
save; and adding abstraction layers without a concrete second use.
