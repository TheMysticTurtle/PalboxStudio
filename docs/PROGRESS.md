# Progress & Continuity

Living log of where the build is and what's next. Read this first when resuming.

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
