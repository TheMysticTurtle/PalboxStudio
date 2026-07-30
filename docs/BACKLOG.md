# Palbox Studio — engineering backlog

A prioritized backlog of hardening and refinement work, drawn from two engineering audits of the
project. Items are grouped by tier — save integrity first, then correctness, then trust and
hardening, then deliberate refinements. Each item earns its place by reducing real risk, improving
testability, or making future changes cheaper.

Items come from the audits and should be confirmed against the current code before implementation.

Status key: `[ ]` open · `[x]` done · `[~]` in progress · `[?]` needs investigation first.

## Decisions already made

- **Preserve durable user metadata through numbered migrations.** The user database now holds
  presets, groups, memberships, and app settings that must survive installer upgrades. Add each
  compatible schema change as a tested, forward-only migration; reject databases newer than the
  running app rather than recreating or silently downgrading them.
- **Adopt core-authoritative validation.** The Rust core is the single source of truth for every
  editable limit; the UI restricts values for convenience only. This removes the editing limits
  that are currently mirrored between the core and the frontend.
- **Generated Rust→TS bindings: revisit later.** Worth adopting once the DTO surface is stable and
  the maintenance benefit is clear; not adopted preemptively.

## Tier 0 — save integrity (do first)

- [x] **Register all writable Global Palbox property schemas.** The core now installs an
  insert-only canonical schema registry before every encode, including the Work Suitability array,
  nested enum, and rank paths that caused the reported multi-user failure. Schemas recovered from
  the source save always win.
- [x] **Detect external save changes.** Record a fingerprint of the source file when a box is
  opened; before saving, compare it and decline to overwrite if the file changed since (for example
  if Palworld or another tool wrote it), with a clear reopen/reload prompt.
- [x] **Add an external-change monitor and conflict UI.** The UI polls the core's fresh content
  fingerprint while a box is open, preserves the in-memory copy, blocks Save, and offers an
  explicit discard-and-reload flow when the source changes or disappears.
- [x] **Detect post-save overwrite.** A change observed during the 30 seconds after Studio saves is
  called out separately as a likely Palworld/other-tool overwrite; monitoring continues afterward.
- [ ] **Preserve dirty work during conflicts.** Add a safe app-level snapshot/export so a user can
  keep unsaved edits without overwriting an externally changed source.
- [~] **Test the write-recovery paths.** Success, exact-backup uniqueness, encode failure, and stale
  source refusal are covered. Add deterministic fault injection for staged-write failure, decode
  failure, replacement failure, cleanup, and manual restore, on Windows and Linux. Every failure
  path should leave either the untouched original or a verified backup, with no orphaned temp file.
- [x] **Commit a safe save fixture and run it in CI.** Add a sanitized or synthetic
  `GlobalPalStorage.sav` that is safe to distribute, so the real-save round-trip test runs on every
  build instead of self-skipping when `PALBOX_TEST_SAV` is unset.
- [ ] **Add property / fuzz tests for the parser.** Exercise truncated files, unknown properties,
  duplicate fields, and unusual slot states; the engine should fail in bounded, typed ways rather
  than panic.

## Tier 1 — correctness

- [x] **Derive Partner Skill level from condensation.** Partner Skill level is a function of
  condensation rank; derive it live (`min(5, condensation + 1)`) rather than storing a separate
  copy that can go stale when condensation or species changes.
- [x] **Surface rank-specific Partner Skill effects.** The reference schema already models
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
- [~] **Add dirty-state and close/open guards.** The core session now tracks dirty state and clears
  it only after successful persistence. The UI avoids no-op DTO flushes and warns before opening or
  reloading over unsaved changes. Add the equivalent guard to application/window close.
- [ ] **Make an explicit CSP decision.** Either define a restrictive Content Security Policy for the
  UI or document why the local threat model makes the current setting acceptable.

## Tier 3 — deliberate refinements (when the area is next touched)

- [x] **Make user metadata upgrade-safe.** Numbered v2, v3, and v4 migrations preserve passive presets,
  groups, memberships, the remembered Palbox path, and auto-open preference across upgrades.
  Migration and fresh-schema behavior are covered by core tests.
- [x] **Move domain rules into the core.** New-Pal initialization and DTO application belong behind
  core operations (for example `create_initialized_pal`, `apply_pal_dto`); the Tauri layer manages
  the session and marshals commands.
- [x] **Single source of truth for limits and computed stats.** Patch-sensitive facts, limits,
  progression rows, and formula operands live in the reference DB. The validated in-memory
  catalog drives transactional semantic mutations and engine-computed combat stats, Trust/EXP,
  Partner Skill rank/effects, and Work totals. The UI owns presentation only; see ADR 0004.
- [ ] **Release provenance.** Publish checksums, a software bill of materials, and dependency-audit
  evidence; consider code signing when the audience and cost justify it.
- [ ] **Performance profile with a near-full box.** Measure startup, reference load, filtering,
  rendering, slot switching, and save time with a roughly 960-slot box; optimize from measurements.
- [ ] **Accessibility, typography, and contrast pass.** Keyboard navigation and focus order, a
  minimum readable type floor, and contrast for small or muted text; test at 1920×1080 with 100%
  and 125% scaling, and on Linux/WebKitGTK.
- [ ] **Contributor / extension guide.** A short data-flow diagram and notes on where new save
  fields, reference data, commands, and UI panels belong.

## Quality of life

- [x] **Remember and auto-reopen the last box.** Store the last-opened `GlobalPalStorage.sav` path,
  with a user toggle to reopen it automatically on launch instead of prompting for the file each
  time — so it feels like a proper Palbox companion. Store the path and toggle in app settings;
  validate the path still exists on launch and fall back to the file picker if it doesn't.
- [ ] **Add ascending/descending box sorting.** Add a visible direction toggle beside the sort
  selector with stable tie-breaking and no mutation of save order.
- [ ] **Share explorer state between compact and expanded views.** Search, filters, groups, sort
  key, and direction should persist when switching layouts.
- [ ] **Add Condensation as a sort key.** The value already exists on `BoxPal`; add it after the
  explorer's sorting state is shared.

## Future projects

- [ ] **Self-owned game-data extractor.** Build the reference database directly from the game's
  packaged assets (for example a small CUE4Parse-based utility against `Pal-Windows.pak`),
  replacing the local generator with a repeatable, self-contained pipeline.

## Explicitly out of scope

Kept out on purpose, per the audits and the project's direction: world/`Level.sav`, base, party,
map, or technology editing; a large Tauri/core rewrite; moving app-only groups or tags into the
save; and adding abstraction layers without a concrete second use.
