# Palbox Studio databases

Palbox Studio uses two SQLite databases with intentionally separate ownership.

## `palbox-reference.db`

Read-only, generated static Palworld 1.0 reference data:

- species, elements, Work Suitabilities, moves, effects, and learnsets;
- passives, effects, availability flags, and innate relationships;
- items, EXP levels, friendship ranks, and localizations;
- Partner Skills, rank values, same-species engine-variant inheritance, and Ranch products;
- filter-field metadata, source provenance, and quality findings.

Generate and validate it from the repository root:

```bash
python scripts/build_reference_db.py --check
```

The builder reads the current local `../PalEdit/psp-reference/data/json` extract plus
the immutable snapshots under `data/reference-sources`. It builds into a temporary
file, runs SQLite integrity/foreign-key checks, and atomically replaces the output.

The Tauri bundle ships this database as `data/palbox-reference.db` and Rust opens it
with SQLite's read-only flag.

## `palbox-user.db`

Writable local app metadata. The checked-in `data/palbox-user.template.db` proves and
tests schema v2; the app creates the real DB in its platform application-data folder
and migrates existing v1 databases automatically.

The current schema stores:

- passive presets with a unique name, zero to four unique passive codes, explicit
  slot order, and timestamps;
- unique user-named groups;
- many-to-many Pal membership keyed by stable `InstanceId`.

Passive codes are deliberately not foreign keys inside this file because the
reference DB is a separate read-only database. `palbox-core` validates every code
against the reference DB in the same operation that writes a preset.

Per-Pal game values remain exclusively in the loaded `GlobalPalStorage.sav`. Applying
a preset writes its codes to that in-memory Pal; group membership is app-only metadata
and never creates a second copy of mutable Pal state.

## Schema changes

Add a numbered migration to the applicable schema and teach the Rust open path to
migrate older user databases before adding features. Schema v2 follows this contract
through `migrations/user-v2-groups.sql`. Rebuild generated DB files; never edit a
`.db` by hand.
