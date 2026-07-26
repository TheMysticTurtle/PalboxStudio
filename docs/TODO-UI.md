# UI TODO — remaining work to "UI ready"

Ordered roughly by priority. The reference tables (SQLite reference DB) + engine are the
data sources; the UI should resolve everything from **codes**, not carry denormalized strings.

## 1. Data-driven wiring (make the UI read real traits, not hardcoded samples)
- [x] **Passives and moves resolve from the Pal's actual codes.** The card resolves localized
      names, ratings, descriptions, elements, power, and category through the startup reference
      cache; no sample strings participate in save editing.
- [x] **Load the reference tables once** into a typed store (`species/moves/passives/elements`
      + `schema`), with a small resolver (code → row) the components use.
- [ ] **Schema-driven filters.** Generate the box/species filter controls from `schema.json`
      (enum→chips, number→range, multi→any-of, bool→toggle); replace the hardcoded element/group
      filters in `GlobalBoxDrawer`. One generic `filter(rows, activeFilters)` over columns.
- [x] **Species selector.** Click the species name on the card → searchable/filterable list from
      the reference DB's audited `palbox_selectable` roster with icons → change species.
- [x] **Box tile → card.** Selecting a box tile loads that pal into the center card.
- [x] **Learnset/bench from data.** Bench "available moves" = the pal's real learnset
      (`species.moves`) resolved via `moves.json`, not the sample list.

## 2. In-game icons for "different stuff" (replace coloured blocks)
Gather real game-texture icons as bundled assets where they exist; keep coloured fallbacks.
- [x] **Element icons** — bundled game-style assets render through the shared `ElementIcon`
      component on cards, filters, skills, and the species selector.
- [ ] **Passive icons** — check whether per-passive or per-group (Attack/Defense/Work/…) icons
      exist; if only grouped, show a group icon + rating chevrons.
- [ ] **Status / misc** — gender, condensation star, Pal Soul, SAN/food, alpha/lucky, skill
      fruit. Source what exists; document any gaps.
- [x] Reuse the existing **work-suit icons** through `WorkIcon`, including selector mini-cards.

## 3. Central card polish (owner's "make it sweet" goals)
- [ ] Portrait **glow tinted to primary element**; dual-type = top-half colour 1 / bottom-half
      colour 2. **Lucky → blue** emanating glow; **Alpha → red**.
- [x] Passive presets: apply + create/edit/delete named sets of up to four passives through the
      shared filtered passive picker. Full-build presets remain a possible later extension.
- [x] Move drag/equip/reorder uses pointer events plus the tested `moveSlots.ts` engine; passive
      add/remove uses the filtered reference-data picker.

## 4. Drawers / actions
- [ ] Advanced: bind IV / per-stat souls / condensation to real save fields (engine).
- [x] Box groups/tags: create/rename/delete in SQLite, assign through a compact shared dropdown
      from the main card or either Global Palbox view, display on both card densities, and filter
      with match-all semantics.
- [ ] Open-file: hand the picked path to the engine to actually load the box.

## 5. Engine + data (parallel track)
- [ ] Rust core: load/write `GlobalPalStorage.sav`; compute displayed stats from
      scaling + level + IV + souls + condensation; expose via Tauri commands.
- [x] Scrape **partnerSkill + farmDrops** — DONE: now in the reference DB (348 partner skills,
      Ranch products) via `scripts/scrape_*.py`; see ADR 0003.
- [x] **SQLite** store for groups / tags / passive presets (schema v2 with automatic v1 migration).
      Settings can join the same user-data plane later.

## 6. Polish / correctness
- [x] Establish a shared readability floor and resize species filters, mini tiles, compact cards,
      expanded cards, main-editor metadata, skill/passive pickers, and work/stat controls as one
      coordinated system. Gameplay information is at least 11px; primary names and controls are
      larger, and responsive grids trade a little density for legibility.
- [ ] Verify window drag + open-file dialog work after a `tauri dev` rebuild.
- [ ] Empty/loading/error states; number formatting; reduced-motion pass.
- [ ] `samplePal` / `sampleBox` become fixtures only; real data comes from the engine.
