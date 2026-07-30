# Palworld 1.0 Global Palbox — save format notes

How Palworld 1.0 stores the Global Palbox, field by field, and the handful of traps a save editor
has to steer around. Everything here has been verified against real 1.0 `GlobalPalStorage.sav`
files (always on scratchpad copies) and cross-checked against the public game databases listed under
**Sources**. This is the practical reference for the engine and the UI; the exact editable ranges
live alongside it in [SPECS-1.0.md](SPECS-1.0.md).

> **The golden rule:** an edit path is only trustworthy once an *unmodified* box round-trips
> byte-identical. Load → save → diff every field; the result must be zero fields added, removed, or
> changed before any real edit is layered on top.

## The save container

- **File:** `GlobalPalStorage.sav`, located at
  `%LOCALAPPDATA%\Pal\Saved\SaveGames\<id>\GlobalPalStorage.sav`.
- **Compression:** Oodle, magic `PlM`, `save_type` **`0x31`**. (Pre-1.0 world saves used zlib `PlZ`,
  `0x32`.) **Preserve the original `save_type` on write** so an untouched box re-encodes
  byte-identical.
- **Layout:** a top-level `SaveParameterArray` holding **960 fixed slots**, each a
  `{ SaveParameter, InstanceId }` pair. An **empty slot has `CharacterID == "None"`**.
- **Slot position is not authoritative.** `SlotId.SlotIndex` values are heavily duplicated in a real
  box (several Pals can share an index), so they do not describe a Pal's display position. See
  *The Global Palbox slot model* below.

## Per-Pal fields and ranges

The values a Pal actually stores. Ranges are summarized here; [SPECS-1.0.md](SPECS-1.0.md) is the
authoritative source for the exact editable limits.

- **Identity:** `CharacterID` (the species codename), `InstanceId` (a stable per-Pal GUID — this is
  the Pal's identity and the key user metadata is tied to), `NickName` (paired with
  `FilteredNickName`), and gender.
- **IVs / talents (breeding traits):** `Talent_HP`, `Talent_Shot`, `Talent_Defense`. Stored as a raw
  byte (0–255); the game **displays 0–100**, so the editor works in 0–100. Palworld 1.0 uses a
  **single** attack IV, `Talent_Shot` — there is no `Talent_Melee`.
- **Level and experience:** `Level` (a byte, written only when above 1; cap **80**) and `Exp`.
- **Condensation:** the `Rank` byte is stored **one-based from 1–5**, while the game and editor show
  **0–4 stars**. An uncondensed Pal therefore stores `Rank = 1`; four stars stores `Rank = 5`.
  Beyond the flat stat bonus, each in-game rank-up also raises one Work Suitability, and reaching
  max rank raises all of them — worth keeping in mind, because editing `Rank` directly does not
  replay that progression (see *Open questions* below).
- **Pal Souls (Statue of Power):** per-stat ranks **0–20** — `Rank_HP`, `Rank_Attack`,
  `Rank_Defence`, and `Rank_CraftSpeed` (Work Speed). Each rank is +3%, to +60% per stat at rank 20.
  Stored as a byte and written only when non-zero.
- **Work Suitability:** stored in `GotWorkSuitabilityAddRankList` as a **bonus rank**
  (`desired_total − species_base`), and **only non-zero bonuses are written** (see *Corruption
  traps*). Effective level runs **1–10**.
- **Moves:** up to **3** equipped active skills in `EquipWaza`. Learned moves live in `MasteredWaza`,
  which real saves keep **empty** unless a move has been explicitly mastered — so the editor keeps
  its move view separate and does not auto-fill it (see *Corruption traps*).
- **Passive skills:** up to **4** per Pal in `PassiveSkillList`. Passive ranks run **−3..5** (there
  is no rank 0; rank **5** is the 1.0 addition). Legality per species is *rollable ∪ innate*.
- **Vitals and status:** current `Hp`, `SanityValue`, food (`FullStomach`), and `PhysicalHealth`.
- **Trust:** `FriendshipPoint` (an integer), from which the displayed trust rank/progress is derived.
- **Flags:** Lucky, Alpha/Boss, and others. **`IsPlayer` is written `False` on every Pal**, so a
  player character is detected by that *value*, not by the key's presence.

## The save-field map

A quick decoder from raw save property to what the editor shows:

| Save property                     | Editor concept                                  |
|-----------------------------------|-------------------------------------------------|
| `CharacterID`                     | Species (codename → localized display name)     |
| `InstanceId`                      | Pal identity; the key user metadata is tied to  |
| `NickName` / `FilteredNickName`   | Nickname                                        |
| `Talent_HP` / `Talent_Shot` / `Talent_Defense` | IVs (display 0–100, raw byte 0–255) |
| `Level` / `Exp`                   | Level and experience                            |
| `Rank`                            | Condensation (stored 1–5 → displayed 0–4 stars) |
| `Rank_HP` / `Rank_Attack` / `Rank_Defence` / `Rank_CraftSpeed` | Pal Soul ranks (0–20) |
| `GotWorkSuitabilityAddRankList`   | Work Suitability bonuses (total − species base) |
| `EquipWaza`                       | Equipped active skills (up to 3)                |
| `MasteredWaza`                    | Learned moves (left empty unless mastered)      |
| `PassiveSkillList`                | Passive skills (up to 4)                        |
| `Hp` / `SanityValue` / `FullStomach` | Health, sanity, food                         |
| `FriendshipPoint`                 | Trust                                           |
| `IsPlayer` (always `False`)       | Player detection (by value)                     |

## The Global Palbox slot model

The Global Palbox stores its Pals as the flat 960-entry `SaveParameterArray` and **nothing else** —
there is no separate container/slot array describing where each Pal sits in the box view, and
`SlotId.SlotIndex` is not a reliable position (its values repeat across the box). That has one
practical consequence worth stating plainly:

- **A newly added or cloned Pal exists in the data, but the game may not place it into the box view
  until it is dragged to an empty slot in-game.** This is expected, not a corruption — the box
  layout is something the game arranges. The editor should surface a gentle "drag it to an empty slot
  in-game" note rather than trying to invent slot-placement logic.

To add or clone: find a slot whose `CharacterID == "None"`, write the new Pal's `SaveParameter` in,
give it a fresh `InstanceId`, and preserve the target slot's identity. To delete: restore the slot to
a pristine `"None"` vacancy.

## The species roster

The game marks many things as Pal-shaped actors beyond the Pals a player can actually own: raid body
parts, summon and predator encounters, tower models, quest helpers, retired models, and uncatchable
bosses. So a simple "is it a Pal?" (or even "is it not a human?") check is not enough to decide what
belongs in the box.

- All Pal-shaped rows are **retained for decoding**, so an unusual save still resolves every entry to
  the right name, data, and portrait.
- Only the **canonical, owned-Pal species** are offered in the species selector. This selectable set
  is derived from ownership-oriented signals (a valid Paldeck index, an enabled normal owned-species
  actor, and no raid/summon/predator/tower/quest actor code), with one canonical row per
  Paldeck/name/tribe identity.
- A set of same-name / same-tribe encounter and appearance codes map back to their canonical owned
  species, so a loaded variant still shows correctly.

Humans and NPCs cannot live in a Global Palbox, so they never appear as box clutter.

## Corruption traps to avoid

These are specific ways an edit can quietly damage a 1.0 box; the editor avoids each one.

- **Never write the phantom `CraftSpeeds` field.** Real 1.0 Pals do not carry it; writing it (for
  example on a species change) can break in-game work behavior. Only `Rank_CraftSpeed` — a Pal Soul
  byte — is real.
- **Never write zero-rank Work Suitability entries.** Write only the non-zero bonuses into
  `GotWorkSuitabilityAddRankList`; padding it with zero-rank entries breaks in-game work assignment.
- **Register schemas for fields the editor can introduce.** `uesave` records Unreal property tags
  while reading. If an optional property was absent from every source Pal, adding only its value
  leaves serialization without a tag. The core's insert-only writable-schema registry supplies the
  canonical tag without replacing any schema recovered from the source save.
- **Never auto-fill `MasteredWaza` from the learnset on load.** Real saves keep it empty; the
  editor's move view is kept separate, and `MasteredWaza` changes only when the user explicitly
  masters or strips a move.
- **Never re-add `Talent_Melee`.** It is gone in 1.0; the single attack IV is `Talent_Shot`.
- **Never write the displayed condensation stars directly to `Rank`.** Translate editor stars
  `0–4` to the save's one-based byte `1–5`; otherwise every edited Pal appears one rank lower
  in-game.
- **Never overwrite a source that changed after open.** The core records SHA-256, size, and modified
  time, verifies them before backup and again before atomic replacement, and requires the user to
  reopen on conflict.

## Open questions worth an in-game round trip

Some behaviors are best confirmed against a real game rather than assumed. When settling these, use
controlled before/after saves on scratchpad copies and diff the relevant fields:

- **Condensation ↔ Work Suitability coupling.** Because in-game rank-ups also grant Work Suitability
  increases, editing `Rank` alone may leave a Pal whose star rank does not carry the job gains normal
  condensation would have produced. It's worth confirming how (and whether) the game derives those
  gains before deciding how the editor should present the two controls. Palworld Save Pal,
  Palworld-Pal-Editor, and PalEdit all expose condenser/rank, Souls, and Work Suitability as separate
  editing concepts; that supports keeping our mutations independent, but it does **not** replace a
  controlled before/after capture of the game's own condenser action.
- **Runtime normalization.** The game may normalize some runtime values (current HP, food, sanity,
  trust) on load, so an in-game round trip is the final word on how an edited value settles.

## Sources

- **Real 1.0 save bytes** are the final authority for anything format-level — verified on scratchpad
  copies of actual `GlobalPalStorage.sav` files.
- **[paldb.cc](https://paldb.cc/en/v1.0.0)** — the most technical public database; mirrors game data
  for pals, skills, passives, and elements, and hosts the icon/texture CDN. Primary game-data
  reference.
- **[palworld.wiki.gg](https://palworld.wiki.gg/)** — the main community wiki.
- **[Fextralife](https://palworld.wiki.fextralife.com/)** — full Paldeck: stats, breeding, Partner
  Skills, Work Suitabilities.
- **Official patch notes** — the v0.4.11 notes record the Statue of Power enhancement limit rising
  from 10 to 20, the cap that persists in 1.0.

> When a game-data number is in question, prefer the kept-current databases (paldb.cc, wiki.gg) over
> any older guide, and prefer real save bytes for anything format-level.
