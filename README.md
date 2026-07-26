# Palbox Studio (provisional name)

A ground-up, well-engineered Palworld 1.0 **global Pal box** editor with a beautiful,
Palworld-styled GUI. A rewrite in intent of the PalEdit 1.0 fork.

Status: **functional pre-release.** Real Global Palbox saves load, edit, back up, verify, and
round-trip through the Rust engine; the Tauri/Svelte editor and reference-backed filters are
working. Installer/bundle validation is the next release step. See
[docs/PROGRESS.md](docs/PROGRESS.md) for the current handoff,
[docs/DIRECTION.md](docs/DIRECTION.md) for the vision,
[docs/SPECS-1.0.md](docs/SPECS-1.0.md) for the authoritative 1.0 data specs, and
[docs/decisions/0001-rust-core-tauri-svelte.md](docs/decisions/0001-rust-core-tauri-svelte.md)
for the architecture rationale.

## Layout
- `core/` — headless Rust engine (`palbox-core`): Palworld 1.0 save model, load/write,
  mutations, verified 1.0 limits. No UI, no Tauri; unit-tested.
- `src-tauri/` — the Tauri desktop shell (`palbox`); depends on `core`, bridges it to the UI.
- `ui/` — the Svelte + Vite frontend (SvelteKit static/SPA).
- `database/` + `data/` — normalized static-reference/user schemas, generated SQLite DBs,
  and retained evidence for externally supplemented 1.0 facts.
- `design/` — design source-of-truth (the State A prototype + notes).

## Develop
Prereqs: Rust (stable-msvc) + Node. Then from the repo root:

```bash
npm install && npm --prefix ui install   # one-time
npm run tauri dev                          # run the app (hot-reloads the UI)
```

On Windows, after installing those one-time dependencies, double-click
`Launch Palbox Studio.cmd` in the repository root. Keep its terminal window open while using the
development build; it owns the hot-reload server and desktop process.

`npm run build` builds the frontend; `cargo test` runs the core tests. Rebuild and verify
the reference/user database templates with `python scripts/build_reference_db.py --check`.

## Principles
- Palworld's visual language; official in-game terminology throughout.
- Modular feature areas over one shared, headless core engine.
- Safety first: never touch live saves — copies, backups, atomic writes only.
- Engineering discipline: commits explain *why*, ADRs for architecture, tests around the core.
