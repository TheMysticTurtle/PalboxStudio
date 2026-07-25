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
   provenance, and explicit quality findings.
2. `palbox-user.db` is created in the platform app-data directory and opened
   read-write. It contains only user-authored Palbox Studio metadata.

The first user feature is a named passive preset containing ordered slots `0..3`.
Rust validates each code against the reference DB before writing or applying it.
Applying a preset modifies only the addressed in-memory Pal; persistence still goes
through the existing backed-up, atomic save workflow.

The desktop UI obtains a compact reference bundle through a Tauri command backed by
SQLite. The old JSON is retained only for plain-browser visual previews.

## Source authority

The current Palworld Save Pal game-data extract is authoritative for internal codes
and every static field it exposes. Retained `palworld.tools` and Palworld Wiki
snapshots supplement Partner Skill and Ranch relationships only. The build rejects
referential-integrity errors and records non-fatal corrections or omissions in
`data_quality_issue`.

## Consequences

- Filters can be generated from one relational schema without duplicating per-Pal
  state.
- Static updates are deterministic, checksummed, and reviewable.
- User presets survive application upgrades without making the reference DB mutable.
- The app carries a bundled SQLite dependency and a roughly 17 MB reference resource.
- Browser-only UI preview still needs the legacy JSON fallback until a mock Tauri
  transport replaces it.
