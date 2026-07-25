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
tests schema v1; the app creates the real DB in its platform application-data folder.

The current schema stores passive presets only:

- a unique user-defined name;
- zero to four unique passive codes;
- explicit slot order;
- timestamps.

Passive codes are deliberately not foreign keys inside this file because the
reference DB is a separate read-only database. `palbox-core` validates every code
against the reference DB in the same operation that writes a preset.

Per-Pal values remain exclusively in the loaded `GlobalPalStorage.sav`. Applying a
preset writes the four codes to that in-memory Pal; it does not create a second copy
of Pal state in SQLite.

## Schema changes

Add a numbered migration to the applicable schema and teach the Rust open path to
migrate older user databases before adding features. Rebuild generated DB files;
never edit a `.db` by hand.
