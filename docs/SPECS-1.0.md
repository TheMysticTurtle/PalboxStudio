# Palworld 1.0 — Technical Specs (authoritative for the core model)

Researched against the **latest** sources, deliberately NOT trusting any pre-1.0 assumptions.
Values are verified against real 1.0 `GlobalPalStorage.sav` bytes and cross-checked against the
current public game databases (see Sources). The save-format layout these ranges live inside is
documented in [SAVE-FORMAT.md](SAVE-FORMAT.md). **This file is the source of truth for value
ranges in the core + UI.**

## Corrections vs. pre-1.0 assumptions (READ THESE)
| Field | Pre-1.0 assumption | **Palworld 1.0 (correct)** |
|---|---|---|
| Work Suitability cap | 5 | **1–10** (wild ~7–8; 9–10 via breed/condense). *Editor: all adjustable 0–10.* |
| Pal Souls per stat | 0–10 (pre-v0.4.11) | **0–20 ranks** (+3%/rank, **+60% max**) |
| Level cap | 80 | **80** (confirmed; exp table has 100 rows of headroom) |
| Attack IV | melee + shot | **single `Talent_Shot`** — no `Talent_Melee` |

## Save container (Global box)
- File `GlobalPalStorage.sav`; **Oodle** compression, magic `PlM`, save_type **`0x31`**.
- Top-level `SaveParameterArray` → **960 fixed slots**, each `{SaveParameter, InstanceId}`;
  **empty slot = `CharacterID == "None"`**. `SlotId.SlotIndex` is non-authoritative (dupes).
- Byte-preserve save_type on write; no-edit round-trip must be byte-identical.

## Per-pal fields & ranges (from the 1.0 save write path + game facts)
- **IVs / Talents** (breeding traits): `Talent_HP`, `Talent_Shot`, `Talent_Defense`. Stored as
  a **byte, raw 0–255**; PSP clamps 0–255. Game **displays 0–100**. → **UI range 0–100**
  (consider an "unrestricted 0–255" power-user note; the byte allows it).
- **Level**: byte; written only when >1; **cap 80** (exp table `exp.json` = levels 1–100).
- **Condensation** (`Rank`): **0–4 stars** (rank 4 = max). Needs **48** same-species (was 116).
  +5% HP/Atk/Def per rank (**+20% at ★4**); each rank-up also **+1 to a work suitability**,
  max rank raises **all**. PSP stores as byte (writes only when non-zero).
- **Pal Souls** (Statue of Power): per-stat ranks **0–20** — `Rank_HP`, `Rank_Attack`,
  `Rank_Defence`, `Rank_CraftSpeed` (Work Speed). +3%/rank, **+60% max per stat**. Byte;
  write only non-zero. *(This is the field the right-drawer "statue levels" edit.)*
- **Work Suitability**: **Lv 1–10**. Stored in `GotWorkSuitabilityAddRankList` as
  **bonus rank = desired_total − species_base**; **write only non-zero entries** (zero-bloat
  breaks in-game work — see the corruption traps in [SAVE-FORMAT.md](SAVE-FORMAT.md)). 12 jobs (see Elements/Jobs below).
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

## Work Suitability jobs (13, official UI names)
Kindling · Watering · Planting · Generating Electricity · Handiwork · Gathering · Lumbering ·
Mining · Crude Oil Extraction · Medicine Production · Cooling · Transporting · Farming.

## The 1.0 game-data layer
The app's static game data — species (elements, scaling, work suitability, learnsets, human
flag), active skills, passives (with `rank`), elements, the level→exp table, friendship ranks,
and localized display names — is served from the bundled reference database
(`data/palbox-reference.db`). Its contents, counts, and provenance are documented in
[DATA-AND-ASSETS.md](DATA-AND-ASSETS.md).

## Corruption traps to AVOID (hard-won — keep them solved; full write-up in [SAVE-FORMAT.md](SAVE-FORMAT.md))
- Never write the phantom `CraftSpeeds` field (real 1.0 pals don't have it).
- Never write zero-rank work-suitability entries; write only non-zero bonuses.
- Never auto-fill `MasteredWaza` from the learnset on load.
- Never re-add `Talent_Melee`.

## Sources

**Primary technical (for save-format truth — use first):**
- Real 1.0 `GlobalPalStorage.sav` bytes, verified on scratchpad copies, and this project's own
  [SAVE-FORMAT.md](SAVE-FORMAT.md) — the authoritative in-repo write-up of the format.

**Official / best community 1.0 wikis & databases (for game-data truth — cross-check here):**
- **Official v0.4.11 patch notes** — https://store.steampowered.com/news/app/1623730/view/518574472406499342
  — increased the Statue of Power enhancement limit from 10 to 20; that cap persists in 1.0.
- **paldb.cc** — https://paldb.cc/en/v1.0.0 — the most technical DB; mirrors game data,
  pal/skill/passive/element tables, and hosts the icon/texture CDN. **Primary data reference.**
- **palworld.wiki.gg** — https://palworld.wiki.gg/ — the main community wiki.
- **Fextralife** — https://palworld.wiki.fextralife.com/ — full Paldeck: stats, breeding
  combos, Partner Skills, Work Suitabilities for all 1.0 Pals.
- **palworld.gg** — https://palworld.gg/ — comprehensive DB + interactive map.
- **Game8** — https://game8.co/games/Palworld — walkthrough/guides, updated for 1.0.
- **Fandom** — https://palworld.fandom.com/wiki/Version_1.0 — the 1.0 version/patch page.
- Progression/mechanics guides used above: palmods.gg (work-suitability, progression-changes),
  nodecraft (work suitability 10), allthings.how (IVs/passives/souls), nexttier.pro (patch notes).

> When a game-data number is in question, prefer **paldb.cc / wiki.gg** (kept current) over any
> older guide, and prefer the **PSP source / real save bytes** for anything format-level.
