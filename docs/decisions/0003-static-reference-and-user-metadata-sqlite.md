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

Schema v1 introduced named passive presets containing ordered slots `0..3`. Schema v2
adds user-named groups and many-to-many membership keyed by stable Pal `InstanceId`,
with an automatic numbered migration for existing databases. Rust validates each
passive code against the reference DB before writing or applying it. Applying a preset
modifies only the addressed in-memory Pal; persistence still goes through the existing
backed-up, atomic save workflow. Groups remain app metadata and never enter the save.

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
- The app carries a bundled SQLite dependency and a roughly 17 MB reference resource.
- Browser-only UI preview still needs the legacy JSON fallback until a mock Tauri
  transport replaces it.
