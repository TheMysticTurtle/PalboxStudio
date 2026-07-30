# Quick Reference — Palbox Studio (Rust)

Practical pointers for building the global-box editor. Every authoritative write-up lives in
this repo — start here, then follow into the deeper docs.

## Where the knowledge lives (all in-repo, no network needed)
- **Save format:** [SAVE-FORMAT.md](SAVE-FORMAT.md) — the 1.0 `GlobalPalStorage.sav` layout,
  the per-Pal fields and save-field map, the slot model, and the corruption traps.
- **Editable value meanings:** [SPECS-1.0.md](SPECS-1.0.md); runtime limits and formula
  operands are the typed rows in `data/palbox-reference.db`.
- **Data & assets:** [DATA-AND-ASSETS.md](DATA-AND-ASSETS.md) — the reference database, icons,
  filtering/legality logic, and game-data field mappings.
- **The engine itself:** `core/src/` — `globalbox.rs` (the flat 960-slot box), `pal.rs`
  (per-Pal save fields and semantic mutations), `projection.rs` (computed public views), and `save.rs`
  (Oodle/GVAS decode/encode with a lossless round-trip). The working implementation of
  everything above.
- **Reference database:** `data/palbox-reference.db` — the bundled, normalized game-data source
  (species, moves, passives, Partner Skills, ranch products, localization) the app reads at
  startup.

## Global Pal box format facts (verified on real 1.0 saves)
- File: `GlobalPalStorage.sav`. Owner's live path (**NEVER edit in place**):
  `%LOCALAPPDATA%\Pal\Saved\SaveGames\<steamid>\GlobalPalStorage.sav`
- Compression: **Oodle**, magic `PlM`, save_type `0x31` (pre-1.0 world = zlib `PlZ` `0x32`).
  Preserve the original save_type on write so round-trips stay byte-identical.
- Layout: top-level `SaveParameterArray` → **960 fixed slots**, each
  `{SaveParameter, InstanceId}`. **Empty slot = `CharacterID == "None"`.**
- `SlotId.SlotIndex` is **NOT** an authoritative display position for the global box —
  values are heavily duplicated in a real box. Don't invent slot-placement logic; an added
  clone may need "drag to an empty slot in-game."
- Level cap **80**; passives have ranks **−3..5** (rank 5 is the 1.0 addition); 1.0 pals have
  a single attack IV (`Talent_Shot`; `Talent_Melee` is gone — don't re-add it).
- **Work Suitability 1–10** (all adjustable to 10 in our editor); **Pal Souls 0–20 per stat**
  (**+3% per rank, +60% max**);
  **Condensation 0–4 stars** (stored as `Rank` 1–5); **IV/talents 0–100** (raw byte 0–255). **See
  [SPECS-1.0.md](SPECS-1.0.md) for meaning; the reference DB is the runtime authority**
  (don't trust our pre-1.0 numbers).
- `IsPlayer` is written `False` on every pal — detect players by *value*, not key-presence.
- The game-data dump's `is_pal` flag means “Pal-shaped engine actor,” not “ownable Pal.”
  Keep all 406 rows for decoding, but offer only the 287 `palbox_selectable` canonical species.
  `species_alias` maps 73 encounter/appearance codes back to their owned species.

## Corruption traps to AVOID (hard-won — keep them solved)
- **Never write `CraftSpeeds`** — real 1.0 pals don't have it; it broke work calc.
- **Never write zero-rank work-suitability entries** into `GotWorkSuitabilityAddRankList`
  (write only non-zero bonuses) — zero-bloat broke in-game work assignment.
- **Don't auto-fill `MasteredWaza`** from the learnset on load — real saves keep it empty;
  keep the UI's move view separate from `MasteredWaza`.
- **Translate condensation at the save boundary:** displayed stars 0–4 ↔ stored `Rank` 1–5.
- **Register writable property schemas in the core before encoding.** Optional fields absent from
  every source Pal otherwise have no `uesave` tag and cannot be introduced safely.
- **Refuse stale writes.** The core fingerprints the opened source and rechecks it before backup and
  again immediately before replacement.

## Golden rules
- **Live saves are sacred** — operate only on scratchpad copies; back up before first write;
  atomic writes only.
- **Prove every change with the no-edit round-trip diff** (load → save → field-by-field
  diff must be **zero** added/removed/mutated) before trusting an edit path.
- One feature per branch; commits explain the *why* + bug/benefit/test notes; ADRs for
  architecture decisions.

## Flavor / nice touches
- The owner is **The Mystic Turtle** (`themysticturtle`) — a quiet turtle nod is welcome; a
  newly added Pal defaults to a turtle (CubeTurtle / *Tetroise*). 🐢
