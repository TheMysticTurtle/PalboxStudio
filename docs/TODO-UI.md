# UI TODO — remaining work to "UI ready"

Ordered roughly by priority. The reference tables (SQLite reference DB) + engine are the
data sources; the UI should resolve everything from **codes**, not carry denormalized strings.

## 1. Data-driven wiring (make the UI read real traits, not hardcoded samples)
- [ ] **Passives read from the pal's actual codes.** Today the card shows hardcoded
      `{name, rating, effects}` from `samplePal`. Instead a pal should hold passive **codes**;
      the card resolves `name / rating / description` from `passives.json`. (This is the
      "passives aren't reading the pal's file traits" issue.) Same pattern for equipped/bench
      **moves** → resolve `name / element / power / category` from `moves.json`.
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
- [ ] **Element icons** — psp `elements.json` already names them (`icon`, `badge_icon`,
      e.g. `Dark_icon`). Source the PNGs (paldb CDN / game textures) → `ui/static/icons/elements/`.
      Replace the CSS diamonds on pills/tiles/moves.
- [ ] **Passive icons** — check whether per-passive or per-group (Attack/Defense/Work/…) icons
      exist; if only grouped, show a group icon + rating chevrons.
- [ ] **Status / misc** — gender, condensation star, Pal Soul, SAN/food, alpha/lucky, skill
      fruit. Source what exists; document any gaps.
- [ ] Reuse the existing **work-suit icons** (already in `ui/static/icons/work/`).

## 3. Central card polish (owner's "make it sweet" goals)
- [ ] Portrait **glow tinted to primary element**; dual-type = top-half colour 1 / bottom-half
      colour 2. **Lucky → blue** emanating glow; **Alpha → red**.
- [ ] Presets: apply + create/save (passives / full builds).
- [ ] Move drag polish; passive add/remove picker (legality: rollable ∪ innate — see
      DATA-AND-ASSETS.md / palworld-reference).

## 4. Drawers / actions
- [ ] Advanced: bind IV / per-stat souls / condensation to real save fields (engine).
- [ ] Box: real groups/tags create/rename (SQLite); Sort menu; Add / Clone / Delete wired.
- [ ] Open-file: hand the picked path to the engine to actually load the box.

## 5. Engine + data (parallel track)
- [ ] Rust core: load/write `GlobalPalStorage.sav`; compute displayed stats from
      scaling + level + IV + souls + condensation; expose via Tauri commands.
- [x] Scrape **partnerSkill + farmDrops** — DONE: now in the reference DB (348 partner skills,
      Ranch products) via `scripts/scrape_*.py`; see ADR 0003.
- [ ] **SQLite** store for groups / tags / presets / settings (mutable user data).

## 6. Polish / correctness
- [ ] Verify window drag + open-file dialog work after a `tauri dev` rebuild.
- [ ] Empty/loading/error states; number formatting; reduced-motion pass.
- [ ] `samplePal` / `sampleBox` become fixtures only; real data comes from the engine.
