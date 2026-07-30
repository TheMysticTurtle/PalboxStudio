# Data & Assets — the game data, icons, and logic map

Quick-reference map so we don't re-derive anything. This catalogs where the game data, icons,
and filtering/legality logic live and how they map.

## Icons (copied into this repo)
- **Palbox Studio identity** → authored source `assets/Palbox Studio Logo.png`; deployed web
  surfaces use `ui/static/logo.png` through `APP_LOGO_ART`, the favicon uses the generated
  128×128 asset, and `src-tauri/icons/` contains the generated desktop application/icon bundle.
  The earlier image with a baked white/grey transparency grid is not a runtime asset.
- **Pal portraits** → `ui/static/pals/T_<CodeName>_icon_normal.png` (379). Served at `/pals/…`.
  Lookup rule: strip a `RAID_` prefix and a trailing `_2` from the
  CharacterID; fall back to `#ERROR.png` when missing (~27 scrapped/quest entities have none —
  placeholder is fine). **In a URL the fallback must be `%23ERROR.png`** (the `#` is a fragment).
  The retained sources contain 296 images at 240×240 and 83 at 128×128, with varying transparent
  padding. `PalArtwork.svelte` is the shared rendered surface that gives them one fixed-size,
  clipped, rounded crop and fallback rule across every card and picker. Keep the original files;
  do not create component-owned copies or per-species CSS corrections.
- **Work-suitability icons** → `ui/static/icons/work/<name>.png` + `no_<name>.png` (13 active +
  13 greyed). Names use the game's internal set — see the mapping below.
- **Element badges** → `ui/static/icons/elements/<element>.webp` (9). These are the retained
  game-style `_icon.webp` assets, copied locally for offline use. Pal species, Partner
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
`APP_LOGO_ART` likewise keeps component code independent of the deployed logo filename.

## SQLite reference dataset — `data/palbox-reference.db`

The desktop app loads static reference data from this normalized SQLite database. It is a
**prebuilt, committed artifact** — it only needs regenerating when Palworld itself changes (see
[`database/README.md`](../database/README.md) for provenance and the regeneration story).

The schema is `database/reference-schema.sql`; provenance, checksums, and known source conflicts
are stored in `data_source` and `data_quality_issue`. Current verified counts are:

- **406** retained Pal-shaped engine rows: Natural 288 / Unobtainable 95 / TowerBoss 23.
  Only **287** are canonical, owned-Pal species safe to offer in the species selector.
  The other rows remain available for decoding unusual saves, and **73** same-name/same-tribe
  encounter or appearance codes map through `species_alias` to a canonical species.
- **351** moves, including 27 audited boss-only placeholder definitions needed by learnsets.
- **420** passives with availability flags and structured effects.
- **2,372** items, **348** species-to-Partner-Skill records (**287** direct source cards plus
  61 same-name/same-tribe engine variants), and **29** Ranch species / **42** product links.
- All 13 Work Suitabilities, the level/EXP table, friendship ranks, and **59,192**
  localization rows across the languages present in the game-data dump.
- Typed editing limits and numeric calculation operands used by the headless engine.
- Self-describing `filter_field` and `filter_option` rows for UI filter generation.

For direct inspection in a SQLite viewer, use the `v_species_*`,
`v_partner_skill_progression`, `v_move_*`, `v_passive_*`, and
`v_reference_sources` views. They place internal codes, readable names, values,
and provenance together without sacrificing the normalized runtime schema.

The source order is deliberate: the Palworld Save Pal 1.0 game-data extract is authoritative for
codes and fields it exposes; retained web snapshots only fill Partner Skill and Ranch relationships.
See `data/reference-sources/README.md` and ADR 0003.

### Palbox-selectable species audit

`is_pal` is not sufficient: the engine marks raid body parts, summon actors, predator encounters,
tower models, quest helpers, retired models, and uncatchable bosses as Pal-shaped actors too.
The `species.palbox_selectable` flag is derived from ownership-oriented signals:

- a valid Paldeck index, enabled data, and a normal owned-species actor;
- no raid/summon/predator/tower/quest/oil-rig actor code;
- no boss-only, raid-only, or tower-only flag;
- explicit exclusion of uncatchable Astralym (`WorldTreeDragon`);
- one canonical row per Paldeck/name/tribe identity.

The installed-database check enforces 287 selectable rows, zero duplicate display names, and the
presence of canonical Xenovader (`DarkAlien`), Xenogard (`WhiteAlienDragon`), and Xenolord
(`DarkMechaDragon`). It also prevents Boltmane (`ElecLion`), Astralym, and Moon Lord raid actors
from entering the chooser. Do not weaken this back to `is_pal` or “not human.”

The SQLite reference DB is now the **sole** source: the old `ui/static/data/*.json` and its
`gen_species.py` generator have been removed. The UI loads the bundle once via `get_reference_data`,
which the engine materializes into memory at startup (`ReferenceCache`) so no command re-opens the
DB. A plain browser has no engine bridge, so browser-only preview no longer carries reference data —
verify in the app (`npm run tauri dev`).

## User metadata — `palbox-user.db`

`database/user-schema.sql` defines the separate writable store. Schema v4 contains named passive
presets with ordered entries, plus user-named groups and many-to-many membership keyed by a
Pal's stable `InstanceId`, and engine-owned app settings for the remembered box and auto-open
toggle. Existing v1/v2/v3 databases migrate in place through the numbered scripts in
`database/migrations/`. The app validates passive codes against
`palbox-reference.db` before saving or applying a preset. The current passive count limit also
comes from the reference DB; it is not duplicated in the durable user schema. No mutable Pal game state is copied into
this database; applying a preset changes only the in-memory Pal loaded from
`GlobalPalStorage.sav`, and the normal explicit save operation remains required. Group membership
never enters the Palworld save.

## Per-pal source-data schema
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
`core/src/projection.rs` is the single calculator for every frontend and card density:
species scaling + level + IVs + Pal Soul ranks + condensation + the Alpha HP bonus + static
self-targeted Max HP/Attack/Defense passive effects. Formula operands come from the typed
`calculation_rules` row in SQLite. It deliberately excludes party, riding, equipment, food,
and server modifiers because those depend on runtime context outside the Global Palbox file.

## Work Suitability — **13** (internal → current 1.0 UI)
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

## Source game-data + relationships
The upstream game-data extract provides: `pals.json` · `active_skills.json` (324) ·
`passive_skills.json` (420, rank −3..5) · `elements.json` (9) · `exp.json` (level→exp) ·
`friendship.json` (trust ranks) · `presets.json` · `bosses.json` · relics · items · technologies ·
buildings · missions · map/fast-travel · `l10n/` · `ui/`. The per-Pal save-field map is in
[SAVE-FORMAT.md](SAVE-FORMAT.md).

## Logic we already solved — don't re-derive
- **Data field mappings** (from the game-data extract): Type←element_types, Moveset←skill_set,
  Scaling←scaling, Suitabilities←work_suitability, Human←`not is_pal`, passive Rating←rank,
  display names←l10n.
- **Passive legality**: legal = **rollable ∪ innate**. `Rollable` bool (85/420 roll on wild pals),
  per-pal `InnatePassives`, and `Exclusive` species on `Unique_` attacks (239 covered).
- **Filtering / sorting** (species browser + ability pickers): element + work-suit + name search;
  **category buckets** — Obtainable (`DeckIndex>=0`) / Boss·Tower (`TowerBoss` or `BOSS_/GYM_/RAID_`)
  / NPC (`Human=true`). Attack-picker tiers: learnset-only / +fruit-teachable / all.
- **"Storable in the Global Palbox" filter**: use the generated `palbox_selectable` field.
  Human exclusion alone is deliberately insufficient; see the audit above.
- **Save format details**: [SAVE-FORMAT.md](SAVE-FORMAT.md).
- **Value ranges + corruption traps**: [SPECS-1.0.md](SPECS-1.0.md), [QUICKREF.md](QUICKREF.md).

## Fallback / CDN
paldb CDN mirror if an icon is ever missing:
`cdn.paldb.cc/image/Pal/Texture/PalIcon/Normal/T_<Code>_icon_normal.webp` (we bundle locally for
offline; don't fetch at runtime).
