# PalEdit — Palworld 1.0 fork (working notes)

Fork of [EternalWraith/PalEdit](https://github.com/EternalWraith/PalEdit) (Nexus mod 104,
upstream stale at v0.12.1 / pre-1.0) updated for **Palworld 1.0** (released 2026-07-10).
Owner runs it via a Vortex dashboard tool tile pointing at the frozen exe.

## LATEST SESSION (2026-07-22, Opus) — picker/stats/level polish + a data-gap find

Five owner-requested conveniences, each its own feature branch merged to main
with `--no-ff`, tested headless on a Global Palbox copy, exe rebuilt + deployed
to `PalEdit-1.0` (smoke-launched OK), CONTRIBUTIONS.md updated. **7 commits
ahead of the fork (`upstream` = TheMysticTurtle/PalEdit) — owner still needs to
push to update the PR.**

- `feature/shared-ability-picker` — generalised `open_ability_search` (added
  `anchor`, `on_choose` callback, `include_none`, optional `pal`). Now also
  backs (a) the fruit "add a move" box — click opens the full tier/element/sort
  picker, ➕ still commits the staged move (`_pick_fruit_move` + reworked
  `appendMove`), and (b) the four passive-preset slots (`pal=None` → every
  passive). Attack rows tinted by element via `PalElements` + `mean_color`
  (passives already tinted by rating).
- `feature/stats-detail-edit` — `open_stats_detail` IV/soul/condensation rows
  are now spinboxes (IV 0-100, soul 0-20, condensation shown as 0-4 stars →
  `SetRank(v+1)`). Commit on arrow/Enter/focus-out → setter →
  `handleMaxHealthUpdates` → `refresh(i)` → live-recompute the combat column.
- `feature/level-entry` — the `Lv.` label is now `Lv.` + a typeable Entry
  (`self.levelvar`/`self.level`); `setlevelfromentry` clamps 1..levelcap,
  reverts non-numbers, keeps ➖/➕ in sync. `self.level` is now an Entry, not a
  Label — onselect sets `levelvar` (was `.config(text=...)`).

Test scripts (scratchpad): `test_pickers.py`, `test_stats.py`, `test_level.py`
— all ALL PASS. Headless gotchas: no-op `Toplevel.grab_set` and the
`messagebox`/`simpledialog` calls, and DON'T instantiate a 2nd `PalEdit()` in
one process (it fights over `log.txt`) — reload into the same app instead.

- `fix/species-case-insensitive` (DONE, merged) — 6 of the owner's box pals
  were `SheepBall` (**Lamball**, not Melpaca — I mis-called it) and failed to
  load: `KeyError: 'SheepBall'`. Cause: data file `SheepBall.json` has
  `CodeName: "Sheepball"` (psp's casing) but the save's CharacterID is
  `SheepBall`; a Windows case-insensitive FS wrote psp's lowercase content into
  the pre-existing capital-B filename. Fix: `LoadPals` builds `PalSpeciesLower`
  (lowercased CodeName -> canonical) and `PalEntity.__init__` falls back to it
  when the exact key misses (Unreal FNames are case-insensitive). Removed the
  old hardcoded SheepBall fix-up + its stray `print`. Verified: all 6 load as
  Lamball, Unknown list 6 -> 0, box stable 49->49 on no-edit round-trip.

### FOR-LATER LIST (owner asked to record, 2026-07-22)
1. **"Legal abilities only" Tools toggle — likely remove.** It only presets the
   ability pickers' default scope; the pickers already have inline scope
   controls (attack tier dropdown; passive "Natural only" checkbox). Owner
   finds it redundant/confusingly named. Plan: drop the menu item, default the
   pickers to obtainable-first, keep the inline toggles. Check the couple of
   `availableAttacks/availablePassives(pal)` default-scope call sites first.
2. **Standard/world save (`Level.sav`) support — DIAGNOSED 2026-07-22.**
   Reproduced with psp's real 1.0 fixtures (`psp-reference/tests/fixtures/
   saves/v1_relics/` — Level.sav + Players/; also `v1_stats`, `reference_saves`).
   ROOT CAUSE: the *only* thing that breaks is decoding `.worldSaveData.
   GroupSaveDataMap` — the vendored `palworld_save_tools/rawdata/group.py`
   raises `Exception("Warning: EOF not reached")` because 1.0 appended trailing
   bytes to guild entries (psp handles this in `guild.rs` + `guild_tail.rs`).
   Compression is fine (vendored tools handle PlM/PlZ/CNK). Everything else
   PalEdit already skip-decodes. FIX (proven in scratchpad/test_world2.py):
   add `.worldSaveData.GroupSaveDataMap` to the skip_decode list in
   PALEDIT_PALWORLD_CUSTOM_PROPERTIES (PalEdit.py ~line 120, currently
   commented out) — preserve guild data as raw bytes for a safe round-trip.
   With just that, the real Level.sav loads ALL pals (~2148 in v1_relics) + all
   10 players and round-trips. TRADE-OFFS to handle when implementing:
   (a) PalGuid's group methods (AddGroupSaveData etc., used by world-mode
   clone/add) can't run against raw-bytes groups — guard them / mark world
   clone-add as unsupported; editing EXISTING world pals is unaffected.
   (b) pure-Python GVAS parse of a big real world may be slow (test world was
   2.3 MB; owner's could be much larger). Next: branch feature/world-save-
   support, make the one-line change, test a REAL edit round-trip via the app
   path (change a level -> save -> reload -> verify), guard the group ops.
   NOTE debug noise: FilteredPals (or nearby) has a `print(f"Filter ...")` that
   dumps the whole palbox — worth removing while in there.
3. **Stats-detail IV/talents polish.** The editable IV section already shows HP
   / Attack / Defence (physical+magical merged into one attack IV in 1.0 —
   confirmed: single `Talent_Shot`). Owner wants: (a) a section TITLE, renamed
   from "IVs / Talents" to **"Potential (Breeding Traits)"** — but VERIFY the
   term first (Palworld may not officially call IVs "Potential"; the save uses
   `Talent_*`; check psp/in-game wording before committing to it); (b) confirm
   the popup shows exactly HP/Attack/Defence (it does). Possibly also
   consolidate the MAIN window's 4 stat readouts (matk PHY + satk MAG are equal
   now) down to HP/Attack/Defence.

## Layout on this machine

- This repo: `C:\Users\Turtle\Documents\Claude\Projects\PalworldEditor\src`
- Frozen build the owner actually launches: `..\PalEdit-1.0\PalEdit.exe`
  (copy of `build\exe.win-amd64-3.14\` — rebuild + recopy after code changes)
- Original Nexus zip + extracted v0.12.1 for reference: `..\` (project root)
- Owner's saves (**NEVER test against these — always copy to scratchpad first**):
  `%LOCALAPPDATA%\Pal\Saved\SaveGames\76561197997626279\`
  - `GlobalPalStorage.sav` — the Global Palbox; **this is what the owner edits.**
    Their world is not hosted on this machine: there is NO `Level.sav` locally,
    only per-world `LocalData.sav` (client-side) + the global box.

## Build / run

```
python PalEdit.py                  # run from source (needs pillow, pyperclip)
python CxFreezeCompile.py build    # freeze -> build/exe.win-amd64-3.14/
python update_data.py              # refresh game data from psp repo dumps
python update_data.py --icons      # + fetch missing pal icons from paldb CDN
```

Python 3.14.4 locally; cx_Freeze 8.6.4. `palworld_save_tools/` is vendored in-tree
(upstream ships it zipped as `palworld_save_tools.zip`; the extracted dir is what
imports resolve to — includes native Oodle libs under `lib/<platform>/ooz.pyd`).

## What 1.0 changed and how this fork handles it

1. **Compression**: 1.0 saves are Oodle-compressed, magic `PlM`, save_type 0x31
   (pre-1.0 world saves were zlib `PlZ` 0x32). The vendored save-tools fork
   already handled PlM. `loadfile` now stashes `self.save_type` from
   `decompress_sav_to_gvas` and `savefile` reuses it, so round-trips preserve
   the original format. Verified: GVAS write of an unmodified GlobalPalStorage
   is **byte-identical** to the input.
2. **`IsPlayer` key**: 1.0 writes `IsPlayer: False` on every pal's
   SaveParameter. PalInfo used key-presence to detect (and reject) player
   characters → every pal failed to load. Now checks the value.
3. **GlobalPalStorage.sav support** (`storage_mode`): top-level property is
   `SaveParameterArray` → 960 fixed slots, each `{SaveParameter, InstanceId}`;
   empty slots have `CharacterID == "None"`. `loaddata` wraps each occupied
   slot into the Level.sav entry shape
   (`{'key': {'InstanceId': ...}, 'value': {'RawData': {'value': {'object': {'SaveParameter': sp}}}}}`)
   so `PalEntity` mutates the same dicts **by reference** and edits flow back
   into the GvasFile on save. In storage mode: `palguidmanager is None`,
   players dict = `{"Global Palbox": PalInfo.PalStoragePlayer()}`,
   `FilteredPals` returns everything. Spawn/clone/delete already bail on
   `palguidmanager is None` (upstream guards) — see "Feature ideas" below.
4. **Data**: level cap 65 → **80**; passives now have **rank 5**
   (`PalEditConfig.skill_col` gained a 9th color — index = rating + 3, would
   IndexError otherwise); 1.0 SaveParameter has no `Talent_Melee` anymore
   (PalEntity re-adds a default; harmless, game ignores it).

## Game data pipeline (`update_data.py`)

Source of truth: JSON dumps in the **oMaN-Rod/palworld-save-pal** repo
(`data/json/*` — actively maintained, was current within a day of 1.0 patches).
Cached in `psp_data_cache/` (gitignored). Field mapping:

| PalEdit                          | psp source                                  |
|----------------------------------|---------------------------------------------|
| `data/pals/<Code>.json` Type     | `element_types` (+ pad "None"; empty+human → `["None"]`) |
| Moveset `EPalWazaID::X: lvl`     | `skill_set` (add `EPalWazaID::` prefix)     |
| Scaling HP/PHY/MAG/DEF           | `scaling.hp/attack/attack/defense` (PHY=MAG; melee stat is legacy) |
| Suitabilities                    | `work_suitability`                          |
| Human                            | `not is_pal`                                |
| `data/attacks/*` Type/Power/Category | `element` / `power` / `type`           |
| `passives.json` Rating (string!) | `rank` (int, −3..5)                         |
| `<lang>/…` display names         | `l10n/<lang>/…` `localized_name`            |

Gotchas learned the hard way:
- `PalInfo.LoadPals` reads the **per-file dirs** (`data/pals/`, `data/attacks/`),
  NOT the aggregate `data/pals.json` — the aggregate is dead weight.
- `LoadPassives` does `l[code]["Name"]` **unguarded** → every passives.json key
  must exist in every lang's passives.json (updater guarantees fallbacks).
- Per-pal files are read with locale-default encoding (`open(..., "r")`) in the
  frozen 3.14 app → keep generated JSON ASCII (`ensure_ascii` default).
- Never emit a `"Tower"` key in generated pal files: the Tower branch in
  LoadPals does a base-species lookup that breaks when the base sorts after
  GYM_* alphabetically (e.g. WorldTreeDragon). GYM files carry full data instead.
- Icons: `T_<Code>_icon_normal.png` in `resources/pals/` after stripping
  `RAID_`/`_2` (see `GetImage`). Missing → `#ERROR.png` fallback (safe).
  paldb CDN mirrors game texture paths:
  `https://cdn.paldb.cc/image/Pal/Texture/PalIcon/Normal/T_<Code>_icon_normal.webp`.
  ~27 scrapped/quest entities (BeardedDragon, YakushimaBoss*, Quest_Farmer03_*…)
  have no icon anywhere; owner is fine with the placeholder.

## Testing recipe (all against scratchpad copies!)

Headless e2e that exercises the real code paths (no dialogs):
instantiate `PalEdit()`, `gui.withdraw()`, decompress a **copy** of
GlobalPalStorage.sav, `GvasFile.read(..., PALEDIT_PALWORLD_CUSTOM_PROPERTIES)`,
`app.loaddata(gvas)`, edit via real setters (`SetLevel/SetTalentHP/...`),
`app.savefile()` (uses `app.filename`), re-parse and assert. Last run: set a
BerryGoat to lvl 80, IVs 100/100/100 → verified on disk. Note
`PalInfo.logger` is only set inside PalEdit's main; standalone PalInfo use
needs a stub logger.

## Ability search + legality filtering (IMPLEMENTED 2026-07-20)

- `update_data.py` now emits: `Rollable` bool in passives.json (from psp
  `add_pal`/`add_rare_pal`; 85/420 roll on wild pals), `InnatePassives` list
  in per-pal files (e.g. JetDragon = Legend + ElementBoost_Dragon_2_PAL), and
  `Exclusive` species lists on every `Unique_` attack (239 covered) — derived
  from skill_set membership, with fallback parsing of `Unique_<PalCode>_...`
  (exact species match, then startswith for families like Yakushima bosses).
- `PalInfo`: `PassiveRollable` dict, `GetLegalPassives(species)`,
  `PalObject._innate_passives`. Legal passives = rollable ∪ innate.
- `PalEdit`: the 4 passive OptionMenus + 3 equipped-attack OptionMenus now
  intercept `<Button-1>` → `open_ability_search(kind, num)`, a searchable
  Toplevel picker (Entry filters as you type, arrows/Enter/double-click,
  Esc closes). Rows show rating (`Swift  [+4]`) / power (`Fire Tackle  (115)`)
  which also disambiguates duplicate localized names — the picker passes the
  EXACT code (`changeskill(num, code=...)` bypasses the old ambiguous
  name→code index lookup; two passives are both named "Swift"!).
  `availableAttacks/availablePassives(pal)` honour the **Tools > "Legal
  abilities only"** checkbutton (`self.filterlegal`, default on); equipped
  abilities always stay listed. Fruit combobox filters as you type and uses
  the same legality source.
- Tested headless on palbox copies (popup open→search→choose→save→re-parse):
  Caprity can't get Legend when filtered, can when unfiltered; zero foreign
  uniques leak into any box pal's attack list; Rare+Legend and an equipped
  unique survived the disk round-trip. NOTE: many pals legitimately have no
  unique move (e.g. Kitsunebi/Foxparks — all-generic learnset). Synthetic
  `event_generate("<Return>")` is flaky without real focus — test via the
  direct code paths (`changeskill(n, code)`, `attacks[n].set + changeattack`).

## Progress log (2026-07-26 session, Opus continuing Fable's work)

Done and merged to main (each its own feature branch + --no-ff bubble, kept
for cherry-picking):
- `feature/session-backup` — DONE. backup_save() copies the .sav to a
  PalEdit-backups/ folder once per file per session before the first write;
  failed backup aborts the save.
- `feature/nickname-edit` — DONE. Double-click the name label; SetNickname
  writes NickName + FilteredNickName.
- `feature/palbox-add-remove` — DONE. Clone/Add New Pal/Delete work in the
  Global Palbox (storage_mode) against the flat 960-slot array. Add defaults
  to CubeTurtle (Tetroise). Helpers: _palbox_values/_palbox_container_id/
  _next_free_slot_index/_find_empty_palbox_entry/_palbox_insert etc.
- `feature/stale-warning-storage` — DONE. Hides the stale-player warning in
  palbox mode (see research below).
- Also: `.gitattributes` line-ending normalization; DeckIndex/TowerBoss
  metadata emitted by update_data.py; repo renamed src→PalEdit.

### Research findings (task: caps + stale warning)
- **Stale-player warning**: CONFIRMED owner's hunch. Warning text (PalEdit.py
  ~2722) is purely a world-save (Level.sav + Players/*.sav) concern — pals
  break when their owning player's Players/<guid>.sav is stale. Global Palbox
  has NO player data, so it's irrelevant there. Fixed: hidden in storage_mode.
- **Editable caps in the current UI**: Pal Souls (Rank_HP/Attack/Defence/
  CraftSpeed) sliders already 0–20 (matches in-game soul max). Condensation
  Rank UI 1–5 (0–4 stars) = game max. IVs (Talent_*) 0–100. **Work
  suitability spinboxes are capped at 5** (PalEdit.py ~2471 `to=5`) — this is
  the "caps at 5" the owner hit. Raising it (owner wants up to 10 for
  make-anything pals) is an UNRESTRICTED-mode toggle to build in
  `feature/stats-panel` (#18): default = standard caps, toggle = raise the
  work-suitability (and optionally soul/condensation) maxima. Note the game
  itself caps work suitability at 5 by normal means; >5 is a power-user knob.

## PR-READY STATUS (2026-07-26 pt.4) — feature-complete, shipped to PalEdit-1.0

Save corruption fully fixed (no-edit round-trip = zero field changes; verified
a maximally-polluted save repairs to a clean 1.0 save on load+save). All owner-
requested features built, each its own feature/* or fix/* branch merged to
main with --no-ff, exe rebuilt + deployed, CONTRIBUTIONS.md current.

Shipped features (in addition to the 1.0 core update): session-backup,
nickname-edit, palbox add/clone/delete (clone keeps source selection, vacant
slots match game SlotIndex=-1), searchable attack picker (tier/element/sort),
grouped passive picker (natural/all, effect blurbs on hover), pal-list filter
(search/element/category Natural·TowerBosses·Unobtainable·NPCs), searchable
species browser (element/category + multi work-suit + NPC-type filters; shows
"Name (CodeName)"; species selector is a button not a dropdown), merchant/NPC
support, work-suitabilities 0-10 with grey/green/red, effect-generated passive
descriptions, detailed stats popup (current vs level-standard + IVs/souls/
condensation), custom named passive presets, stale-warning hidden in palbox.
Fixes: CraftSpeeds, zero-rank suit bloat + flip-flop, MasteredWaza learnset
pollution, Talent_Melee — all purged; refresh() selection-clear bug.
Facelift was REVERTED (owner wants a real from-scratch UI later).

REMAINING / future: real UI rewrite; clone auto-appear in-game (game-side
refresh, not a format bug — clone is structurally identical to a real pal;
psp gps.rs confirms array index = slot position); comprehensive field audit
(#25); "give any pal any job" (add base-0 work suitability — needs in-game
test of whether AddRank alone grants it). psp-reference/ (gitignored) has the
authoritative source; psp itself is buggy (owner hit load errors) so use it for
mechanics only, don't mimic blindly. BUILD GOTCHA: never chain build+deploy
with bash `&` (wiped PalEdit-1.0 once via robocopy /PURGE mid-build); deploy
with plain /E, smoke-test via PowerShell Start-Process.

## NEXT SESSION PLAN (2026-07-26 pt.3) — do 1.0 properly, not half-assed

STATUS: save-corruption is FIXED. Full no-edit round-trip diff of every pal =
ZERO fields added/removed/mutated (CraftSpeeds, zero-rank suits, learnset in
MasteredWaza, Talent_Melee all purged; suitability flip-flop fixed). Facelift
was REVERTED (owner wants a real from-scratch UI later, not a recolor).
PalEdit-1.0 rebuilt + redeployed clean (I had briefly wiped it with a bad
robocopy /PURGE while the build was mid-flight — fixed; NEVER background the
build in the same bash line as the deploy again).

### BUILD/DEPLOY GOTCHA (caused a wipe today)
Do the rebuild and the smoke-test as SEPARATE steps, never `rm -rf build &&
build && ./exe &` on one line — the `&` backgrounds it and the deploy then
mirrors an empty folder. Deploy with robocopy WITHOUT /PURGE (use plain /E) so
a partial source can't delete the live build. Smoke-test via PowerShell
Start-Process/Stop-Process, not bash `&`.

### THE MERCHANT / NPC "invalid pal object" BUG (owner hit this)
Owner tried to catch/keep merchants; PalEdit threw "invalid pal object".
FINDING: our PalSpecies already HAS all of psp's character keys (834 species;
missing set vs psp = EMPTY, including every Trader/Merchant: Male_Trader01,
VisitingMerchant, PalTrader, BountyTrader, BOSS_* traders, Male_DarkTrader*,
etc.). So the data isn't the gap. The failure is almost certainly in
PalEntity.__init__ / SetType for these NPC/trader characters — likely:
  - the BOSS_ prefix stripping logic (line ~425) or PalSpecies lookup throwing
    for names like `Male_Trader01_v18` (versioned) or `Arena_*`/`SUMMON_*`;
  - or GetImage / suitability / learnset code assuming a pal shape a human
    NPC doesn't have. REPRODUCE FIRST: put a merchant CharacterID (e.g.
    `Male_Trader01`, `VisitingMerchant`) into a scratchpad palbox slot, load
    it, capture the exact exception + line. Then make PalEntity tolerate
    human/NPC characters (Human=true path) end-to-end: load, display, save,
    round-trip. Confirm the localized display NAMES exist for traders in
    en-GB/pals.json (psp l10n has them) so they show a real name not a code.
  Merchants are desirable: catching a merchant lets you buy its wares at base,
  so full NPC support (list, add, keep) is a real feature, not an edge case.

### FEATURE GAP vs palworld-save-pal (box editing only — owner does NOT want map/base/tech editing)
Compare psp-core/src/domain/pal.rs field-by-field against PalInfo. Candidates
psp edits on a pal that PalEdit may not expose yet — audit each and add the
worthwhile ones:
  - Full IV control HP/Shot/Defense with the correct 0-100 (game) vs 0-255
    (raw byte) understanding; psp clamps 0-255. Decide the UI range.
  - Souls Rank_HP/Attack/Defence/CraftSpeed (we have 0-20 sliders — verify
    against psp) and the condensation Rank.
  - Level + Exp (have), Gender (have), Lucky/Boss/Alpha flags (have),
    Nickname (have).
  - Learned vs Equipped moves (fixed today) — cross-check psp active/learned.
  - Passive skills (have, with legality) — cross-check psp passive list caps.
  - Things to CHECK we handle: `Hp` (current HP) not desyncing from max after
    edits; `PhysicalHealth`; `SanityValue`; `FoodWithStatusEffect`; friendship
    (`FriendshipPoint`) for taming/partner; `MasteredWaza`/`EquipWaza` (done);
    `Exp`/level tables. Owner also floated ATTRIBUTE/soul caps to 10 with a
    standard-vs-unrestricted toggle — build alongside the stats panel (#18).
  - Import/Export a pal to JSON (upstream has dumppals/spawnpal for world
    saves; make it work for the Global Palbox too, using the storage_mode
    _palbox_insert path — lets owner share/backup individual pals).

### REQUESTED FEATURE: pal/entity LIST FILTER (do this, mirrors the passive/attack pickers)
Add a filter bar over the main pal list (self.listdisplay) AND reuse it in the
future species browser. Filters:
  - element type, work-suitability, name search (like the ability pickers).
  - a CATEGORY toggle: Obtainable pals (DeckIndex>=0) / Boss & tower pals
    (TowerBoss or BOSS_/GYM_/RAID_) / NPCs & humans & merchants (Human=true).
    Owner explicitly wants NPCs/merchants kept and sortable, NOT hidden.
  DeckIndex/TowerBoss/Human already emitted in per-pal JSON + on PalObject
  (_deck_index/_tower_boss). Build a shared FilterBar so the species selector,
  the palbox list, and the (future) species browser all use one implementation.

### STILL-OPEN from earlier
- Clone auto-populate in Global Palbox: game doesn't slot an inserted clone
  until dragged. Root cause: GlobalPalStorage.sav has no CharacterContainer
  Slots array (world saves do); its SlotIndex is non-authoritative (duplicates
  in real box). NEXT: grep psp for GlobalPalStorage / DimensionPalStorage add
  path; if psp also can't auto-place, make add/clone show a "drag to an empty
  slot in-game" note. Don't invent slot logic.
- "Reset/clean pal" one-click action (owner can do it manually for now via
  re-box → load → save).

### METHODOLOGY THAT WORKED — keep using it
1. For any field question, check the REAL save data (parse a scratchpad copy)
   AND palworld-save-pal source (psp-core/src/domain/{pal,containers}.rs) —
   it's the authoritative public 1.0 editor. 2. Prove correctness with the
   no-edit round-trip diff (load→save→field-by-field diff = must be zero).
   3. One feature/fix per branch, --no-ff merge, test on COPIES only.

## AUDIT (2026-07-26 pt.2) — pre-1.0 cruft purge, researched vs palworld-save-pal

Authoritative reference: **oMaN-Rod/palworld-save-pal** (public 1.0 editor,
Rust). Key files: `psp-core/src/domain/pal.rs`, `.../containers.rs`. Findings:

### CONFIRMED game-breakers — FIXED
1. **Work-suitability zero-entry bloat** (fixed earlier). psp writes work
   suitability ONLY into `GotWorkSuitabilityAddRankList`, filtering zero-rank
   AND unknown names (pal.rs ~621-647). PalEdit wrote 13 zero-entries/pal →
   breaks in-game work. FIXED: prune on load, write only non-zero.
2. **CraftSpeeds phantom field** (FIXED this pass, merged `fix/purge-
   craftspeeds`). psp NEVER writes CraftSpeeds; real 1.0 pals don't have it
   (only `Rank_CraftSpeed`, a soul byte). PalEdit's SetType wrote it on every
   species change. FIXED: stopped writing it + strip on load. This is a prime
   suspect for the owner's "stamped pals suddenly grazing" (species-changed
   pals carried phantom CraftSpeeds into the world).

### CONFIRMED deviation — NOT yet fixed (do next, carefully)
3. **MasteredWaza pollution.** Real save has MasteredWaza EMPTY; psp preserves
   it as-is and inits new pals empty (pal.rs 216/605/744). PalEdit's
   `CleanseAttacks` (PalInfo ~373-419) auto-fills MasteredWaza from the
   learnset + equipped moves on EVERY load → open+save turns 0 entries into N
   for every pal (verified: HawkBird 0→3, Boar 0→2…). More "accidental cheat"
   than save-break, but it's pollution. FIX IS DELICATE: `_learntMoves` is a
   live reference to `MasteredWaza.values`, and the move-edit UI
   (StripAttack/FruitAttack/the learnt-moves listbox) mutates it. Proper fix:
   build the UI's move view as a SEPARATE list; leave MasteredWaza alone
   except when the user explicitly masters/strips a move. Test move add/strip
   round-trips before trusting.

### Clone auto-populate bug — ROOT-CAUSE UNDERSTOOD, needs design
World saves place pals via `worldSaveData.CharacterContainerSaveData.Slots`
(slot_index → RawData.instance_id; nil GUID = empty; SlotNum = capacity —
containers.rs). **GlobalPalStorage.sav has NONE of that — only
SaveParameterArray.** Its `SlotId.SlotIndex` values are heavily DUPLICATED in
the real box (four pals at slot 0, etc., up to 47), so SlotIndex is NOT the
authoritative display position for the global box. That's why an inserted
clone exists in data but the game doesn't slot it into the box view until you
drag it. Our `_next_free_slot_index` is essentially cosmetic here. NEXT:
find how psp/KrisCris add to the *global* box (their world-save path uses the
Slots array, which the global box lacks) — grep psp for GlobalPalStorage /
DimensionPalStorage handling; if none, the global box may only support
placement the game does itself, and add/clone should warn "drag to an empty
slot in-game." Do not invent slot logic.

### Other pre-1.0 smells to clean (low risk)
- Hardcoded test values set during UI build: PalEdit ~2091-2093
  (attacks = GrassPanda/ThreeThunder/DarkLaser) and ~2509-2510 (test skills);
  overwritten on pal load but dead cruft — remove.
- `EmptySuitObject` in EmptyObjectHandler.py now unused (only fed CraftSpeeds)
  — safe to delete.

### Owner symptom mapping
- "stamped pals suddenly grazing" → CraftSpeeds + suitability bloat (both now
  fixed for NEW edits; already-deployed world pals need re-import/reset).
  NEXT SESSION should add a "reset work suitabilities / clean pal" action so
  the owner can repair pals already in their world.
- "clone won't auto-populate, had to drag" → global-box slot model above.

## STOP-THE-DAY STATUS (2026-07-26 end) — READ FIRST TOMORROW

**PR is BLOCKED** until the work-suitability editor is finished (owner's
call). Do not open the pull request yet.

### Shipped today and deployed to PalEdit-1.0 (owner can launch now)
session-backup, nickname-edit, palbox-add-remove, stale-warning-storage,
attack-search, passive-search, and the critical **fix/work-suitability-
corruption** (merged to main, exe rebuilt + deployed).

### The save-corruption bug (FIXED) — what it was
Owner's farm/ranch pals (Penking, Incineram grazing) produced nothing and
showed no work levels. Root cause was TWO defects:
1. `PalEntity.__init__` wrote a zero-rank entry into
   `GotWorkSuitabilityAddRankList` for every suitability on load → opening +
   saving injected 13 phantom entries/pal (286 for the owner's 22-pal box)
   into a list the game leaves empty, breaking in-game work assignment.
   FIXED: prune zero-rank entries on load; `SetSuit` creates an entry only
   for a non-zero bonus and removes it at zero. Loading a corrupted save in
   the new build and saving REPAIRS it (286 → 0, verified).
2. `onselect` set each suitability spinbox value before its minimum, so a
   stale minimum from the previous pal clamped the value; a later click wrote
   the wrong value back (the "flip-flop"). FIXED: configure range before
   value; `setsuits` bails while `is_onselect`.
**Owner recovery:** open the real GlobalPalStorage.sav in the new build and
save (game closed) — the phantom entries are pruned out. Session-backup keeps
the pre-edit copy in a PalEdit-backups/ folder just in case.

### TOMORROW — first job: the work-suitability EDITOR (finishes the PR blocker)
Owner wants: an EXACT readout of every pal's work suitabilities AND the
ability to fully adjust ALL of them (natural + modded, e.g. give Penking a
ranch level it doesn't have naturally). Current UI is 13 spinboxes
(base+added, min=species base, to=5). Needed:
- Show, per suit, the effective level clearly (species base + AddRank bonus),
  ideally labelled so you can SEE which are natural vs added.
- Allow setting any suit 0..N freely (a "standard vs unrestricted" toggle like
  the ability pickers — default caps at 5, unrestricted goes to 10 for the
  owner's make-anything pals). Data path: `GotWorkSuitabilityAddRankList`
  rank = desired_total − species_base (already how SetSuit-via-setsuits works;
  just widen the range and relabel).
- INVESTIGATE (open question): does the game actually grant a suitability the
  pal has NO species base in, purely from a positive AddRank? If not, giving
  Penking ranch may need a different field. Candidates seen on real pals:
  `CurrentWorkSuitability`, `WorkSuitabilityOptionInfo`,
  `WorkSuitabilityOverflowGrantedRankList`. Test in-game with a scratchpad
  copy before promising the "give any pal any job" feature.
- SEPARATE latent bug to fix while here: `PalInfo.SetType` (species change)
  writes a `CraftSpeeds` field that real 1.0 saves DO NOT have (verified:
  0 pals have it). It's probably a pre-1.0 leftover and may itself break work
  calc on species-changed pals. Decide whether to stop writing CraftSpeeds
  entirely (likely yes) — check the game reads work suitability only from
  species data + GotWorkSuitabilityAddRankList.

### TOMORROW — remaining roadmap (after the suitability editor)
- `feature/species-browser` (#14): in-game-style popup — element + work-suit
  filters, name search, ICON thumbnails, category buckets (obtainable via
  DeckIndex>=0 / tower boss via TowerBoss / NPC via Human). Reuse for the
  species selector AND the palbox list. NPCs stay (owner catches merchants).
- `feature/stats-panel` (#18): computed standard stats near the portrait,
  raised/lowered indicators, collapsible fine-tune section. Fold in the
  soul/condensation "unrestricted caps" toggle (research done: souls 0-20 ok,
  condensation 1-5 game max, work-suit cap 5 is the one to raise).
- `feature/ui-facelift` (#19): modernize the whole layout last.
- THEN: write CONTRIBUTIONS.md and open the PR.

## Branch workflow (agreed 2026-07-26 — SUPERSEDES all earlier roadmap notes)

Goal: upstream (EternalWraith) should be able to cherry-pick features with a
clear record; owner gets solid fallback points if a feature breaks.

- `main` — integration line the owner's exe is built from. Merge finished
  features with `--no-ff` so each feature is one visible bubble.
- `base/palworld-1.0` — marker at the 1.0 core update (a13940f).
- `feature/<name>` — one branch per feature, branched from current main,
  self-contained commits with PR-quality messages. Test on scratchpad save
  copies BEFORE merging. Keep the branch after merge (cherry-pick record).
- When everything ships: write `CONTRIBUTIONS.md` mapping every feature →
  branch/commits with a summary (the "small document" for the upstream PR).
- Quality bar: industry best practice, no corner-cutting, comment the code,
  fix genuinely-out-of-place things when touched — without breaking core.

## Feature roadmap (owner's list, 2026-07-26 — OVERRIDES earlier notes where they differ)

1. `feature/session-backup` — automatic backup of the loaded .sav, once per
   edit session, before the first write.
2. `feature/palbox-add-remove` — add NEW pals to the Global Palbox (default
   species: a turtle pal — owner is "The Mystic Testudine", quiet nod), fix
   clone in storage mode (find CharacterID=="None" slot, deepcopy selected
   SaveParameter in, PRESERVE target slot's SlotId, fresh InstanceId GUID,
   loaddata to refresh), and delete (reset slot back to a None template).
3. `feature/nickname-edit` — rename pals (1.0 also has FilteredNickName +
   LastNickNameModifierPlayerUid — check whether they must be set together).
4. `feature/species-browser` — in-game-style browser for BOTH the species
   selector and the palbox list: element + work-suitability filters, name
   search, icon thumbnails next to names (analyze the in-game filter menu
   structure for layout). Category toggle: **obtainable pals / boss pals /
   NPCs** — NPCs STAY available (owner: you can catch merchants and use them
   in base!), just sorted into their own bucket. DeckIndex>=0 = obtainable,
   TowerBoss flag, Human flag = NPC bucket.
5. `feature/attack-search` — same browser aesthetic: filter by element, sort
   by damage, 3-tier toggle: learnset-only / + fruit-teachable / ALL attacks.
   Never hard-restrict — "make it clear where the standard and whacko line
   stands", don't stop anyone from going whacko.
6. `feature/passive-search` — grouped by effect type with a blurb of what
   each adjusts (PassiveDescriptions), updates the visual attribute
   indicators, toggle natural-for-this-pal / all passives.
7. Rank/soul caps: UI caps enhancement at 5 but owner says pals reach 10 via
   mutations — research the real 1.0 caps in game data (psp) and raise;
   toggle standard vs unrestricted attribute editing.
8. `feature/stats-panel` — near the portrait: computed standard attributes
   for the level, indicators for raised/lowered stats, collapsible detailed
   section for fine-tuning breeding-relevant values.
9. `feature/ui-facelift` — modernize the whole UI last ("forgotten 2005
   software"), free rein on layout.
10. RESEARCH: upstream's "players who haven't joined in a while can break
    your save" warning — likely about Level.sav worlds referencing missing
    Players/*.sav; determine if it applies at all to GlobalPalStorage-only
    editing (owner suspects stale; verify and document).

Owner explicitly does NOT want upstream's open-issue backlog solved — only
this list. Palworld Save Pal (oMaN-Rod, open source) may be consulted for
implementation insight, but PalEdit stays streamlined and compact.

## Publishing

Owner publishes via GitHub Desktop (repo folder renamed src→PalEdit so the
default repo name is right). Upstream remote = `upstream`. Owner's
Nexus/Vortex flow unchanged — they launch the local exe, not a download.
