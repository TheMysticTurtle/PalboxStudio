# Progress & Continuity

Living log of where the build is and what's next. Read this first when resuming.

## Session — 2026-08-02: Pal Awakening and focused editor refinements

- Verified the Palworld 1.0 save representation from the local sanitized Pal extracts:
  Awakening is `bIsAwakening`, a `BoolProperty` independent of condensation's one-based `Rank`.
- Added the Awakened Boolean to the headless engine's read/edit contract and writable-schema
  registry. Enabling it through the semantic mutation also sets condensation to four displayed
  stars, stored as `Rank = 5`; no UI code writes raw save encodings.
- Added a fifth, labeled Awakening Crystal control beside larger condensation stars in the
  Progression drawer. The artwork is the locally bundled Palworld Neutral Awakening Crystal.
- Added focused regression coverage for Boolean read/write, max-rank coupling, missing-schema
  repair, encode/decode, and fresh projection back to the UI contract.
- Newly created Pals now receive explicit 50/50/50 HP/Attack/Defense IVs, uncondensed `Rank = 1`,
  and `bIsAwakening = false`. A missing-schema regression verifies all four values survive a full
  encode/decode cycle.
- Work Suitability controls now consume the engine's species-aware `available` projection. The UI
  shows the species' real jobs plus any nonzero saved exception, instead of all 13 jobs.
- Reordered only the title-bar save actions so the status message and Open Backup grow to the left
  while Save Box remains anchored beside the working-copy badge.
- Added a preset-builder convenience action that copies the selected Pal's current passive skills
  into the unsaved preset draft; the existing Save Preset action remains the only persistence step.

**Verification:** all 35 Rust/core tests and 11 UI unit tests pass; `cargo fmt --check` is clean;
Svelte reports 0 errors and 0 warnings; the production UI build succeeds.

## Session — 2026-07-30: DB facts, engine authority, and semantic Pal views

Completed the authority migration on `fix/engine-save-authority` without changing
the Global Palbox lifecycle or the backed-up atomic save contract:

- Advanced the generated reference DB to schema v4. Typed `editor_limits` and
  `calculation_rules` rows now own patch-sensitive ranges and formula operands;
  Work icons/order, EXP, Friendship, and Partner Skill rank rows are loaded with
  the existing species/move/passive catalog.
- Added a validated, indexed in-memory `ReferenceCatalog`. The database is opened
  once at startup; projections, validations, presets, and box tiles reuse the
  cache instead of issuing render-time queries.
- Added semantic engine input plus engine-owned projections for combat stats,
  Work base/bonus/total levels, Trust, EXP, and Partner Skill level/rank effect.
  The engine alone translates whole HP, food percentage, Trust progress, Work
  bonuses, and condensation's one-based save encoding.
- Added identity-checked, transactional slot mutation. A stale frontend cannot
  write through a reused slot after its `InstanceId` changes.
- Removed the UI combat-stat calculator and duplicated Work/element/category
  catalogs. Main and box cards now consume the same engine projections; species
  changes go through the engine and preserve raw Work bonuses.
- Exposed the complete DB-backed Trust range, including ranks −3 through 10, so
  editing another field cannot silently normalize negative Friendship to zero.
- Advanced `palbox-user.db` to schema v4. Existing v1/v2/v3 databases migrate
  forward without losing presets, groups, memberships, or preferences. The user
  DB preserves ordered preset entries but no longer duplicates Palworld's current
  passive-slot limit; the reference catalog and engine enforce it.
- Added readable SQLite views for species, Work, Partner Skill progression,
  moves/effects, passives/effects, and source provenance. Internal codes and
  human-readable names appear together for direct DB Browser inspection.
- Recorded the final authority boundary in ADR 0004.

**Verification:** 32 Rust/core tests pass; 11 UI unit tests pass; Svelte checking
reports 0 errors and 0 warnings; the production UI build succeeds; deterministic
reference/user database generation and installed-database validation pass.

**Next:** no additional architecture rewrite is expected for this slice. The next
engineering priority remains deterministic persistence fault injection, followed
by the window-close dirty guard and a safe dirty-conflict snapshot/export.

## Session — 2026-07-30: consistent cards and verified condenser controls

- Standardized compact Global Palbox cards at 190px, enough for the optional species subtitle,
  condensation count, and group tag without producing staggered rows.
- Standardized species-picker cards at 250px, enough for Work Suitability chips to wrap to a
  second row without making that species taller than its neighbors.
- Replaced font-dependent male/female characters with one reusable stroked SVG icon across compact,
  expanded, and editable Pal cards. The badges retain their existing gender color and accessible
  label while rendering consistently in the desktop WebView.
- Audited the current source of Palworld Save Pal and the other Pal editors used as references.
  Their terminology confirms that “ascension,” “rank,” “condensation,” and the Pal Essence
  Condenser refer to the same progression field. PSP's normal control displays four stars and
  writes `pal.rank = displayed stars + 1`; Pal Souls and Work Suitability are separate controls and
  save properties.
- Kept the existing progression control instead of introducing a conflicting duplicate. It is now
  labeled **Pal Essence Condenser** and reports both the user-facing 0–4 stars and save-facing
  `Rank` 1–5. It does not silently rewrite Work Suitability bonuses; the controlled-save coupling
  question remains open below.

**Verification:** 12 UI unit tests pass; Svelte checking reports 0 errors and 0 warnings; the
production build succeeds.

## Session — 2026-07-29: durable preferences and roomier launch

- Increased the default desktop window from 1280×800 to 1440×900 so the compact Global Palbox
  retains more usable matrix space after adding the Last Palbox control. The existing 1024×680
  minimum is unchanged, so users can still resize the window down.
- Advanced `palbox-user.db` to schema v3 with a dedicated `app_setting` table. The engine now owns
  the remembered Global Palbox path and auto-open preference alongside presets and groups.
- Existing values stored under the old webview local-storage key are imported into SQLite once,
  then removed only after a successful database write. Fresh installs and upgrades use the same
  authoritative preference commands.
- The v3 migration is idempotent so concurrent preset/group/preference reads during startup cannot
  race the schema upgrade.

**Verification:** 28 core tests cover preference durability, normalization, concurrent startup,
v1→v3 migration, and v2 user-metadata preservation;
12 UI unit tests cover legacy-import selection. Svelte checking and a production build remain the
release gates.

## Session — 2026-07-29: seamless reopen and source-conflict monitor

- Added a compact **Last Palbox** row at the top of the Global Palbox drawer. A remembered path can
  be opened in one click, and the adjacent **Open on launch** toggle automatically reopens it on
  future starts. An invalid/moved path disables auto-open and falls back to the normal picker
  without trapping the user in a repeated startup error.
- The last-box preference is app metadata, not Pal save data. It is updated only after a successful
  open and is now persisted through the engine-owned user database described above.
- Added an always-on source monitor while a box is open. The shell asks the core for a freshly
  hashed source status every 1.5 seconds; source changes or disappearance preserve the in-memory
  copy, block Save, show a clear conflict banner, and offer an explicit discard-and-reload action.
  The core still rechecks the fingerprint immediately before replacement and remains authoritative.
- A conflict detected during the 30 seconds after Studio saves is identified separately as a
  likely Palworld/other-tool overwrite.
- Slot selection now flushes a full DTO only when the selected Pal actually differs from its loaded
  baseline. Merely viewing or switching clean Pals no longer marks the engine session dirty.
- Opening another box or reloading a conflict warns before discarding unsaved in-memory edits.

**Verification:** core session tests cover current/changed source status; UI unit tests cover
legacy preference parsing/import and post-save conflict classification.

**Next:** complete deterministic Tier 0 persistence fault injection, then add the remaining
window-close dirty guard and dirty-conflict snapshot/export.

## Session — 2026-07-29: engine-owned schemas and safe save session

Started `fix/engine-save-authority` from the isolated condensation fix
(`fix/condensation-rank-encoding`, commit `e10ee6f`) and addressed the reported multi-user Work
Suitability save failure:

```text
write_sav: missing property schema for path:
SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.WorkSuitability
```

- Added an insert-only core registry for every property schema Palbox Studio can currently create.
  The registry is invoked inside `write_sav`; schemas read from the source file always win.
- Reproduced the exact nested-property error by stripping the source schemas, then proved the core
  write boundary repairs it. First-row, canonical-order, zero-removal, invalid-name, and range
  regressions now cover the Work Suitability mutation.
- Added `core/tests/fixtures/synthetic-global-palbox.sav`: a 960-slot sanitized fixture containing
  one synthetic `CubeTurtle` and no user identity data. Save-dependent tests no longer self-skip in
  normal local or CI runs.
- Moved whole-DTO application, documented limit enforcement, species-dependent Work Suitability
  validation, and new-Pal initialization into the Rust core. Tauri now marshals those operations.
- Added the headless `SaveSession`, which owns the parsed save, dirty state, SHA-256/size/mtime
  source fingerprint, schema preparation, encode/decode validation, verified unique backup,
  synced/decoded temporary file, second stale-source check, atomic replacement, and post-save
  fingerprint refresh.
- A source changed after open is refused before backup/replacement; a source changed during staging
  is refused before replacement. Encode failures leave the original untouched and create no backup.
- Pruned five merged local feature branches and the two corresponding obsolete remote branches.

**Verification:** `cargo test` runs 25 core tests plus the shell/doc tests with no skipped
save-dependent cases. The reported schema failure and a no-schema first bonus both round-trip on the
committed fixture and on a scratchpad copy of the current real Global Palbox.

**Still to centralize:** replace the UI's full DTO submission with granular typed engine mutations;
return computed combat stats, trust/EXP, Partner Skill rank/effect, Work Suitability totals, and
editing limits from the engine; add watcher/post-save overwrite UX and fault injection for every
staged-write/replacement failure.

**Recommended next slice:** finish the Tier 0 persistence contract with deterministic fault
injection for staged writes, staged decode, atomic replacement, cleanup, and manual restore. Then
add watcher/conflict and post-save-overwrite UX on top of the fingerprint authority already in the
core. After that safety boundary is closed, continue the authority migration with granular typed
engine mutations and engine-computed display values.

## Session — 2026-07-28: condensation rank encoding fix

Fixed the condensation off-by-one at the core save boundary. Palworld's `Rank` byte is one-based
(1–5), while the game and editor display 0–4 stars. The engine now subtracts one on read and adds
one on write, including `Rank = 1` for an uncondensed Pal, with regression coverage for every rank.
The consolidated save-format, specs, and quick-reference docs now state both domains explicitly.

**Why we got confused:** during the documentation consolidation, the displayed star count and raw
save value were accidentally collapsed into one 0–4 range. The resulting docs said `Rank` was 0–4
and should be omitted at zero, and the first Rust setter implemented that description literally.
The older PalEdit adapter and the separate `palworld-reference` notes already had the correct
translation (`display = Rank - 1`, `Rank = display + 1`), but that distinction did not make it into
Palbox Studio's new engine contract. Existing tests only proved that our reader and writer agreed
with each other; they did not exercise every condensation value against the game's one-based
meaning. The new boundary test covers all five values and asserts both representations explicitly.

**Released-save safety audit:** version 1.1.0 did not clamp a loaded raw `Rank = 5` to 4. The old
core reader passed 5 through, `dtoToPal` and `palToDto` both copied it unchanged, and the old setter
wrote 5 back. Saving flushes the selected Pal's full DTO, but even an unrelated edit therefore
preserved its original condensation byte; unselected Pals were never passed through the mutation
path at all. A max-condensed Pal could drop from raw 5 to raw 4 only when the user clicked a
condensation star (including the apparently already-selected fourth star), which explicitly
replaced the value with the UI's 1–4 button value. The released bug affected display, Partner Skill
level/stat previews, and deliberate condensation edits — it did not silently lower condensation
across an opened or otherwise edited box.

## Session — 2026-07-27: documentation consolidation + audit backlog

Consolidated the project's reverse-engineering and data knowledge into its own documentation, and
stood up a prioritized backlog from the two engineering audits.

**Done — committed `8ee2b09` (local, not yet pushed):**
- New `CLAUDE.md` (project front door) and `docs/SAVE-FORMAT.md` (the 1.0 save-format layout,
  per-Pal fields, save-field map, slot model, and corruption traps, in one reference).
- QUICKREF, SPECS-1.0, DATA-AND-ASSETS, DIRECTION, RESEARCH-landscape, and `reference/README` now
  point at the in-repo references.
- `scripts/build_reference_db.py` untracked (kept locally); the reference DB
  (`data/palbox-reference.db`) ships prebuilt and committed, with provenance/attribution recorded
  in `database/README.md`.
- Tidied related `core/src` comments.

**Decisions this session (captured in [BACKLOG.md](BACKLOG.md)):**
- Remove the speculative user-DB migration machinery; recreate on an incompatible schema.
- Adopt core-authoritative validation (single source of truth for editable limits); revisit
  generated Rust→TS bindings later, only if they earn their keep.
- A self-owned game-data extractor (e.g. CUE4Parse against the game's packaged assets) is future
  work.

**Next up:** work the backlog in [BACKLOG.md](BACKLOG.md), Tier 0 (save integrity) first. A fuller
reconciled write-up of both audits can still be added as `docs/AUDIT-2026-07-27.md` if useful.

## Packaging handoff

Current working branch: `feature/groups-passive-presets`.

The editor is a functional pre-release, not a UI scaffold. The next task is Windows
installer/bundle validation through the existing Tauri 2 configuration. Start by reading this
file and the three ADRs in `docs/decisions/`; do not replace the working architecture or introduce
sample data. The expected pre-bundle gate is:

```powershell
npm.cmd run check
npm.cmd run build
cargo fmt --all -- --check
cargo test
python scripts/build_reference_db.py --check
npm.cmd run tauri build
```

`src-tauri/tauri.conf.json` already enables bundling, declares the desktop icon set, and packages
`data/palbox-reference.db` as a runtime resource. Confirm the generated installer launches without
the repository, loads that packaged reference DB, creates the writable user metadata DB under the
app data directory, opens a copied `GlobalPalStorage.sav`, and can reveal its verified backup.
The save boundary is non-negotiable: encode/decode validation → unique byte-verified backup →
synced/decoded temp file → atomic replacement. Do not weaken it to make packaging easier.

Two untracked PNGs under `assets/` are design concepts, not required runtime assets:
`Expanded Global Palbox Card Design Concept.png` and `Main Focued Palcard Design Concept.png`.
Do not accidentally include or commit them during bundling.

## UI polish checkpoint — collapsible compact Pal tags

The compact Global Box keeps the group filter visible but collapses the selected-Pal tag editor
to a single summary row by default. The row shows the selected Pal, assigned-tag count, and a clear
chevron; expanding it restores the full shared create/manage/assignment control. This keeps the
tile matrix visually dominant without removing tag management.

## UI polish checkpoint — tag popover and compact Global Box

Branch: `feature/groups-passive-presets`.

The shared Pal-tag control now has a measured responsive height instead of relying on its
initially collapsed `scrollHeight`. Its header, create form, create/manage footer, and error
message are fixed regions; only the tag rows scroll. When tags exist, the picker reserves room
for at least three rows plus the create/manage button and chooses above/below placement from that
real target height, preventing the footer from covering entries near the bottom of the editor.

The compact Global Box now always exposes the same tag control while a box is open. Global tag
creation and management remain available before selecting a Pal; assignment rows stay disabled
until a Pal is selected, then target that Pal's stable `InstanceId`. The compact tile scroller also
has ten pixels of top breathing room so the first row's hover lift and shadow are not clipped.

## Feature branch checkpoint — clean tag baseline and verified save backups

Branch: `feature/groups-passive-presets`.

The pre-release user metadata database now starts from the canonical schema already present in
`database/user-schema.sql`. The incompatible local prototype database discovered during tag
testing was preserved as a dated `.bak` and replaced with a clean database; no speculative
compatibility layer was added. Canonical integration tests cover creating reusable tags,
case-insensitive name uniqueness, assigning multiple tags by stable Pal `InstanceId`, replacing
assignments atomically, renaming, and cascading membership cleanup on deletion.

The selected-Pal gender control is now a large labeled **Male**/**Female** pill with an explicit
vector symbol, keyboard focus treatment, and a tooltip describing the next value. It retains the
existing one-click toggle behavior while making the control readable at normal viewing distance.

Every **Save Box** operation now encodes and decodes the edited payload before touching the source,
creates a collision-safe backup in the save's sibling `PalboxStudio-backups` folder, syncs it, and
verifies it byte-for-byte against the current source. A backup failure aborts the save before the
source can be changed. The replacement is likewise staged, synced, byte-checked, decoded, and then
atomically renamed. After a successful save, **Open backup** reveals the exact backup file so users
do not have to discover the folder manually.

## Feature branch checkpoint — groups and passive presets

Branch: `feature/groups-passive-presets`.

The writable user database was schema v2 at this checkpoint. Migration `user-v2-groups.sql`
upgrades existing v1 databases in place, adding named groups plus many-to-many membership keyed by
a Pal's stable `InstanceId`. Schema v3 later adds durable app settings. Groups remain Palbox Studio
metadata and never enter `GlobalPalStorage.sav`.
Rust owns name/foreign-key/membership validation; deleting a group cascades only its app-owned
memberships. Core tests cover v1 migration, case-insensitive uniqueness, atomic assignment
replacement, and cascade behavior.

The main card and both Global Palbox views share one compact tag picker. Its anchored dropdown
selects from every reusable tag without blocking or overflowing the Global Box sidebar; a
secondary create/manage view owns tag creation, rename, and deletion. Compact cards show the
first tag, expanded cards show every tag, and both box views can filter for Pals belonging to
every selected tag.

The existing passive-preset engine now has its UI. Users can build or edit a named preset with up
to four unique reference-backed passives through the full passive filter, delete presets, and
apply saved presets from buttons directly below the selected Pal's passive list. Applying changes
the live in-memory Pal; the normal backed-up **Save Box** operation remains the only write to the
Palworld save.

The main-card and preset-builder passive pickers are the same shared component with the same
full-catalog default. New enabled entries such as Lunker therefore appear from the canonical
in-memory reference data in both places; species, normal-pool, and lucky-pool scopes remain
available as optional filters.

## Feature branch checkpoint — readable card and selector scale

Branch: `feature/readability-polish`.

The card system now has a documented semantic type scale in `tokens.css`; routine gameplay
information no longer drops below 11px. This pass raises the selectors, compact Global Palbox
tiles, expanded trading cards, main editor metadata, active-skill control bindings, passives,
Work Suitability controls, and Advanced drawer labels together instead of applying isolated
one-off zooms.

The species selector uses a wider responsive dialog and 185px minimum card width, with 82px
portraits, 22px element emblems, larger suitability icons and levels, and 35–36px filter targets.
At a roughly 950px application window it intentionally shows four readable columns instead of
five compressed columns. The compact drawer widened to 480px, while the expanded Global Palbox
grid now targets 325px cards so names, calculated stats, moves, suitability levels, passives, and
groups stay readable without losing the trading-card layout.

Browser QA covered the full species dialog at 950×655, both Global Palbox card densities, and a
populated main editor at 1400×900. The temporary QA-only route was removed afterward.

Selector card rows are content-sized and the localized species-name row is non-shrinking. This
prevents the enlarged portrait, element, and Work Suitability content from collapsing the species
name when the chooser contains the full 287-species production roster.

## Feature branch checkpoint — species mini-cards and reliable skill ordering

Branch: `feature/pal-image-filter-polish`.

The species selector now renders a dedicated `SpeciesMiniTile` from the same canonical
`SpeciesRow`, portrait resolver, element theme colors, element emblems, and Work Suitability icon
registry used by the rest of the application. Each tile shows only selection-relevant identity
data: the portrait, localized species name, elements, and nonzero suitability levels.

Active/inactive skill dragging no longer depends on HTML `DataTransfer`, which was advertised by
the cursor but unreliable inside WebView2. `PalCard` now tracks a five-pixel pointer-drag gesture,
resolves the row under the pointer, and passes every bench/equip/reorder operation through the pure,
unit-tested `moveSlots.ts` ordering engine. Moving a bench skill into a full active set returns the
displaced third skill to the bench without duplication or loss. Browser interaction QA verified
bench-to-active movement and active-slot reordering; the temporary QA route was removed afterward.

The three active rows also show their default mounted bindings: **RMB** for Mounted Skill 1,
**E** for Mounted Skill 2, and **C** for Mounted Skill 3. They are explicitly presented as defaults
because Palworld allows the player to rebind controls.

## Feature branch checkpoint — canonical Palbox roster and application identity

Branch: `feature/pal-image-filter-polish`.

The full 1.0 species source was audited end to end. Its 406 `is_pal` rows include real species plus
raid body parts, summon/predator/oil-rig encounters, tower actors, quest helpers, appearance
variants, retired models, and uncatchable bosses. Reference schema v2 retains all 406 for decoding
but marks exactly 287 unique owned-Pal species as `palbox_selectable`; the species selector shows
only those rows. Seventy-three variant codes map to canonical owned species through
`species_alias`, so loaded oddities still receive the correct name, data, and portrait.

The generator now fails if selectable names duplicate, if canonical Xenovader/Xenogard/Xenolord
are absent, or if Boltmane, Astralym, or Moon Lord leak into the chooser. Xenovader, Xenogard,
Xenolord, and the other normal Paldeck rows are classified Natural even when the game data also
marks that species as eligible for predator encounters.

`assets/Palbox Studio Logo.png` is now the authored identity source. The clean artwork was used to
regenerate the complete desktop Tauri icon set plus the deployed web logo and favicon. Runtime
components resolve it through `APP_LOGO_ART`; the checkerboard-backed predecessor is no longer used.
Because reference data is cached at startup, a running development app must restart after a DB
regeneration to see roster changes.

## Feature branch checkpoint — Pal artwork, match-all filters, and Soul percentages

Branch: `feature/pal-image-filter-polish`.

Pal portraits now render through `PalArtwork.svelte`, a single clipped artwork surface used by
the compact cards, expanded cards, species selector, and main editor card. It gives every source
image the same fixed-size frame, crop, corner treatment, fallback behavior, and subtle zoom without
creating a second set of derived image files or adding per-species layout exceptions.

The shared species filter now has explicit intersection semantics: a Pal must match every selected
value within a facet and every other active facet. Category buttons are mutually exclusive because
a species cannot simultaneously be Natural, a Tower Boss, and Unobtainable. The pure matching
module has focused unit tests and remains the source for both Global Palbox views and the species
selector.

Statue of Power enhancement ranks are 0–20 at +3% per rank, or +60% per stat at rank 20. The
Advanced drawer shows rank and actual percentage together; the main card summarizes the four real
percentages instead of adding unlike ranks into one number. Shared constants drive UI limits,
combat-stat calculations, and the Rust boundary.

## Feature branch checkpoint — modular Pal cards

Branch: `feature/modular-pal-cards`.

The Global Palbox now uses one data-driven presentation pipeline rather than independent card
implementations. `palPresentation.ts` resolves save codes against the in-memory reference bundle
once for every card density. `BoxTile.svelte` is a small dispatcher over:

- `CompactPalTile.svelte` — portrait, localized nickname/species, level, gender, elements,
  condensation, variants, and the first assigned group.
- `ExpandedPalCard.svelte` — the trading-card view with calculated Max HP/Attack/Defense, only nonzero
  Work Suitabilities, the three equipped moves, every passive, variants, and groups.

The box engine summary now exposes the fields needed by those cards from the full Pal record it
already reads; expanding the box does not issue one command/query per Pal. The expanded view also
has its own nickname/species search, shared species facets, and box-order/name/level sorting.

Artwork is resolved centrally through `icons.ts`: Pal portraits (including aliases/fallback),
element badges (using the canonical icon basename from SQLite), Alpha/Lucky emblems, and Work
Suitability icons. Shared `PalPortrait`, `ElementIcon`, and `WorkIcon` components keep every card
density on that contract. Reference-backed element colors now flow through the presentation layer;
CSS token colors are startup fallbacks only.

The main editor card now uses that same presentation engine for species identity, elemental
coloring, artwork, variants, and calculated Max HP/Attack/Defense. Its trading-card hero sits above
the actual editing workspace; species, nickname, gender, Alpha/Lucky, level, vitals, trust,
passives, moves, and Work Suitability remain the existing live save controls rather than display
copies. Clicking the gender badge toggles Male/Female through the existing DTO flush/save path.
The zero-state uses the same layout with zeroed, inert controls and the Palbox Studio logo.

Both box grids use content-sized implicit rows. Compact tiles keep a stable minimum height, while
expanded rows grow to the tallest card in that row so variable Work Suitability, move, and passive
counts remain inside the card instead of being clipped.

Expanded cards show calculated Max HP, Attack, and Defense instead of IVs. The shared
`palStats.ts` calculator combines cached species scaling with the Pal's level, IVs, Soul ranks,
condensation, Alpha HP bonus, and static self-targeted passives; runtime party/equipment/server
modifiers are intentionally excluded. IVs remain editable in the Advanced drawer.

The Advanced drawer's old emoji placeholder is replaced by bundled Statue of Power artwork.
Its path is registered in `icons.ts`, keeping the asset swappable from the same centralized
artwork contract as Pal portraits, elements, variants, and Work Suitability icons.

## Session 4 (2026-07-25) — real-save editing and editor controls

**The save-editing path has completed its first real game test.** A Pal added through Palbox
Studio was written to a backed-up Global Palbox save, loaded into Palworld, and appeared in the
player's box. Slot selection is now a numeric storage-slot concern; a Pal's GUID `InstanceId`
remains identity data and is no longer overloaded as a UI selection key.

**The fake preview state is gone.** Starting without an open save shows an empty box and a
zeroed, inert main card with the Palbox Studio logo. Selecting a real Pal populates that same
card directly. The old `samplePal.ts` / `sampleBox.ts` fixtures were removed rather than hidden
behind save-only routing or a separate blocking screen.

**The main editor now exposes the requested real fields and selectors.**

- Move selection is split into the species' default learnset, Skill Fruit moves, and all moves.
  Active and inactive moves can be clicked or dragged between lists and reordered.
- HP, sanity, food, and trust progress retain numeric entry and also use range controls; trust
  rank is editable separately. Newly created Pals initialize with full HP, sanity, and food.
- Work Suitability only renders entries the selected species can actually possess.
- Species, Alpha, and Lucky are editable on the main card. Pal tiles use species display names
  plus the Alpha/Lucky emblems; Lamball's `Sheepball` data code is explicitly mapped to the
  case-sensitive `SheepBall` portrait asset.
- The passive reference builder fills blank descriptions and normalizes structured effects into
  short plain-English descriptions instead of exposing raw localization markup.

**Element placeholders have been replaced.** The nine retained game-style element badges now
live under `ui/static/icons/elements/` and render through one `ElementIcon` component in species
tiles, species/move filters, Partner Skills, main-card element pills, and active/inactive move
rows. CSS element colors remain responsible only for surrounding emphasis.

**Developer launch path:** double-click `Launch Palbox Studio.cmd` from the repository root.
It validates the local toolchain/dependencies and starts the Tauri development app; packaged
installer/executable builds remain a later release task.

**Verification:** `npm run check` passes with 0 errors / 0 warnings, `npm run build` passes,
all 11 Rust tests pass, the reference DB generator check passes, and the launcher self-check
passes.

## Session 3 (2026-07-25) — normalized 1.0 reference database

**Static SQLite is live.** `scripts/build_reference_db.py --check` generates
`data/palbox-reference.db` from the local PSP 1.0 extract plus retained source snapshots.
Schema: `database/reference-schema.sql`. Source rows: 406 Pal-shaped engine actors, 351 moves, 420 passives,
2,372 items, 348 species-to-Partner-Skill records (287 direct cards plus 61 audited
same-name/same-tribe engine variants), 42 Ranch-product links across 29 species, and 59,192
localizations. The builder records source hashes/provenance and 94 non-fatal audit findings,
including 27 game-dump learnset moves without definitions, two blank external descriptions,
two corrected source-image codes, 61 inherited variant relationships, the stale Wiki Sibelyx
079 row, and the old “Pal Fluids” label. Integrity, foreign-key, source-hash, and
expected-count checks pass.

**Reference material is retained.** Raw HTML/API responses and normalized derivatives live in
`data/reference-sources/`, documented with authority rules and hashes. They are not temporary
scrape files.

**Desktop data path switched from JSON to SQLite.** The generated DB is a Tauri resource,
opened read-only by `palbox-core`; `get_reference_data` returns the UI bundle. Partner Skill
names/descriptions and Ranch products now reach the UI. `ui/static/data` remains only as a
plain-browser visual-preview fallback.

**Passive preset foundation is wired end-to-end.** `database/user-schema.sql` enforces ordered
slots 0–3; the real `palbox-user.db` is created in app data and contains no per-Pal state.
Rust validates every passive code against the reference DB. Tauri commands list choices, list/
save/delete presets, and apply a preset directly to the addressed in-memory Pal. Saving the box
is still a separate explicit backed-up operation. ADR 0003 records the split.

**Verification:** `python scripts/build_reference_db.py --check` passes; `cargo test --workspace`
passes 7 tests; `npm run check` passes with 0 errors / 0 warnings.

## Session 2 (2026-07-24 pt.2) — UI build-out
**New reference (use it):** `../palworld-reference/` (sibling to PalboxStudio) —
`pal-data.md`, `save-and-file-format.md`, `paledit-code-map.md`, README. Filter categories,
move/passive legality, the 13 work suits, per-pal save fields.

**Done this session:**
- Real assets + data: 379 pal icons + 13 work icons in `ui/static`; card renders the real
  Incineram (CodeName Baphomet) + real work-suit icons. WORK_SUITS → **13** (added Crude Oil Extraction).
- **Advanced drawer complete**: IV sliders (0–100), Statue of Power per-stat souls
  HP/ATK/DEF/WS (0–20, ±/clickable pips), condensation stars (0–4). Model gained `ivs` +
  `soulRanks` (per the PSP DTO map).
- **Global Box drawer**: Open-file button (`tauri-plugin-dialog`, defaults to
  `%LOCALAPPDATA%/Pal/Saved/SaveGames`), search + element + group filters, 3-col tile matrix,
  **Expand → full `BoxMatrix` overlay** of large tiles. `BoxTile` (sm/lg), `sampleBox`
  (real species). Left drawer opens on load.
- **RULE — humans/NPCs are EXCLUDED entirely** (can't live in a global box). Base filter on
  every species/box list is `Human !== true`; never show them as clutter or a filter bucket.

**Passives/moves now RESOLVE FROM CODES (fixed):** a pal stores passive/move codes;
`refdata.svelte.ts` loads the reference tables once and resolvers turn codes → name/rating/
description/element/power. `PassiveChip` takes a code; `PalCard` resolves moves; `samplePal`
uses real codes (`Legend`/`PAL_ALLAttack_up2`/`PAL_Sanity_Down_2`, learnset move codes). Verified.
Data audit: "Lunker" IS present (code `Nushi`); vendored PSP snapshot = v1.2.0 (2026-07-19).

**Engine + data-flow plan = ADR 0002** (`docs/decisions/0002-...md`): three data planes (static
reference JSON / editable save model / mutable app-data SQLite), the Tauri command contract,
the `PalDto` (raw-editable vs engine-computed stats), the backup→atomic-write save flow, and the
SQLite schema for groups/tags/presets. Build the UI against those DTO shapes so the engine drops in.

**ENGINE STARTED — I/O + read proven on the REAL save.** `core/` now depends on oMaN-Rod's
`uesave` fork (rev pinned = PSP's known-good commit) with the **`oodle` feature — it BUILDS and
works on this machine**. Modules: `save.rs` (`read_sav`/`write_sav` = Oodle/GVAS decode/encode,
lossless round-trip), `ue.rs` (uesave→Palworld type aliases + our own accessors), `globalbox.rs`
(`slot_count` + `list_pals` over `SaveParameterArray`). **Global-box ONLY** (no world/Level.sav).
Verified on a scratchpad COPY of the owner's real `GlobalPalStorage.sav`: **960 slots / 61 pals**
read correctly; round-trip re-decodes. `cargo test -p palbox-core` = 4/4 (set env
`PALBOX_TEST_SAV` to the copy; NEVER touch live saves).

**ENGINE DONE + UI WIRED — the core loop works.** `core/src/pal.rs`: full `PalDto` read + a
setter (edit port) for every field (level/nickname/gender/ivs/souls/condensation/lucky/passives/
equipped-moves). `src-tauri`: Tauri commands `open_box`/`get_pal`/`update_pal`/`save_box`
(session state + **backup→atomic write**). Verified on the real save (read full DTO, edit level
55→80, save round-trip; `cargo test -p palbox-core` 6/6). UI wired via `engine.ts` (invoke),
`mapper.ts` (dtoToPal/palToDto join species), `stores/box.svelte.ts`: **Open Global Palbox →
real tiles → select → card shows the real pal → edit → Save Box (backup + write)**. Build +
svelte-check 0/0. Run `npm run tauri dev` to use it.
**Still pending:** work-suitability edit setter (array-of-structs; read works, edit doesn't
persist yet), the computed combat-stat FORMULA (Attack/Def/WorkSpeed + max HP shown are
placeholders = base scaling), groups/tags SQLite, schema-driven filters + species selector,
nickname-clear edge, in-game icon assets. `samplePal`/`sampleBox` are now fallback fixtures only.

**Historical JSON checkpoint (superseded by SQLite):**
`ui/static/data/{species,moves,passives,elements,schema}.json` via `scripts/gen_species.py` —
406 Pal-shaped engine rows + 351 moves + 420 passives + 9 elements,
validated (0 dangling), provenance in each `_meta`. Docs in DATA-AND-ASSETS.md. Static reference
= JSON-in-git; mutable user data (groups/tags/presets) = **SQLite later** (like psp-db).

**Next:** (a) wire the **data-driven filters** — feed `GlobalBoxDrawer` + a new species-selector
off `species.json` + `schema.json` (generate filter controls from the schema; replace the
hardcoded element/group filters); (b) scrape **partnerSkill + farmDrops** (the two GAP columns)
from a wiki into the generator; (c) SQLite store for groups/tags/presets. Then wire the Rust
engine (load `GlobalPalStorage.sav` → real box + computed pal stats;
expose `core::limits` + game data via Tauri commands); species selector popup (click the
species name → filterable **storable-species** list w/ icons); real groups/tags create/rename;
central-card glow goals. NOTE: the open-file button needs a `tauri dev` rebuild to work
(dialog plugin is compiled in).

## Status — end of session 2026-07-24

**Done**
- Docs baseline: [DIRECTION](DIRECTION.md), [SPECS-1.0](SPECS-1.0.md) (authoritative value
  ranges), [DESIGN-HANDOFF](DESIGN-HANDOFF.md), [QUICKREF](QUICKREF.md), [ADR 0001]
  (decisions/0001-rust-core-tauri-svelte.md), [RESEARCH](RESEARCH-landscape.md).
- Toolchain (all installed this session): **Rust stable-msvc**, MSVC C++ Build Tools 2026,
  WebView2, Node 24. `cargo` lives at `~/.cargo/bin` — NOT on PATH in fresh shells; prepend
  `export PATH="$HOME/.cargo/bin:$PATH"`.
- **Scaffold builds green** → `target/debug/palbox.exe`. Cargo workspace:
  - `core/` (`palbox-core`) — headless engine; seeds `limits` (1.0 ranges) + tests.
  - `src-tauri/` (`palbox`) — Tauri v2 shell; `core_version` bridge command.
  - `ui/` — SvelteKit (static/SPA) + Vite; Svelte 5 runes.
- **UI so far** (all lifted to the prototype's exact CSS + self-hosted Rajdhani/Barlow fonts):
  - Frameless window (`tauri.conf` `decorations:false`) + custom title bar (`TopBar.svelte`,
    drag region + working min/max/close).
  - Tri-panel shell: `Drawer.svelte` (generic, purple "box" / cyan "advanced" tones, real
    edge tabs), `Backdrop.svelte`, `+page.svelte` composes it.
  - Center **`PalCard.svelte`** — header (editable name, gender, element pills, PRESETS,
    Favorite), NEXT exp, 3-col body: Partner Skill + Passives (left); portrait + Level editor
    (clamped 1–80) + equipped moves (center); Stats incl. HP/Attack/Defense/Work Speed/SAN/
    Food/Trust + all 12 Work Suitability rows (0–10 steppers) (right).
  - Reusable: `SectionHeader`, `ElementPill`, `PassiveChip`, `WorkSuitRow`.
  - Data layer (`ui/src/lib/data/`): `types.ts`, `constants.ts` (`LIMITS` mirrors
    `core::limits`, `ELEMENT_COLOR`, `WORK_SUITS`, helpers), `samplePal.ts` (Incineram
    placeholder — no save-loading yet). Drawer state: `stores/ui.svelte.ts`.

## Next (in rough priority)
1. **Moves: bench + drag** (owner asked). Add the scrolling **AVAILABLE MOVES** list under the
   equipped slots, and HTML5 **drag-to-equip / drag-to-unequip** (bench→equip zone equips,
   swapping the oldest when 3 equipped; equipped→bench unequips). Equipped max = 3
   (`LIMITS.equippedMovesMax`). Exact markup: prototype `Palbox Studio.dc.html` lines 160–191.
   Needs a bench-moves list in the pal model.
2. **Right drawer (Advanced)** — IV/breeding sliders (0–100), Statue of Power (Pal Souls
   per-stat 0–20 rank bars + actual percentage + statue image), Souls & Condensation cards,
   amber backup warning.
   Prototype lines 308–393.
3. **Left drawer (Global Box)** — tile grid (matrix/list toggle), search/filter/sort, groups
   & tags, add/clone/delete, filter modal. Prototype lines 255–306 + 396+ (READ the rest of
   the `.dc.html`, lines 397–556, not yet reviewed). Groups = user-named, filter-like; tags
   bind pals to groups; **app-side metadata keyed by InstanceId, NEVER written to the `.sav`**.
4. **Wire the Rust core** — load/write `GlobalPalStorage.sav`, replace `samplePal` with real
   data, make editors persist (backup-before-write). Expose `core::limits` via a Tauri command
   so `LIMITS` isn't duplicated on the TS side.

## New requests (2026-07-24, late) — top of next session
1. **Window won't drag** (frameless; resize works, drag doesn't). Cause: `data-tauri-drag-region`
   needs the window permission `core:window:allow-start-dragging`; the min/max/close buttons
   need `allow-minimize` / `allow-toggle-maximize` / `allow-close`. **Added to
   `src-tauri/capabilities/default.json` this session — VERIFY in `npm run tauri dev`.** If still
   stuck: ensure the title-bar element (incl. its empty/spacer areas) carries
   `data-tauri-drag-region` and no child overlays it.
2. **Species selector popup** (change a pal's species). Clicking the **species name on the card**
   opens a filterable/searchable list of every species/entity that CAN live in the Global
   Palbox. **Superseded rule:** excluding humans alone proved insufficient; use reference-schema
   v2's audited `palbox_selectable` value so encounter actors and uncatchable bosses are excluded
   as well. Selecting one changes the species. Needs: a distinct,
   clickable **species label** on the card (separate from the editable nickname input), a
   species list source (psp `data/json/pals.json` + l10n names), and the picker component
   (reuse for the box filter later). Owner suggested clicking the species name to trigger it.
3. **Left drawer opens by default** — done: `stores/ui.svelte.ts` `leftOpen: true` (pops out on
   load).
4. **Real-data placeholder + real pal icons.** Replace the made-up `samplePal` with a REAL pal
   built from PSP data (`PalEdit/psp-reference/data/json/pals.json` — real species, scaling
   stats, elements, work_suitability, skill_set; l10n display names), shaped to our `Pal` model
   so it's "ready to wire" and shows real numbers. Use the actual **pal icon asset** per species
   (PalEdit convention `T_<Code>_icon_normal.png` from the paldb CDN — bundle locally for
   offline; see PalEdit `update_data.py --icons` / `GetImage`). **First check if icons already
   exist under `psp-reference`.** Map via the PSP DTO field list above.
5. **Central card visual goals** (iterate later — owner wants it to "look sweet"; not urgent):
   - Portrait shows the pal image with a **faint glow tinted to its PRIMARY element**. For a
     DUAL-element pal, split it: **top half of the glow = element 1 color, bottom half =
     element 2 color**.
   - **Lucky** → a larger **blue** glow emanating across the rest of the card background.
   - **Alpha** → same idea but **red**.
   - Expect many more tweaks to the center card over time.

## PSP pal field map — our "does it map cleanly?" checklist
Source of truth for stats = PSP's `PalDto` (`PalEdit/psp-reference/psp-core/src/dto/pal.rs`).
Our UI `Pal` model maps cleanly for most fields; fixes to make while wiring the core:
- **Pal Souls are PER-STAT**: `rank_hp / rank_attack / rank_defense / rank_craftspeed` (each
  0–20). Our model has a single `souls` → change to `{hp,attack,defense,craftSpeed}`. (The
  Advanced-drawer design already shows per-stat.)
- **IVs/talents**: `talent_hp / talent_shot / talent_defense` (0–100 display, raw byte) — add
  to the model (only placeholder in the Advanced drawer today).
- **Trust** = `friendship_point` (raw i64) — derive our trust rank/pct from it (invented now).
- **Condensation** = `rank` (0–4) ✓. **Nickname** is separate from `character_id` (species
  codename) — split them (ties into the species-selector work).
- **Identity/placement for save I/O** (mostly non-UI): `instance_id`, `character_id`,
  `storage_slot`, `storage_id`.
- **Moves**: `learned_skills` (learnset) is the REAL source for the bench "available moves";
  `active_skills` (equipped) ✓, `passive_skills` ✓, `work_suitability` (map) ✓.
- **Flags/status**: `is_lucky` ✓, `is_boss` (≈ our `alpha`), `is_tower` / `is_predator`
  (NPC/boss buckets → feed the species selector's "storable in global box" filter), `is_sick`,
  `owner_uid`, `group_id`. `stomach` (food, absolute) → foodPct, `sanity` → san, `hp`/`max_hp`
  ✓, `exp` (raw; we keep only pct).
Full DTO: instance_id · character_id · owner_uid · is_lucky · is_boss · is_tower · gender ·
nickname · group_id · stomach · sanity · hp · level · exp · rank · rank_hp/attack/defense/
craftspeed · talent_hp/shot/defense · max_hp · storage_slot · storage_id · learned/active/
passive_skills · work_suitability · is_sick · friendship_point.

## Trivial cleanup
- 2 svelte a11y warnings: add `role="group"` to the two move drop-zone `<div>`s in PalCard
  (`.moveslots` + `.bench`) — `a11y_no_static_element_interactions`.

## Deferred / loose ends
- If frameless window isn't wanted, revert `decorations:false` in `src-tauri/tauri.conf.json`.

## How to run / verify
- Run the app: `npm run tauri dev` (repo root). Rebuilds Rust + serves UI with HMR.
  **`tauri.conf` changes (e.g. `decorations`) only apply on a rebuild.**
- Frontend only: `npm run build` (→ `ui/build`, which Tauri embeds — build it BEFORE
  `cargo build`). Type-check: `npm run check`. Core tests: `cargo test -p palbox-core`.
- Preview screenshots: the in-app **browser pane only composites when it's the user's focused
  view** — when a screenshot times out, verify via `read_page` / injected JS + console instead.

## Key rules (don't relearn the hard way)
- **Design source of truth = `design/state-a-prototype/Palbox Studio.dc.html`.** LIFT its inline
  CSS values; do NOT re-derive layout from notes (that caused the wonky scaling this session).
  Its `support.js` runtime is intentionally missing — ignore it.
- **Value ranges = SPECS-1.0.md**, not our pre-1.0 fork (work suit 0–10, souls 0–20, level 80,
  condensation 0–4, IV 0–100, passives −3..5, single `Talent_Shot`; element codenames ≠ UI
  names). Cross-check game data against paldb.cc / wiki.gg; format against the PSP Rust source
  in `PalEdit/psp-reference/`.
- Never touch live saves — scratchpad copies only; no-edit round-trip diff must be zero.
- GUI stays modular and separate from the headless core (owner keeps tweaking the UI).
