# Palbox Studio (provisional name)

A ground-up, well-engineered Palworld 1.0 **global Pal box** editor with a beautiful,
Palworld-styled GUI. A rewrite in intent of the PalEdit 1.0 fork.

Status: **pre-scaffold / discussion.** See [docs/DIRECTION.md](docs/DIRECTION.md) for the
vision and the open technical decisions we're settling before writing code.

## Principles
- Palworld's visual language; official in-game terminology throughout.
- Modular feature areas over one shared, headless core engine.
- Safety first: never touch live saves — copies, backups, atomic writes only.
- Engineering discipline: commits explain *why*, ADRs for architecture, tests around the core.
