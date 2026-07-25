# Data & Assets — what the first build (PalEdit) already gives us

Quick-reference map so we don't re-derive anything. The first build cleanly gathered the game
data, icons, and the filtering/legality logic — this catalogs where it all is and how it maps.

## Icons (copied into this repo)
- **Pal portraits** → `ui/static/pals/T_<CodeName>_icon_normal.png` (379). Served at `/pals/…`.
  Lookup rule (PalEdit `GetImage`): strip a `RAID_` prefix and a trailing `_2` from the
  CharacterID; fall back to `#ERROR.png` when missing (~27 scrapped/quest entities have none —
  placeholder is fine). **In a URL the fallback must be `%23ERROR.png`** (the `#` is a fragment).
- **Work-suitability icons** → `ui/static/icons/work/<name>.png` + `no_<name>.png` (13 active +
  13 greyed). Names use PalEdit's internal set — see the mapping below.
- Elements have **no** image here; we draw them as CSS diamonds (`--el-*` tokens).

## Per-pal data schema (`PalEdit/palworld_pal_edit/resources/data/pals/<Code>.json`)
```
CodeName, Type[2] (element codenames, "None"-padded), Moveset {EPalWazaID::X: unlockLevel},
RaidMoveset, Scaling {HP, PHY, MAG, DEF}, Suitabilities {13 internal keys: level},
Human (bool), InnatePassives[], DeckIndex (>=0 = obtainable), TowerBoss (bool)
```
- **Display names**: `data/en-GB/pals.json` maps CodeName → localized (`Baphomet` = "Incineram",
  `Alpaca` = "Melpaca"). Also `en-GB/attacks.json`, `passives.json`, `ui.json`. Langs: en-GB,
  it-IT, zh-CN.
- **Attacks**: `data/attacks/<Waza>.json` → {element, power, category}. **Passives**:
  `data/passives.json`. Aggregates: `data/{pals,attacks,elements,passives}.json`.

## Work Suitability — **13** (internal → official UI). We had 12; ADD **Oil Extraction**.
| internal key / icon file          | official UI name        |
|-----------------------------------|-------------------------|
| EmitFlame / kindling              | Kindling                |
| Watering / watering               | Watering                |
| Seeding / planting                | Planting                |
| GenerateElectricity / generating  | Generating Electricity  |
| Handcraft / handiwork             | Handiwork               |
| Collection / gathering            | Gathering               |
| Deforest / deforesting            | Lumbering               |
| Mining / mining                   | Mining                  |
| OilExtraction / extracting        | Oil Extraction          |
| ProductMedicine / production      | Medicine Production     |
| Cool / cooling                    | Cooling                 |
| Transport / transporting          | Transporting            |
| MonsterFarm / farming             | Farming                 |

## Element codenames (data) → UI name
Normal→Neutral · Fire · Water · Electricity→Electric · Leaf→Grass · Ice · Earth→Ground · Dark ·
Dragon. (Type[] pads a 2nd slot with `"None"` for single-element pals.)

## Source data + relationships (`PalEdit/psp-reference/data/json/`)
`pals.json` · `active_skills.json` (324) · `passive_skills.json` (420, rank −3..5) ·
`elements.json` (9) · `exp.json` (level→exp) · `friendship.json` (trust ranks) · `presets.json` ·
`bosses.json` · relics · items · technologies · buildings · missions · map/fast-travel · `l10n/` ·
`ui/`. The PSP Rust engine reading these: `psp-core/src/domain/{pal,gps,containers,guild}.rs`;
its **PalDto field list is mapped in [PROGRESS.md](PROGRESS.md)**.

## Logic we already solved — don't re-derive (see `PalEdit/CLAUDE.md` + `PalEdit.py`)
- **Data pipeline + field mappings**: `update_data.py` and the mapping table in `PalEdit/CLAUDE.md`
  (Type←element_types, Moveset←skill_set, Scaling←scaling, Suitabilities←work_suitability,
  Human←`not is_pal`, passive Rating←rank, display names←l10n).
- **Passive legality**: legal = **rollable ∪ innate**. `Rollable` bool (85/420 roll on wild pals),
  per-pal `InnatePassives`, and `Exclusive` species on `Unique_` attacks (239 covered).
- **Filtering / sorting** (species browser + ability pickers, all in `PalEdit.py`): element +
  work-suit + name search; **category buckets** — Obtainable (`DeckIndex>=0`) / Boss·Tower
  (`TowerBoss` or `BOSS_/GYM_/RAID_`) / NPC (`Human=true`). Attack-picker tiers: learnset-only /
  +fruit-teachable / all.
- **"Storable in the Global Palbox" filter** (for the species selector): **exclude Human NPCs**;
  drive with DeckIndex / TowerBoss / Human. (Directly answers the species-selector request.)
- **Deep save analysis**: `PalEdit/docs/save-editing-analysis.md`.
- **Value ranges + corruption traps**: `docs/SPECS-1.0.md`, `docs/QUICKREF.md`.

## Fallback / CDN
paldb CDN mirror if an icon is ever missing:
`cdn.paldb.cc/image/Pal/Texture/PalIcon/Normal/T_<Code>_icon_normal.webp` (we bundle locally for
offline; don't fetch at runtime).
