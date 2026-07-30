# Palbox Studio — Global Palbox Identity, File Lifecycle, and Live-Editing Investigation

## Purpose

Document what is currently known about `GlobalPalStorage.sav`, identify the risks around swapping multiple Global Palboxes and editing while Palworld is running, and define a controlled investigation plan before Palbox Studio claims or enables live in-game editing.

This work should follow the current save-integrity hardening. Live editing must remain experimental until Palworld's actual load, cache, and write behavior is proven.

---

## 1. Current findings

### 1.1 Global Pal identities are GUID-based, not sequential counters

Each occupied Global Palbox entry carries an `InstanceId`. This is a GUID/UUID-style identity, not a small counter that restarts at zero in every new box.

A separate `SlotIndex` may start at zero in each container. Duplicate slot indexes across different boxes are expected and do not imply duplicate Pal identities.

Palbox Studio currently:

- Preserves the `InstanceId` of an existing Pal.
- Generates a fresh UUID v4 when adding a new Pal.
- Generates a fresh UUID v4 when cloning a Pal.
- Assigns the next available slot index separately.

### 1.2 Multiple fresh Global Palboxes should contain unrelated identities

Two independently created Global Palboxes may both have a Pal at slot index `0`, but their Pal `InstanceId` values should be different.

The practical duplicate-ID risk is not random collision. It is copying or forking existing data while preserving the original Pal identities.

Examples:

- **Fresh box A + fresh box B:** expected to be safe.
- **Box B copied from box A:** may contain the same Pal identities.
- **A Pal exported from a world after originally being imported from another Global Palbox:** identity behavior needs controlled confirmation.

### 1.3 Swapping Global Palbox files appears conceptually viable

Expected workflow:

1. Import Pals from Global Palbox A into a test world.
2. Close or otherwise safely unload the Global Palbox.
3. Replace `GlobalPalStorage.sav` with independently created Global Palbox B.
4. Import Pals from B into the same world.
5. Optionally export a world Pal into B.

The unresolved question is whether exporting a previously imported Pal preserves its original Global Pal identity, assigns a new identity, or stores a separate genetic-record identity. That determines what happens when the same Pal exists in two different Global Palbox files.

### 1.4 Palworld appears to maintain an in-memory Global Palbox subsystem

Reverse-engineered SDK headers expose a `UPalGlobalPalStorageSubsystem` implemented as a `UGameInstanceSubsystem`. It contains:

- A transient in-memory `SaveParameterArray`.
- An `IsLoadedData` flag.
- Load, save, and import delegates.
- Lookup and duplicate-check functions keyed by `FPalInstanceID`.

The Global Palbox UI separately exposes methods named:

- `LoadGPSData()`
- `SaveGPSDataAsync()`
- `ImportGPSData(...)`
- `ExportGPSData(...)`
- `DeleteGPSData(...)`

This strongly suggests that Palworld loads Global Palbox data into memory, operates on that cached array, and writes it back asynchronously.

It does **not** prove exactly when the initial load occurs or whether reopening the menu forces a disk reload.

### 1.5 Live external editing is therefore not yet safe to assume

Several architectures remain possible:

1. **Load whenever the Global Palbox UI opens**
   - Close menu, save from Studio, reopen menu.
   - Near-live editing may work.

2. **Load once per game instance or world session**
   - Studio can change the disk file, but Palworld continues using stale in-memory data.
   - Palworld may later overwrite Studio's changes.

3. **Load at main menu or world entry**
   - Editing while at the main menu may work.
   - Editing after loading a world may not.

4. **Conditional reload**
   - The UI may call `LoadGPSData()`, but `IsLoadedData` may cause it to reuse the cached array instead of rereading disk.

Until tested, Palbox Studio should continue recommending that Palworld be fully closed before saving.

---

## 2. Save-integrity behavior Palbox Studio should implement first

### 2.1 Source-file fingerprint

When a box is opened, record:

- File path
- File size
- Last-modified time
- SHA-256 hash of the complete file

The hash should be authoritative. Size and timestamp are useful for quick checks and diagnostics, but they are not sufficient on their own.

### 2.2 Recheck immediately before replacement

Before saving:

1. Flush intentional UI edits into the in-memory model.
2. Encode and validate the proposed output.
3. Re-read and hash the source file.
4. Compare it with the opening fingerprint.
5. Refuse replacement if the source changed externally.
6. Only then create the verified backup and perform the staged replacement.

The external-change check should occur as late as practical to reduce the time-of-check/time-of-use window.

### 2.3 Source monitor for user experience, not authority

A cross-platform file watcher or bounded polling monitor may provide early notification that Palworld or another program changed the file.

Monitor notifications should:

- Mark the source as externally changed.
- Show a clear reload/conflict notice.
- Never be the sole basis for allowing a save.
- Be confirmed by a fresh hash before replacement.

### 2.4 Conflict policy

#### Clean Studio session + external file change

Offer:

- Reload the changed file.
- Keep viewing the old in-memory copy without saving.
- Open the changed file as a separate session later.

#### Dirty Studio session + external file change

Block saving over the source. Offer:

- Discard Studio edits and reload.
- Preserve the current work as an app-level snapshot or export.
- Cancel and investigate manually.

Do not attempt automatic three-way merging in the first implementation.

### 2.5 Post-save monitoring

After Studio replaces the file:

- Record the new fingerprint.
- Ignore watcher events that match Studio's own known write.
- Continue watching briefly.
- Warn if Palworld immediately overwrites the file with a different hash.

This would reveal unsafe live-edit states and prevent the user from assuming a successful Studio save remained in place.

---

## 3. Controlled investigation plan

### Phase A — Build a safe test environment

Use:

- A disposable local world.
- Backups of the world and every Global Palbox file.
- Steam/cloud synchronization disabled or isolated during testing.
- A dedicated test Pal with an obvious nickname.
- Two independently created Global Palbox files, A and B.
- SHA-256 hashes and timestamps logged before and after every action.

Do not use a primary world or the user's only Global Palbox.

### Phase B — Observe disk activity with Process Monitor

Filter Process Monitor to:

- The Palworld process.
- Paths ending in `GlobalPalStorage.sav`.

Record these operations where available:

- `CreateFile`
- `ReadFile`
- `WriteFile`
- Rename/replacement operations
- Delete/disposition operations
- File close events

Mark timestamps for each checkpoint:

1. Start Palworld.
2. Reach the main menu.
3. Load a world.
4. Approach or interact with the Global Palbox.
5. Open the Global Palbox UI for the first time.
6. Close and reopen the UI.
7. Import a Pal.
8. Export a Pal.
9. Delete or favorite a Global Pal entry.
10. Close the UI.
11. Return to the title screen.
12. Exit Palworld.

Questions to answer:

- At what point is the file first read?
- Is it read again when the UI reopens?
- Are writes immediate after import/export, delayed until menu close, delayed until title screen, or delayed until process exit?
- Does the game replace the file atomically or write it in place?
- Does simply viewing the Global Palbox cause a write?

### Phase C — External-edit behavior matrix

Use a harmless visible change, preferably a unique nickname.

#### Test 1 — Palworld closed

Baseline expected behavior:

1. Edit with Studio.
2. Start Palworld.
3. Confirm the edit appears.
4. Exit normally.
5. Confirm the edit remains on disk.

#### Test 2 — Palworld at main menu

1. Start Palworld and remain at the main menu.
2. Edit and save with Studio.
3. Load the test world.
4. Open the Global Palbox.
5. Confirm whether the edit appears.
6. Exit and confirm whether it remains on disk.

#### Test 3 — World loaded, Global Palbox never opened

1. Load the test world.
2. Do not open the Global Palbox.
3. Edit and save with Studio.
4. Open the Global Palbox for the first time.
5. Confirm whether the edit appears.
6. Trigger one normal game-side Global Palbox action.
7. Confirm whether Palworld preserves or overwrites the edit.

#### Test 4 — Global Palbox opened once, then closed

1. Open and close the Global Palbox.
2. Edit and save with Studio.
3. Reopen the Global Palbox.
4. Confirm whether the edit appears.
5. Trigger a normal game-side save action.
6. Confirm the final disk contents.

#### Test 5 — Menu remains open

Initially observe only; do not enable this as normal behavior.

1. Leave the Global Palbox UI open.
2. Record whether Palworld holds the file open.
3. Attempt an external save only on a disposable copy and only after earlier phases are understood.
4. Watch for file sharing violations, ignored changes, crashes, stale display, or later overwrite.

#### Test 6 — Return to title screen

Determine whether returning to title:

- Flushes the in-memory array.
- Destroys and recreates the game-instance subsystem.
- Causes the next world load to reread the file.

#### Test 7 — Full process restart

Confirm that a complete restart always reloads the externally modified file and establishes the conservative supported workflow.

---

## 4. Multi-box identity test matrix

### Test A — Independent fresh boxes

1. Create box A and box B independently.
2. Add one Pal to each.
3. Confirm their `InstanceId` values differ.
4. Import both into the same disposable world.
5. Confirm both remain usable after restart.

Expected result: no identity collision.

### Test B — Copied/forked box

1. Copy box A to create box C.
2. Modify the copied Pal without changing its identity.
3. Import the box-A Pal into the world.
4. Swap to box C and attempt to import its matching identity.
5. Record whether the game blocks, replaces, merges, or duplicates it.

Purpose: document the game's behavior when two files carry the same genetic identity.

### Test C — Export an imported Pal into a different box

1. Import a Pal from box A into the test world.
2. Swap to fresh box B.
3. Export that world Pal into B.
4. Compare:
   - The original box-A Global Pal `InstanceId`
   - The world Pal identity
   - The new box-B Global Pal identity
5. Attempt to use both box A and box B in the same world.

This directly answers whether Palworld preserves the original global identity or assigns a new one on export.

### Test D — Re-export and duplicate checks

Repeat export of the same Pal into:

- The same Global Palbox.
- A different empty slot.
- A different Global Palbox file.

Record the behavior of duplicate checks, overwrite prompts, and identity values.

---

## 5. Optional runtime instrumentation

If Process Monitor and controlled file swaps do not fully resolve the lifecycle, build a small diagnostic mod that logs calls around the Global Palbox subsystem.

Useful observation points:

- Subsystem initialization and deinitialization
- `LoadGPSData`
- `SaveGPSDataAsync`
- Load/save delegates
- Import/export calls
- `IsLoadedData` before and after calls
- `SaveParameterArray` count and selected test identity
- World entry, world exit, return to title, and process shutdown

The generated SDK `.cpp` files contain stubs, not Pocketpair's real implementation. Runtime instrumentation is the definitive way to learn when these methods are actually called.

Keep the diagnostic mod separate from Palbox Studio. Its purpose is evidence collection, not a required runtime dependency.

---

## 6. Recommended product modes

### Supported mode — game closed

Default and fully supported.

- Palworld must be closed.
- Normal fingerprint, backup, validation, and replacement protections apply.

### Experimental mode — main menu

Only expose after the main-menu test repeatedly succeeds.

- Detect that Palworld is running.
- State that only main-menu editing has been validated.
- Watch for a later game overwrite.
- Keep verified backups.

### Experimental mode — world loaded, box not opened

Only expose if testing proves the subsystem has not loaded yet and Palworld does not overwrite the edit.

### Unsupported mode — Global Palbox already loaded/opened

Keep blocked or heavily warned until testing proves a safe reload procedure.

A future supported live-edit flow might require:

1. Close the Global Palbox UI.
2. Save from Studio.
3. Explicitly trigger a reload in-game.
4. Reopen the UI.
5. Verify the disk hash remains stable.

Do not infer that closing and reopening the menu reloads the file; prove it.

---

## 7. Acceptance criteria before claiming live editing

Live or near-live editing is supportable only when all of the following are demonstrated:

- The exact read point is known.
- The exact write points are known.
- Palworld does not silently overwrite a successful Studio edit in the supported state.
- Studio detects every observed external change before replacement.
- A failed or conflicting save leaves the original or a verified backup.
- The supported reload procedure works repeatedly across restarts.
- Behavior is confirmed on Windows and, where relevant, Linux/Proton.
- Multi-box identity behavior is documented.
- Exporting an already imported Pal into another box has a known identity outcome.
- Automated regression tests cover stale-file refusal and conflict handling.

---

## 8. Recommended backlog additions

### Tier 0 — save integrity

- [x] **Implement source-file fingerprints and stale-write refusal.** Store SHA-256, size, and modified time at open; recheck immediately before replacement and block if the source changed.
- [x] **Add external-change monitor and conflict UI.** The UI polls the core's fresh content fingerprint, preserves in-memory edits, blocks Save, and offers explicit reload; persist-time hash verification remains authoritative.
- [x] **Detect post-save overwrite.** A source change observed during the 30 seconds after Studio saves receives a distinct overwrite warning; monitoring continues for later conflicts.
- [ ] **Add a safe app-level snapshot/export for dirty conflicts.** Let users preserve unsaved work without overwriting an externally changed source.

### Investigation

- [?] **Determine Palworld's `GlobalPalStorage.sav` lifecycle.** Use Process Monitor and controlled state-by-state edits to determine load, cache, and write timing.
- [?] **Confirm multiple Global Palbox identity behavior.** Test independent boxes, copied boxes, and exporting an imported Pal into a second box.
- [?] **Evaluate main-menu and live editing.** Keep game-running support experimental until repeatable behavior and overwrite safety are proven.

### Later refinement

- [ ] **Add Palworld-running detection and clearly defined operating modes.**
- [ ] **Consider a diagnostic runtime mod for subsystem call logging.**
- [ ] **Document the validated Global Palbox swap workflow for users.**

---

## 9. Evidence sources reviewed

Repository/source paths reviewed during this investigation:

- Palbox Studio: `core/src/globalbox.rs`
- Palworld Modding Kit: `Source/Pal/Public/PalUIGlobalPalStorage.h`
- Palworld Modding Kit: `Source/Pal/Public/PalGlobalPalStorageSubsystem.h`
- Generated SDK: `WBP_GlobalPalStorage_ForDisplay_classes.hpp`

These sources establish the saved identity structure and exposed subsystem/UI architecture. They do not expose Pocketpair's full runtime implementation, so the lifecycle conclusions remain hypotheses until observed at runtime.

---

## 10. Known Work Suitability save failure

### Exact reported error

The following error has now been reported by more than one user while adjusting a Pal's Work Suitability:

```text
write_sav: missing property schema for path: SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.WorkSuitability
```

### Likely trigger

The failure appears to depend on the shape of the original save rather than the selected Work Suitability itself.

Palbox Studio currently rebuilds `GotWorkSuitabilityAddRankList` from scratch whenever Work Suitability is applied. Each non-zero entry contains:

- `WorkSuitability` — enum value such as `EPalWorkSuitability::Handcraft`
- `Rank` — integer AddRank bonus

`uesave` records property schemas while reading the original file. If the original Global Palbox did not contain a populated Work Suitability bonus row, it may have no recorded schema for the nested `WorkSuitability` field. Palbox Studio can build the in-memory property tree, but `write_sav` then refuses to serialize the newly introduced field because its exact dotted-path schema is missing.

This explains why the problem can appear intermittent:

- Editing a Pal/file that already contains populated Work Suitability bonus data may work.
- Adding the first Work Suitability bonus to a Pal or box with no prior populated row may fail.
- A newly added or cloned Pal may expose the problem more readily depending on the source slot/schema history.

The error occurs during encoding. The existing replacement pipeline should therefore stop before replacing the original file, but this must be covered by an explicit regression test.

**Implementation status — fixed on `fix/engine-save-authority` (`11b6d1b`).** The core write
boundary now installs every currently writable property schema without replacing source schemas.
The exact nested error is reproduced with raw `uesave`, repaired by the core boundary, and
round-tripped on both the committed sanitized fixture and a scratch copy of a current real save.

### Required schema registration

Before serialization, ensure the save carries schemas for all three paths:

```text
SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList
SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.WorkSuitability
SaveParameterArray.SaveParameter.GotWorkSuitabilityAddRankList.Rank
```

Expected tags:

| Path suffix | Required schema |
|---|---|
| `GotWorkSuitabilityAddRankList` | Array of struct `PalWorkSuitabilityInfo` |
| `.WorkSuitability` | Enum `EPalWorkSuitability` |
| `.Rank` | `IntProperty` |

Implementation rules:

1. Register a schema only when the real save did not already provide one.
2. Never overwrite a schema read from the source file.
3. Perform schema registration centrally for every writable `SaveParameter` field rather than adding one-off fixes inside UI handlers.
4. Call the registration before `write_sav`, or at session initialization before any edit can introduce an absent property.
5. Keep the existing behavior of omitting zero-rank entries and removing the array when no non-zero entries remain.

### Required tests

- [x] Start from a fixture with no `GotWorkSuitabilityAddRankList`; add one Work Suitability bonus; encode, decode, and verify it.
- [ ] Start from an empty array with no element schemas; add the first row and round-trip it.
- [ ] Edit a Pal that already contains a Work Suitability row.
- [x] Add several different Work Suitability rows and verify canonical order.
- [x] Set the final non-zero bonus to zero and verify that the property is removed.
- [x] Verify unknown Work Suitability names are rejected or omitted.
- [x] Verify a forced `write_sav` failure leaves the original untouched and creates no unnecessary backup.
- [x] Run the same tests against a sanitized real Global Palbox fixture in CI.

### Backlog entry

- [x] **Register all writable Global Palbox property schemas.** Fix the reported Work Suitability serialization failure by registering the array, nested enum, and rank schemas before writing. Expand this into a complete schema list for every property Palbox Studio can create when it was absent from the source save.

---

## 11. Ascending / descending sort-direction toggle

> This section treats “ascension toggle” as the ascending/descending sort toggle discussed for the box explorer.

### Current behavior

The compact Global Box drawer currently filters the source list but does not apply an explicit sort. It therefore displays Pals in the order supplied by the box model.

The expanded Global Palbox matrix has a local sort selector with three choices:

- Box order
- Name
- Level

Its directions are currently hard-coded:

- Box order — ascending slot
- Name — ascending name
- Level — descending level

There is no direction control, and the compact drawer and expanded matrix do not share one sort state.

### Recommended implementation

This is a frontend-only feature. It does not require a Rust-core, Tauri-command, or save-format change.

Create shared box-explorer state:

```ts
export type BoxSortKey =
  | "slot"
  | "name"
  | "level"
  | "condensation";

export type SortDirection = "asc" | "desc";
```

Recommended state fields:

```ts
sortKey: BoxSortKey;
sortDirection: SortDirection;
search: string;
selectedGroups: Set<number>;
speciesFilter: SpeciesFilterState;
```

Place them in a shared `boxExplorer.svelte.ts` store so:

- The compact drawer and expanded matrix show the same results.
- Expanding or collapsing the box does not reset search, filters, groups, sort key, or direction.
- The comparison logic exists in one tested helper rather than two components.

Use a pure comparator with a stable slot fallback:

```ts
export function compareBoxPals(
  a: BoxPal,
  b: BoxPal,
  key: BoxSortKey,
  direction: SortDirection,
): number {
  let primary = 0;

  switch (key) {
    case "name":
      primary = a.name.localeCompare(b.name, undefined, {
        sensitivity: "base",
        numeric: true,
      });
      break;
    case "level":
      primary = a.level - b.level;
      break;
    case "condensation":
      primary = a.condensation - b.condensation;
      break;
    case "slot":
    default:
      primary = a.slot - b.slot;
      break;
  }

  if (primary !== 0) {
    return direction === "asc" ? primary : -primary;
  }

  return a.slot - b.slot;
}
```

Sort a copy rather than mutating the authoritative tile list:

```ts
return [...rows].sort((a, b) =>
  compareBoxPals(a, b, sortKey, sortDirection)
);
```

### UI recommendation

Place a compact direction button directly beside the Sort selector:

```text
Sort: [ Level ▼ ] [ ↓ ]
```

Requirements:

- Display `↑` for ascending and `↓` for descending.
- Tooltip and accessible label must say `Sort ascending` or `Sort descending`.
- Support mouse, keyboard, and controller/focus navigation.
- Keep the selected direction visible rather than hiding it inside the option text.
- Do not physically reorder `box.tiles` or modify the save. This is a display-only sort.

Suggested natural defaults when a key is first selected:

| Sort key | Default |
|---|---|
| Box order | Ascending |
| Name | Ascending |
| Level | Descending |
| Condensation | Descending |

After the user presses the direction button, preserve their explicit choice until they select a different sort key or reset filters.

### Scope options

#### Minimal patch

- Add `sortDirection` to `BoxMatrix.svelte`.
- Normalize all comparators to ascending and invert the primary result for descending.
- Add the arrow button.

This is fast, but the compact drawer still lacks sorting and expanding the view still changes behavior.

#### Recommended patch

- Create one shared explorer/filter/sort store.
- Move the visible-list calculation into a pure helper.
- Use it in both `GlobalBoxDrawer.svelte` and `BoxMatrix.svelte`.
- Add sort selector and direction button to both views, or keep the controls in one persistent shared toolbar.

### Required tests

- [ ] Slot ascending and descending.
- [ ] Name ascending and descending, case-insensitive and numeric-aware.
- [ ] Level ascending and descending.
- [ ] Condensation ascending and descending.
- [ ] Equal primary values fall back to ascending slot order.
- [ ] Filtering occurs before sorting.
- [ ] Compact and expanded views produce the same ordered instance IDs.
- [ ] Changing sort direction never changes `box.tiles` or marks the save dirty.
- [ ] Search/filter/sort state survives expand and collapse.
- [ ] Direction button exposes the correct accessible label and pressed state.

### Backlog entries

- [ ] **Add ascending/descending box sorting.** Add a visible direction toggle beside the sort selector with stable tie-breaking and no mutation of save order.
- [ ] **Share explorer state between compact and expanded views.** Search, filters, groups, sort key, and direction should persist when switching layouts.
- [ ] **Add Condensation as a sort key.** The value already exists on `BoxPal`; this is a small UI-only addition once shared sorting is implemented.
