export interface BoxPreferencesValue {
  lastBoxPath: string;
  autoReopen: boolean;
}

const STORAGE_KEY = "palboxStudio.boxPreferences.v1";

export const DEFAULT_BOX_PREFERENCES: BoxPreferencesValue = {
  lastBoxPath: "",
  autoReopen: false,
};

export function parseBoxPreferences(raw: string | null): BoxPreferencesValue {
  if (!raw) return { ...DEFAULT_BOX_PREFERENCES };
  try {
    const value = JSON.parse(raw) as Partial<BoxPreferencesValue>;
    return {
      lastBoxPath: typeof value.lastBoxPath === "string" ? value.lastBoxPath : "",
      autoReopen: value.autoReopen === true,
    };
  } catch {
    return { ...DEFAULT_BOX_PREFERENCES };
  }
}

export function serializeBoxPreferences(value: BoxPreferencesValue): string {
  return JSON.stringify({
    lastBoxPath: value.lastBoxPath,
    autoReopen: value.autoReopen,
  });
}

export function readStoredBoxPreferences(
  storage: Pick<Storage, "getItem">,
): BoxPreferencesValue {
  return parseBoxPreferences(storage.getItem(STORAGE_KEY));
}

export function writeStoredBoxPreferences(
  storage: Pick<Storage, "setItem">,
  value: BoxPreferencesValue,
): void {
  storage.setItem(STORAGE_KEY, serializeBoxPreferences(value));
}
