# Palbox Studio databases

Palbox Studio uses two SQLite databases with intentionally separate ownership.

## `palbox-reference.db`

Read-only, generated static Palworld 1.0 reference data:

- species, elements, Work Suitabilities, moves, effects, and learnsets;
- passives, effects, availability flags, and innate relationships;
- items, EXP levels, friendship ranks, and localizations;
- Partner Skills, rank values, same-species engine-variant inheritance, and Ranch products;
- filter-field metadata, source provenance, and quality findings.

This database is a **prebuilt, committed artifact** (`data/palbox-reference.db`): because it
holds static Palworld 1.0 facts, it only needs regenerating when the game itself changes. It
ships in the Tauri bundle, and Rust opens it with SQLite's read-only flag.

The generator that produced it is kept as a local maintainer tool and is not part of the public
tree. A self-contained extractor — building the data directly from the game's own packaged
assets — is planned future work.

**Provenance & attribution.** The data derives from Palworld 1.0's own game data, cross-checked
against the public community databases ([paldb.cc](https://paldb.cc),
[wiki.gg](https://palworld.wiki.gg)) and the open-source
[palworld-save-pal](https://github.com/oMaN-Rod/palworld-save-pal) project. Save serialization
uses [`uesave-rs`](https://github.com/oMaN-Rod/uesave-rs). Palworld is a trademark of
Pocketpair, Inc.; Palbox Studio is an unofficial, fan-made tool.

## `palbox-user.db`

Writable local app metadata. The checked-in `data/palbox-user.template.db` proves and
tests schema v3; the app creates the real DB in its platform application-data folder
and migrates existing v1/v2 databases automatically.

The current schema stores:

- passive presets with a unique name, zero to four unique passive codes, explicit
  slot order, and timestamps;
- unique user-named groups;
- many-to-many Pal membership keyed by stable `InstanceId`;
- engine-owned application settings, currently the remembered Global Palbox path and
  its auto-open toggle.

Passive codes are deliberately not foreign keys inside this file because the
reference DB is a separate read-only database. `palbox-core` validates every code
against the reference DB in the same operation that writes a preset.

Per-Pal game values remain exclusively in the loaded `GlobalPalStorage.sav`. Applying
a preset writes its codes to that in-memory Pal; group membership is app-only metadata
and never creates a second copy of mutable Pal state.

## Schema changes

Add a numbered migration to the applicable schema and teach the Rust open path to
migrate older user databases before adding features. Schema v2 adds groups through
`migrations/user-v2-groups.sql`; schema v3 adds app settings through
`migrations/user-v3-app-settings.sql`. Rebuild generated DB files; never edit a `.db`
by hand.
