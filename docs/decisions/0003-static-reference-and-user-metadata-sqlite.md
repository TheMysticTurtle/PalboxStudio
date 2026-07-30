# ADR 0003: Separate static-reference and user-metadata SQLite databases

Date: 2026-07-25  
Status: Accepted

## Context

The first UI scaffold shipped static game facts as five generated JSON files.
Filters need richer relationships, Partner Skills and Ranch products were missing,
and patch verification needs reviewable provenance. Mutable Pal values already have
one authoritative home: `GlobalPalStorage.sav`. Copying them into an app database
would create synchronization and corruption risks.

Users also need durable app-owned data, beginning with reusable passive presets.

## Decision

Use two databases:

1. `palbox-reference.db` is generated, bundled, normalized, and opened read-only.
   It contains only static Palworld 1.0 facts, localization, filter metadata, source
   provenance, explicit quality findings, canonical Palbox eligibility, and aliases
   from engine encounter codes to owned species.
2. `palbox-user.db` is created in the platform app-data directory and opened
   read-write. It contains only user-authored Palbox Studio metadata.

Schema v1 introduced named passive presets containing ordered entries. Schema v2
adds user-named groups and many-to-many membership keyed by stable Pal `InstanceId`.
Schema v3 adds engine-owned app settings, initially the remembered Global Palbox path
and auto-open toggle. Schema v4 removes the duplicated four-passive database constraint;
the current limit comes from the reference DB and is enforced by the engine. Automatic
numbered migrations preserve existing user metadata.
Rust validates each passive code against the reference DB before writing or applying
it. Applying a preset modifies only the addressed in-memory Pal; persistence still
goes through the existing backed-up, atomic save workflow. Groups and settings remain
app metadata and never enter the save.

The desktop UI obtains a compact reference bundle through a Tauri command backed by
SQLite. Components never query the database directly.

## Source authority

The current Palworld Save Pal game-data extract is authoritative for internal codes
and every static field it exposes. Retained `palworld.tools` and Palworld Wiki
snapshots supplement Partner Skill and Ranch relationships only. The build rejects
referential-integrity errors and records non-fatal corrections or omissions in
`data_quality_issue`.

## Consequences

- Filters can be generated from one relational schema without duplicating per-Pal
  state.
- All 406 Pal-shaped engine rows remain resolvable while the mutation UI exposes only
  the 287 audited, unique, transferable species.
- Static updates are deterministic, checksummed, and reviewable.
- User presets survive application upgrades without making the reference DB mutable.
- Group names and memberships survive application upgrades independently of save writes.
- Remembered-box and auto-open preferences survive application upgrades without relying on the
  webview profile.
- The app carries a bundled SQLite dependency and a roughly 17 MB reference resource.
- Browser-only UI preview has no authoritative reference-data fallback; validate
  reference-driven behavior through the desktop engine.
