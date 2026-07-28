# Research — Palworld save-editing landscape (2026-07)

Captured while deciding D1 (core language) / D2 (frontend). Sources are live GitHub repos;
recency is `pushed_at` (last code push) from the GitHub API on 2026-07-24.

## Key correction to an earlier assumption
"Palworld format knowledge lives in Python, so updates land there first" — **not a durable
moat.** The canonical Python parser `cheahjs/palworld-save-tools` (863★) last had a code
push **2024-10-06**. In practice, active editors tend to maintain their own bundled parser
fork in whatever language they're written, so we would maintain one regardless of language
choice.

## Parsers
- **cheahjs/palworld-save-tools** — Python, Palworld-aware, 863★. Canonical but upstream
  stale (last code push 2024-10). Living forks are bundled inside active editors.
- **uesave-rs (trumank)** + **Palworld forks** — Rust. General UE save lib; Palworld forks
  add Level.sav / Pal RawData support. PSP maintains an active fork.
- **DKingAlpha/palworld-uesave-rs** — Rust, 20★, stale (2024-01). Not the live one.
- No maintained **C#**, **Go**, or **pure-JS** Palworld-aware parser found. JS/TS appears
  only as a frontend, or via the Rust parser compiled to WASM (iebb/PalworldSaveEditor).

## Leading modern editors (both put a web UI on the core)
- **KrisCris/Palworld-Pal-Editor** — **Python** (Flask) + bundled save-tools fork; **Vue 3
  / TS** frontend; GUI (embedded web) + WebUI + CLI. 534★, active (pushed 2026-07-20).
- **oMaN-Rod/palworld-save-pal (PSP)** — **Rust** (uesave-rs fork) core; **Svelte**
  frontend; **Tauri** desktop shell; SQLite for presets/settings; WebSocket API. 253★,
  active (pushed 2026-07-19). **Migrated from a Python/FastAPI + palworld-save-tools
  backend to the Rust core** — a mature project that walked the Python→Rust path.

## Implications for our choice
- Realistic core languages: **Python** or **Rust** (others lack a parser to build on).
- Frontend: **web-tech (Svelte leaning)** either way — validated by both leaders; best for
  the Palworld look and the cleanest design handoff.
- **Path A — Rust + Tauri + Svelte (PSP model):** best distribution (single signed binary
  vs. our cx_Freeze pain), fast/safe, the modern convergence point; build on an existing
  1.0-capable Palworld uesave-rs fork rather than porting from scratch. Cost: steeper
  language, slower format iteration, needs Rust appetite.
- **Path B — Python + Svelte (KrisCris model):** reuse the Python 1.0 save parsing we already
  maintained; fastest iteration; Python is where most RE happens. Cost: heavier exe packaging.

## Open input needed
Appetite for a **Rust** core is the deciding factor between A and B. Distribution quality
(clean standalone exe for the Vortex tile) favors A; reuse of our existing 1.0 work and
iteration speed favor B.

## Sources
- https://github.com/cheahjs/palworld-save-tools
- https://github.com/oMaN-Rod/palworld-save-pal
- https://github.com/KrisCris/Palworld-Pal-Editor
- https://github.com/DKingAlpha/palworld-uesave-rs
- https://github.com/iebb/PalworldSaveEditor
- https://github.com/RayChen200318/palworld-save-studio
