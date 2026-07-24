# Palworld 1.0 — Technical Specs (authoritative for the core model)

Researched 2026-07-24 against the **latest** sources, deliberately NOT trusting our pre-1.0
PalEdit inheritance. Sources: the vendored PSP 1.0 Rust editor
(`PalEdit/psp-reference/psp-core/src/domain/pal.rs` — how it clamps/writes each field) and its
maintained 1.0 data JSONs (`PalEdit/psp-reference/data/json/*`), cross-checked with 1.0 game
guides (see Sources). **This file is the source of truth for value ranges in the core + UI.**

## Corrections vs. our old pre-1.0 fork (READ THESE)
| Field | Old/forked assumption | **Palworld 1.0 (correct)** |
|---|---|---|
| Work Suitability cap | 5 (spinboxes `to=5`) | **1–10** (wild ~7–8; 9–10 via breed/condense). *Editor: all adjustable 0–10.* |
| Pal Souls per stat | "0–20" (stale note) | **0–10 ranks** (+3%/rank, **+30% max**) |
| Level cap | 80 | **80** (confirmed; exp table has 100 rows of headroom) |
| Attack IV | (fork had melee+shot) | **single `Talent_Shot`** — no `Talent_Melee` |

## Save container (Global box)
- File `GlobalPalStorage.sav`; **Oodle** compression, magic `PlM`, save_type **`0x31`**.
- Top-level `SaveParameterArray` → **960 fixed slots**, each `{SaveParameter, InstanceId}`;
  **empty slot = `CharacterID == "None"`**. `SlotId.SlotIndex` is non-authoritative (dupes).
- Byte-preserve save_type on write; no-edit round-trip must be byte-identical.

## Per-pal fields & ranges (from `pal.rs` write path + game facts)
- **IVs / Talents** (breeding traits): `Talent_HP`, `Talent_Shot`, `Talent_Defense`. Stored as
  a **byte, raw 0–255**; PSP clamps 0–255. Game **displays 0–100**. → **UI range 0–100**
  (consider an "unrestricted 0–255" power-user note; the byte allows it).
- **Level**: byte; written only when >1; **cap 80** (exp table `exp.json` = levels 1–100).
- **Condensation** (`Rank`): **0–4 stars** (rank 4 = max). Needs **48** same-species (was 116).
  +5% HP/Atk/Def per rank (**+20% at ★4**); each rank-up also **+1 to a work suitability**,
  max rank raises **all**. PSP stores as byte (writes only when non-zero).
- **Pal Souls** (Statue of Power): per-stat ranks **0–10** — `Rank_HP`, `Rank_Attack`,
  `Rank_Defence`, `Rank_CraftSpeed` (Work Speed). +3%/rank, **+30% max per stat**. Byte;
  write only non-zero. *(This is the field the right-drawer "statue levels" edit.)*
- **Work Suitability**: **Lv 1–10**. Stored in `GotWorkSuitabilityAddRankList` as
  **bonus rank = desired_total − species_base**; **write only non-zero entries** (zero-bloat
  breaks in-game work — a bug we caused pre-1.0). 12 jobs (see Elements/Jobs below).
- **Moves = Active Skills**: up to **3 equipped** (`EquipWaza`); **learned** in `MasteredWaza`
  (**leave empty unless explicitly mastered** — do NOT auto-fill from learnset). 324 skills
  defined; power **0–1200**; each has an element.
- **Passive Skills**: up to **4** per pal (`PassiveSkillList`). 420 defined; **rank −3..5**
  (no 0; **rank 5 is the 1.0 addition**). Legality = rollable ∪ innate (per species).
- **Identity/other**: `NickName` (+ `FilteredNickName`), gender, `Exp`, **Trust** /
  `FriendshipPoint` (int), Lucky/Boss/Alpha flags, `IsPlayer` written **`False`** on every pal
  (detect players by value, not key presence).

## Elements (9) — internal codename → official UI name
`Normal`→**Neutral** · `Fire`→Fire · `Water`→Water · `Electricity`→**Electric** ·
`Leaf`→**Grass** · `Ice`→Ice · `Earth`→**Ground** · `Dark`→Dark · `Dragon`→Dragon.
**The save/data use the left codenames; show the right names.** (`elements.json` has all 9.)

## Work Suitability jobs (12, official UI names)
Kindling · Watering · Planting · Generating Electricity · Handiwork · Gathering · Lumbering ·
Mining · Medicine Production · Cooling · Transporting · Farming.

## 1.0 data JSONs we build the app's game-data layer from (`psp-reference/data/json/`)
`pals.json` (species: elements, scaling, work suitability, skill_set, human flag),
`active_skills.json` (324), `passive_skills.json` (420, with `rank`), `elements.json` (9),
`exp.json` (level→exp, 1–100), `presets.json`, `friendship.json`, plus `l10n/<lang>/…` for
localized display names. These are actively maintained current with patches — our data layer
mirrors them (as PalEdit's `update_data.py` already did).

## Corruption traps to AVOID (we caused these pre-1.0 — do NOT repeat)
- Never write the phantom `CraftSpeeds` field (real 1.0 pals don't have it).
- Never write zero-rank work-suitability entries; write only non-zero bonuses.
- Never auto-fill `MasteredWaza` from the learnset on load.
- Never re-add `Talent_Melee`.

## Sources
- PSP 1.0 source: `PalEdit/psp-reference/psp-core/src/domain/{pal,gps,containers}.rs`; data:
  `PalEdit/psp-reference/data/json/*`.
- https://nodecraft.com/support/games/palworld/general/palworld-work-suitability-level-10-explained
- https://www.palmods.gg/guides/whats-new/work-suitability
- https://www.palmods.gg/guides/whats-new/progression-changes
- https://allthings.how/palworld-1-0-how-to-max-your-pals-with-ivs-passives-and-souls/
- https://nexttier.pro/guide/palworld-patch-notes
