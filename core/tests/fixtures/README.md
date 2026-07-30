# Synthetic Global Palbox fixture

`synthetic-global-palbox.sav` is a distributable regression fixture for the
real Palworld 1.0 Global Palbox codec.

It was produced from a scratchpad copy, never a live save:

1. Decode the copied `GlobalPalStorage.sav`.
2. Select one game-produced empty slot.
3. Replace all 960 slots with that empty template.
4. Clear trailing bytes.
5. Add one synthetic `CubeTurtle` through the Rust core, producing fresh
   non-user GUIDs and fixed health/food defaults.
6. Re-encode and decode the generated file.

The retained blank-slot strings were audited before generation. They contain
only empty/default values, nil GUIDs, and Palworld enum names—no player name,
Steam ID, owner GUID, nickname, or other user data.

SHA-256:
`25062750F7960B1D2BF81BC8390840FDF87DE0F4541D87C0FA1C349473634D0F`
