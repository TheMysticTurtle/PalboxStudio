# Palbox Studio databases

Palbox Studio uses two SQLite databases with intentionally separate ownership.

## `palbox-reference.db`

Read-only, generated static Palworld 1.0 reference data:

- species, elements, Work Suitabilities, moves, effects, and learnsets;
- passives, effects, availability flags, and innate relationships;
- items, EXP levels, friendship ranks, and localizations;
- Partner Skills, rank values, same-species engine-variant inheritance, and Ranch products;
- typed editor limits and calculation operands;
- filter-field metadata, source provenance, and quality findings.

This database is a **prebuilt, committed artifact** (`data/palbox-reference.db`): because it
holds static Palworld 1.0 facts, it only needs regenerating when the game itself changes. It
ships in the Tauri bundle, and Rust opens it with SQLite's read-only flag.

The normalized tables are intentionally pleasant to inspect directly. For common
questions, start with the human-readable views:

- `v_species_summary` and `v_species_work_suitability`;
- `v_partner_skill_progression`;
- `v_move_catalog` and `v_move_effect_catalog`;
- `v_passive_catalog` and `v_passive_effect_catalog`;
- `v_reference_sources`.

Those views keep internal codes beside localized names and expose source provenance
without requiring a long hand-written join in DB Browser for SQLite.

The deterministic maintainer tool is `scripts/build_reference_db.py`; its reviewable
inputs live under `data/reference-sources/`. A self-contained extractor that builds
directly from the game's packaged assets is planned future work.

**Provenance & attribution.** The data derives from Palworld 1.0's own game data, cross-checked
against the public community databases ([paldb.cc](https://paldb.cc),
[wiki.gg](https://palworld.wiki.gg)) and the open-source
[palworld-save-pal](https://github.com/oMaN-Rod/palworld-save-pal) project. Save serialization
uses [`uesave-rs`](https://github.com/oMaN-Rod/uesave-rs). Palworld is a trademark of
Pocketpair, Inc.; Palbox Studio is an unofficial, fan-made tool.

## `palbox-user.db`

Writable local app metadata. The app creates the real DB in its platform application-data
folder and migrates existing v1/v2/v3/v4 databases automatically to schema v5.

The current schema stores:

- passive presets with a unique name, unique passive codes, explicit slot order,
  and timestamps;
- unique user-named groups;
- many-to-many Pal membership keyed by stable `InstanceId`;
- engine-owned application settings: the remembered Global Palbox path, its auto-open
  toggle, and the four HP/SAN/Food/Trust MAX preferences.

Passive codes are deliberately not foreign keys inside this file because the
reference DB is a separate read-only database. `palbox-core` validates every code
and the current DB-backed passive-slot limit in the same operation that writes a
preset. The user schema intentionally does not duplicate that patch-sensitive limit.

Per-Pal game values remain exclusively in the loaded `GlobalPalStorage.sav`. Applying
a preset writes its codes to that in-memory Pal; group membership is app-only metadata
and never creates a second copy of mutable Pal state.

## Schema changes

Add a numbered migration to the applicable schema and teach the Rust open path to
migrate older user databases before adding features. Schema v2 adds groups through
`migrations/user-v2-groups.sql`; schema v3 adds app settings through
`migrations/user-v3-app-settings.sql`; schema v4 removes the duplicated passive-slot
cap through `migrations/user-v4-dynamic-preset-slots.sql`; schema v5 adds the four
persisted vital MAX preferences through `migrations/user-v5-vital-max-preferences.sql`.
Rebuild generated DB files; never edit a `.db` by hand.
