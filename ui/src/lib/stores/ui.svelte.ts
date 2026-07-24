// Shared UI state (Svelte 5 runes module store).
// Import `{ ui }` anywhere and read/mutate its fields reactively — keeps drawer
// state out of any single component so edge tabs, headers, and shortcuts agree.
export const ui = $state({
  /** Left "Global Box" drawer open? */
  leftOpen: false,
  /** Right "Advanced" (IV / Statue) drawer open? */
  rightOpen: false,
});
