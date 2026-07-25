# ADR 0002 — Data planes, engine contract, and app-data schema

- **Status:** Accepted (plan; engine not built yet)
- **Date:** 2026-07-24
- **Purpose:** lock how data flows so the UI wires onto the engine cleanly with no rework.

## The three data planes

Everything in the app is one of three kinds of data. Keeping them separate is the whole design.

| Plane | What | Lifetime | Where it lives | Owner |
|---|---|---|---|---|
| **Reference** (static) | species, moves, passives, elements, schema | changes on game patch | `ui/static/data/*.json` (generated) | `gen_species.py` |
| **Save model** (editable) | the pals in the Global Palbox + their per-instance values | per edit session | `GlobalPalStorage.sav` | Rust engine |
| **App data** (mutable meta) | groups, tags, passive/build presets, settings | forever, user-owned | **SQLite** (`palbox.db`) | Rust engine |

Rule: a pal stores **codes + raw editable values**; all human-readable info is **resolved** from
the reference plane at display time (already done for passives/moves — see refdata.svelte.ts).
App-data (groups/tags/presets) is keyed by the pal's **InstanceId** and is **never** written to
the `.sav`.

## The flow (matches the intended UX)

```
pick a pal  →  engine returns its instance (raw editable fields, keyed by codes)
            →  UI resolves static data from the reference tables (name/desc/element/…)
            →  UI shows editable fields; user edits (mutates the in-memory instance)
            →  Save  →  engine: backup original, then atomic-write the edited box
filters     →  run over the reference tables (schema-driven), display matching items
```

## Engine contract (Tauri commands)

The UI only ever talks to the engine through these. Types are illustrative (serde structs).

**Box lifecycle**
- `default_save_dir() -> String` — `%LOCALAPPDATA%/Pal/Saved/SaveGames` (dialog default).
- `open_box(path: String) -> BoxSummary` — decompress + parse; returns `{ path, count, pals: BoxTile[] }` (lightweight tiles: instanceId, species, name, level, elements, alpha, lucky).
- `get_pal(instance_id: String) -> PalDto` — full editable fields for one pal (below).
- `save_box() -> SaveResult` — **backup original → atomic write** the edited box; returns `{ backupPath }`. **Mandatory backup before any write.**
- `close_box()` — drop the in-memory model.

**Mutations** (granular; each validates + clamps to the 1.0 limits, returns the updated `PalDto`)
- `set_level`, `set_nickname`, `set_gender`, `set_favorite`
- `set_iv(stat, value)`, `set_soul_rank(stat, rank)`, `set_condensation(rank)`
- `set_passives(codes: string[])`, `set_moves(equipped: string[])`
- `set_work_suit(job, level)`
- `set_species(code)` — changes CharacterID (re-derives type/learnset)
- `add_pal(species)`, `clone_pal(instance_id)`, `delete_pal(instance_id)`

**App data** (SQLite; see schema below)
- `list_groups()`, `create_group(name)`, `rename_group(id, name)`, `delete_group(id)`
- `set_pal_groups(instance_id, group_ids[])`
- `list_presets(kind)`, `save_preset(kind, name, payload)`, `delete_preset(id)`
- `get_setting(key)`, `set_setting(key, value)`

### `PalDto` — raw editable fields + computed display fields

Mirrors PSP's `PalDto` (see PROGRESS.md field map). Split into what the user **edits** vs what
the engine **computes** for display (so the UI never re-implements game formulas):

- **Editable (raw):** `instanceId`, `species` (character_id), `nickname`, `gender`, `level`, `exp`,
  `condensation` (rank), `ivs{hp,shot,defense}` (talent_*), `soulRanks{hp,attack,defense,craftSpeed}`
  (rank_*), `workSuit{13}` (GotWorkSuitabilityAddRankList), `passives[]` (codes), `equippedMoves[]`,
  `learnedMoves[]`, `favorite`, flags (`alpha`/is_boss, `lucky`/is_lucky).
- **Computed (read-only, engine-derived from scaling + level + IV + soul + condensation):**
  `stats{hp, hpMax, attack, defense, workSpeed}`, `trust` (from friendship_point), `expToNext`/`expPct`.

## Save / backup flow (non-negotiable safety)

1. On `save_box`: copy the original `.sav` to `<name>.YYYYMMDD-HHMMSS.bak` (a `PalboxStudio-backups/`
   folder) **before** touching it. A failed backup **aborts** the save.
2. Write to a temp file, then atomic rename over the target (no partial writes).
3. Preserve compression/`save_type` so an unmodified pal round-trips **byte-identical**.
4. Never operate on the live save in place beyond the read; all edits go through the copy-in-memory
   → backup → write path. (Carried from the PalEdit rules.)

## App-data SQLite schema (`palbox.db`)

Mutable, relational, user-owned — the correct place for a real DB (openable in DB Browser). Keyed
by the save's `InstanceId` GUIDs, independent of the save file.

```sql
CREATE TABLE groups (
  id         INTEGER PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE,
  color      TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
-- a pal (by InstanceId) belongs to zero or more groups (tags bind pals to groups)
CREATE TABLE pal_group (
  instance_id TEXT NOT NULL,
  group_id    INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  PRIMARY KEY (instance_id, group_id)
);
CREATE TABLE presets (
  id         INTEGER PRIMARY KEY,
  kind       TEXT NOT NULL,          -- 'passives' | 'build'
  name       TEXT NOT NULL,
  payload    TEXT NOT NULL,          -- JSON: e.g. { "passives": ["Legend", ...] }
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (kind, name)
);
CREATE TABLE settings ( key TEXT PRIMARY KEY, value TEXT NOT NULL );
```

## How the UI wires in (per plane) — the "similar format" pattern

Every editable subject is a small component with the **same shape**: resolve its *options* from the
reference tables, read the pal's *current value*, and emit edits.
- **Passives / Moves** — picker lists options from `passives.json` / `moves.json` (with legality:
  rollable ∪ innate; learnset/fruit/all tiers — see `palworld-reference/`); chip resolves the code.
- **Work suit / IV / Souls / Condensation** — steppers/sliders bound to the raw value, clamped to
  `LIMITS`.
- **Filters** — one generic engine over the reference rows, driven by `schema.json`
  (enum→chips, number→range, multi→any-of, bool→toggle). The box filter and the species selector
  are the **same** component pointed at different columns.
- **Groups/tags** — read/write via the app-data commands; a group doubles as a filter chip.

Until the engine exists, the UI runs on `samplePal`/`sampleBox` fixtures shaped exactly like the
DTOs above, so swapping in real engine data is a drop-in.

## Consequences
- The UI can be fully built and wired against these DTO shapes now; the engine slots in behind
  them with zero UI rework.
- Three stores/plumbing: reference (fetch JSON, done), save model (engine commands), app data
  (engine + SQLite). Each independently testable.
- Reference stays diff-reviewable JSON; mutable data gets real relational integrity in SQLite.
