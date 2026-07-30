import {
  DEFAULT_BOX_PREFERENCES,
  readStoredBoxPreferences,
  writeStoredBoxPreferences,
} from "$lib/data/boxPreferences";

export const boxPreferences = $state({
  ...DEFAULT_BOX_PREFERENCES,
  loaded: false,
});

function persist() {
  if (typeof localStorage === "undefined") return;
  writeStoredBoxPreferences(localStorage, boxPreferences);
}

export function loadBoxPreferences() {
  if (boxPreferences.loaded) return;
  if (typeof localStorage !== "undefined") {
    Object.assign(boxPreferences, readStoredBoxPreferences(localStorage));
  }
  boxPreferences.loaded = true;
}

export function rememberBoxPath(path: string) {
  boxPreferences.lastBoxPath = path;
  persist();
}

export function setAutoReopen(enabled: boolean) {
  boxPreferences.autoReopen = enabled && Boolean(boxPreferences.lastBoxPath);
  persist();
}
