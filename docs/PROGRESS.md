# Progress & Continuity

Living log of where the build is and what's next. Read this first when resuming.

## Session 2 (2026-07-24 pt.2) — UI build-out
**New reference (use it):** `../palworld-reference/` (sibling to PalboxStudio) —
`pal-data.md`, `save-and-file-format.md`, `paledit-code-map.md`, README. Filter categories,
move/passive legality, the 13 work suits, per-pal save fields.

**Done this session:**
- Real assets + data: 379 pal icons + 13 work icons in `ui/static`; card renders the real
  Incineram (CodeName Baphomet) + real work-suit icons. WORK_SUITS → **13** (added Oil Extraction).
- **Advanced drawer complete**: IV sliders (0–100), Statue of Power per-stat souls
  HP/ATK/DEF/WS (0–10, ±/clickable pips), condensation stars (0–4). Model gained `ivs` +
  `soulRanks` (per the PSP DTO map).
- **Global Box drawer**: Open-file button (`tauri-plugin-dialog`, defaults to
  `%LOCALAPPDATA%/Pal/Saved/SaveGames`), search + element + group filters, 3-col tile matrix,
  **Expand → full `BoxMatrix` overlay** of large tiles. `BoxTile` (sm/lg), `sampleBox`
  (real species). Left drawer opens on load.
- **RULE — humans/NPCs are EXCLUDED entirely** (can't live in a global box). Base filter on
  every species/box list is `Human !== true`; never show them as clutter or a filter bucket.

**Next:** wire the Rust engine (load `GlobalPalStorage.sav` → real box + computed pal stats;
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
   per-stat 0–10 rank bars + statue image), Souls & Condensation cards, amber backup warning.
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
   Palbox. **EXCLUDE human NPCs** — the game does not allow storing humans in the global box
   (filter out Human/NPC entities; the species data has a human flag — see PalEdit
   `update_data.py` `Human = not is_pal`). Selecting one changes the species. Needs: a distinct,
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
  0–10). Our model has a single `souls` → change to `{hp,attack,defense,craftSpeed}`. (The
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
- **App icon from the cropped logo**: owner swapped a cropped `assets/Palbox Studio Logo.png`,
  but `src-tauri/icons/*` and `ui/static/logo.png` are still from the ORIGINAL. Regenerate:
  `npm run tauri -- icon "assets/Palbox Studio Logo.png"`, then remove the ios/android sets and
  copy `src-tauri/icons/128x128@2x.png`→`ui/static/logo.png`, `128x128.png`→`ui/static/favicon.png`.
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
- **Value ranges = SPECS-1.0.md**, not our pre-1.0 fork (work suit 0–10, souls 0–10, level 80,
  condensation 0–4, IV 0–100, passives −3..5, single `Talent_Shot`; element codenames ≠ UI
  names). Cross-check game data against paldb.cc / wiki.gg; format against the PSP Rust source
  in `PalEdit/psp-reference/`.
- Never touch live saves — scratchpad copies only; no-edit round-trip diff must be zero.
- GUI stays modular and separate from the headless core (owner keeps tweaking the UI).
