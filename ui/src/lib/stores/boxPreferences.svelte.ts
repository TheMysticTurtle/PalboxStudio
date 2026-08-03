import {
  clearLegacyBoxPreferences,
  DEFAULT_BOX_PREFERENCES,
  readLegacyBoxPreferences,
  shouldMigrateLegacyBoxPreferences,
  type BoxPreferencesValue,
} from "$lib/data/boxPreferences";
import {
  getAppPreferences,
  saveAppPreferences,
} from "$lib/data/engine";
import { isTauri } from "@tauri-apps/api/core";

export const boxPreferences = $state({
  ...DEFAULT_BOX_PREFERENCES,
  loaded: false,
  error: "",
});

let loadPromise: Promise<void> | null = null;

export type VitalMaxPreference = "maxHp" | "maxSanity" | "maxFood" | "maxTrust";

function currentPreferences(): BoxPreferencesValue {
  return {
    lastBoxPath: boxPreferences.lastBoxPath,
    autoReopen: boxPreferences.autoReopen,
    maxHp: boxPreferences.maxHp,
    maxSanity: boxPreferences.maxSanity,
    maxFood: boxPreferences.maxFood,
    maxTrust: boxPreferences.maxTrust,
  };
}

async function persist(next: BoxPreferencesValue): Promise<boolean> {
  try {
    const stored = await saveAppPreferences(next);
    Object.assign(boxPreferences, stored);
    boxPreferences.error = "";
    if (typeof localStorage !== "undefined") {
      clearLegacyBoxPreferences(localStorage);
    }
    return true;
  } catch (error) {
    boxPreferences.error =
      `Could not save preferences to palbox-user.db: ${String(error)}`;
    return false;
  }
}

export async function loadBoxPreferences(): Promise<void> {
  if (boxPreferences.loaded) return;
  if (loadPromise) return loadPromise;
  loadPromise = (async () => {
    const legacy = typeof localStorage === "undefined"
      ? { ...DEFAULT_BOX_PREFERENCES }
      : readLegacyBoxPreferences(localStorage);
    try {
      let stored = await getAppPreferences();
      if (shouldMigrateLegacyBoxPreferences(stored, legacy)) {
        stored = await saveAppPreferences(legacy);
      }
      Object.assign(boxPreferences, stored);
      if (typeof localStorage !== "undefined") {
        clearLegacyBoxPreferences(localStorage);
      }
    } catch (error) {
      // Browser-only previews have no engine bridge. Retaining the legacy
      // values here also keeps a failed DB migration recoverable.
      Object.assign(boxPreferences, legacy);
      boxPreferences.error = isTauri()
        ? `Could not load preferences from palbox-user.db: ${String(error)}`
        : "";
    } finally {
      boxPreferences.loaded = true;
    }
  })();
  return loadPromise;
}

export async function rememberBoxPath(path: string): Promise<boolean> {
  const previous = currentPreferences();
  const next = { ...previous, lastBoxPath: path };
  Object.assign(boxPreferences, next);
  if (await persist(next)) return true;
  Object.assign(boxPreferences, previous);
  return false;
}

export async function setAutoReopen(enabled: boolean): Promise<boolean> {
  const previous = currentPreferences();
  const next = {
    ...previous,
    autoReopen: enabled && Boolean(previous.lastBoxPath),
  };
  Object.assign(boxPreferences, next);
  if (await persist(next)) return true;
  Object.assign(boxPreferences, previous);
  return false;
}

export async function setVitalMaxPreference(
  preference: VitalMaxPreference,
  enabled: boolean,
): Promise<boolean> {
  const previous = currentPreferences();
  const next: BoxPreferencesValue = { ...previous, [preference]: enabled };
  Object.assign(boxPreferences, next);
  if (await persist(next)) return true;
  Object.assign(boxPreferences, previous);
  return false;
}
