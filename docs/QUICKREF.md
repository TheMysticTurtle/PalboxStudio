# Quick Reference — Palbox Studio (Rust)

Practical pointers for building the global-box editor. The two authoritative sources are
**our own RE notes** and the **vendored PSP Rust source** — both already on disk.

## Where the knowledge lives (all local, no network needed)
- **Our RE notes:** [../../PalEdit/CLAUDE.md](../../PalEdit/CLAUDE.md) — the 1.0 format
  write-up, corruption bugs we found + fixed, data pipeline, testing recipe.
- **Our deeper analysis:** [../../PalEdit/docs/save-editing-analysis.md](../../PalEdit/docs/save-editing-analysis.md)
- **PSP Rust source (mechanics reference only — PSP is buggy, don't copy blind):**
  `PalEdit/psp-reference/psp-core/src/domain/`
  - `gps.rs` — **GlobalPalStorage** handling (our box! start here)
  - `pal.rs` — per-pal fields (IVs/talents, souls, passives, moves, level, gender, flags)
  - `containers.rs` — container/slot model (world uses a Slots array; the global box does NOT)
  - `guild.rs` / `guild_tail.rs` — the 1.0 guild-tail bytes we couldn't handle in Python
    (world-save only; out of scope for v1 but here if we ever need it)
  - `player.rs`, `raw.rs`, `uid_swap.rs`, `relic.rs`, `summaries.rs`, `world*.rs`
  - DTOs in `psp-core/src/dto/`, tests in `psp-core/tests/`
- **Game data (source of truth for species/moves/passives):**
  `PalEdit/psp-reference/data/json/*` — actively maintained, current within ~a day of
  patches. This is what PalEdit's `update_data.py` pulled from.

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
- **Work Suitability 1–10** (all adjustable to 10 in our editor); **Pal Souls 0–10 per stat**;
  **Condensation 0–4 stars**; **IV/talents 0–100** (raw byte 0–255). **See
  [SPECS-1.0.md](SPECS-1.0.md) — the authoritative source for all value ranges** (don't trust
  our pre-1.0 numbers).
- `IsPlayer` is written `False` on every pal — detect players by *value*, not key-presence.

## Corruption traps to AVOID (we caused these in PalEdit; do NOT repeat)
- **Never write `CraftSpeeds`** — real 1.0 pals don't have it; it broke work calc.
- **Never write zero-rank work-suitability entries** into `GotWorkSuitabilityAddRankList`
  (write only non-zero bonuses) — zero-bloat broke in-game work assignment.
- **Don't auto-fill `MasteredWaza`** from the learnset on load — real saves keep it empty;
  keep the UI's move view separate from `MasteredWaza`.

## Golden rules
- **Live saves are sacred** — operate only on scratchpad copies; back up before first write;
  atomic writes only.
- **Prove every change with the no-edit round-trip diff** (load → save → field-by-field
  diff must be **zero** added/removed/mutated) before trusting an edit path.
- One feature per branch; commits explain the *why* + bug/benefit/test notes; ADRs for
  architecture decisions.

## Flavor / nice touches (from PalEdit)
- Owner is "**The Mystic Testudine**" — a quiet turtle nod is welcome (PalEdit defaults a
  new pal to CubeTurtle / *Tetroise*). 🐢
