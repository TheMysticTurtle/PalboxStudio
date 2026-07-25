# Progress & Continuity

Living log of where the build is and what's next. Read this first when resuming.

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
