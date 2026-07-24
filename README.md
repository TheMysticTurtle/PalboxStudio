# Palbox Studio (provisional name)

A ground-up, well-engineered Palworld 1.0 **global Pal box** editor with a beautiful,
Palworld-styled GUI. A rewrite in intent of the PalEdit 1.0 fork.

Status: **design phase.** Stack is locked (Rust + Tauri + Svelte); the UI is out for a design
pass. See [docs/DIRECTION.md](docs/DIRECTION.md) for the vision,
[docs/DESIGN-HANDOFF.md](docs/DESIGN-HANDOFF.md) for the design brief, and
[docs/decisions/0001-rust-core-tauri-svelte.md](docs/decisions/0001-rust-core-tauri-svelte.md)
for the architecture rationale.

## Principles
- Palworld's visual language; official in-game terminology throughout.
- Modular feature areas over one shared, headless core engine.
- Safety first: never touch live saves — copies, backups, atomic writes only.
- Engineering discipline: commits explain *why*, ADRs for architecture, tests around the core.
