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
- **Element badges** → `ui/static/icons/elements/<element>.webp` (9). These are the retained
  PalEdit/PSP game-style `_icon.webp` assets, copied locally for offline use. Pal species, Partner
  Skills, move rows, and element filters all render them through the shared `ElementIcon` component;
  cached SQLite element colors drive borders/backgrounds, with the existing `--el-*` tokens as
  startup fallbacks before the reference bundle is available.
- **Statue of Power** → `ui/static/icons/statue-of-power.webp`. This is bundled for offline use
  and exposed as `STATUE_OF_POWER_ART`; the Advanced drawer never owns the filename. Source:
  [Palpedia's Statue of Power structure artwork](https://palpedia.com/structures/statue-of-power),
  retrieved 2026-07-25. The asset identity was cross-checked against the
  [Palworld Wiki](https://palworld.wiki.gg/wiki/Statue_of_Power) and
  [PalDB](https://paldb.cc/Statue_of_Power) structure records.

All UI artwork paths resolve through `ui/src/lib/data/icons.ts`; components do not own filenames.
The element basename comes from the cached SQLite `element.icon` field, Work Suitability components
pass their canonical basename to `workIcon`, and Alpha/Lucky use the shared variant registry.
`palIcon` remains the only place that handles portrait aliases and the missing-art fallback.

## SQLite reference dataset — `data/palbox-reference.db`

The desktop app now loads static reference data from normalized SQLite rather than the generated
JSON tables. Build it with:

```bash
python scripts/build_reference_db.py --check
```

The schema is `database/reference-schema.sql`; provenance, checksums, and known source conflicts
are stored in `data_source` and `data_quality_issue`. Current verified counts are:

- **406** storable species: Natural 223 / Unobtainable 160 / TowerBoss 23.
- **351** moves, including 27 audited boss-only placeholder definitions needed by learnsets.
- **420** passives with availability flags and structured effects.
- **2,372** items, **348** species-to-Partner-Skill records (**287** direct source cards plus
  61 same-name/same-tribe engine variants), and **29** Ranch species / **42** product links.
- All 13 Work Suitabilities, the level/EXP table, friendship ranks, and **59,192**
  localization rows across the languages present in the game-data dump.
- Self-describing `filter_field` and `filter_option` rows for UI filter generation.

The source order is deliberate: the Palworld Save Pal 1.0 game-data extract is authoritative for
codes and fields it exposes; retained web snapshots only fill Partner Skill and Ranch relationships.
See `data/reference-sources/README.md` and ADR 0003.

The SQLite reference DB is now the **sole** source: the old `ui/static/data/*.json` and its
`gen_species.py` generator have been removed. The UI loads the bundle once via `get_reference_data`,
which the engine materializes into memory at startup (`ReferenceCache`) so no command re-opens the
DB. A plain browser has no engine bridge, so browser-only preview no longer carries reference data —
verify in the app (`npm run tauri dev`).

## User metadata — `palbox-user.db`

`database/user-schema.sql` defines the separate writable store. It currently contains named passive
presets with ordered slots `0..3`, which enforces the four-passive maximum in SQLite as well as Rust.
The app validates codes against `palbox-reference.db` before saving or applying a preset. No mutable
Pal state is copied into this database; applying a preset changes only the in-memory Pal loaded from
`GlobalPalStorage.sav`, and the normal explicit save operation remains required.

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

## Derived combat stats

Global Palbox saves carry the inputs for Attack and Defense, not ready-made display values.
`ui/src/lib/data/palStats.ts` is the shared calculator for every card density and the main editor:
species scaling + level + IVs + Pal Soul ranks + condensation + the Alpha HP bonus + static
self-targeted Max HP/Attack/Defense passive effects. It deliberately excludes party, riding,
equipment, food, and server modifiers because those depend on runtime context outside the Global
Palbox file. IVs remain separate editor metadata in the Advanced drawer.

## Work Suitability — **13** (internal → current 1.0 UI). We had 12; add **Crude Oil Extraction**.
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
| OilExtraction / extracting        | Crude Oil Extraction    |
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
