export interface BoxPreferencesValue {
  lastBoxPath: string;
  autoReopen: boolean;
  maxHp: boolean;
  maxSanity: boolean;
  maxFood: boolean;
  maxTrust: boolean;
}

export const LEGACY_STORAGE_KEY = "palboxStudio.boxPreferences.v1";

export const DEFAULT_BOX_PREFERENCES: BoxPreferencesValue = {
  lastBoxPath: "",
  autoReopen: false,
  maxHp: true,
  maxSanity: true,
  maxFood: true,
  maxTrust: false,
};

export function parseBoxPreferences(raw: string | null): BoxPreferencesValue {
  if (!raw) return { ...DEFAULT_BOX_PREFERENCES };
  try {
    const value = JSON.parse(raw) as Partial<BoxPreferencesValue>;
    const lastBoxPath =
      typeof value.lastBoxPath === "string" ? value.lastBoxPath : "";
    return {
      lastBoxPath,
      autoReopen: value.autoReopen === true && Boolean(lastBoxPath),
      maxHp: typeof value.maxHp === "boolean" ? value.maxHp : true,
      maxSanity: typeof value.maxSanity === "boolean" ? value.maxSanity : true,
      maxFood: typeof value.maxFood === "boolean" ? value.maxFood : true,
      maxTrust: typeof value.maxTrust === "boolean" ? value.maxTrust : false,
    };
  } catch {
    return { ...DEFAULT_BOX_PREFERENCES };
  }
}

export function shouldMigrateLegacyBoxPreferences(
  current: BoxPreferencesValue,
  legacy: BoxPreferencesValue,
): boolean {
  return !current.lastBoxPath && Boolean(legacy.lastBoxPath);
}

export function readLegacyBoxPreferences(
  storage: Pick<Storage, "getItem">,
): BoxPreferencesValue {
  return parseBoxPreferences(storage.getItem(LEGACY_STORAGE_KEY));
}

export function clearLegacyBoxPreferences(
  storage: Pick<Storage, "removeItem">,
): void {
  storage.removeItem(LEGACY_STORAGE_KEY);
}
