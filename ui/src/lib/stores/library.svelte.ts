// Durable Palbox Studio metadata. This store mirrors only the writable user
// SQLite database (groups/memberships and passive presets); Pal save values
// continue to live exclusively in the open GlobalPalStorage session.
import {
  createGroup,
  deleteGroup,
  deletePassivePreset,
  listGroupMemberships,
  listGroups,
  listPassivePresets,
  renameGroup,
  savePassivePreset,
  setPalGroups,
  type PassivePreset,
  type UserGroup,
} from "$lib/data/engine";

export const library = $state({
  groups: [] as UserGroup[],
  memberships: {} as Record<string, number[]>,
  presets: [] as PassivePreset[],
  loaded: false,
  loading: false,
  error: "",
});

let started = false;

function sortGroups(groups: UserGroup[]) {
  return groups.sort((a, b) => a.name.localeCompare(b.name) || a.id - b.id);
}

function sortPresets(presets: PassivePreset[]) {
  return presets.sort((a, b) => a.name.localeCompare(b.name) || a.id - b.id);
}

export async function loadUserLibrary() {
  if (started || library.loading) return;
  started = true;
  library.loading = true;
  library.error = "";
  try {
    const [groups, memberships, presets] = await Promise.all([
      listGroups(),
      listGroupMemberships(),
      listPassivePresets(),
    ]);
    library.groups = sortGroups(groups);
    library.memberships = Object.fromEntries(
      memberships.map((membership) => [membership.instanceId, membership.groupIds]),
    );
    library.presets = sortPresets(presets);
    library.loaded = true;
  } catch (error) {
    // Plain browser previews have no Tauri transport. Leave an empty retryable
    // store; the desktop app surfaces real command errors in the controls.
    started = false;
    library.error = String(error);
  } finally {
    library.loading = false;
  }
}

export function groupIdsFor(instanceId: string): number[] {
  return library.memberships[instanceId] ?? [];
}

export function groupNamesFor(instanceId: string): string[] {
  const assigned = new Set(groupIdsFor(instanceId));
  return library.groups.filter((group) => assigned.has(group.id)).map((group) => group.name);
}

export async function createUserGroup(name: string): Promise<UserGroup> {
  library.error = "";
  try {
    const group = await createGroup(name);
    library.groups = sortGroups([...library.groups, group]);
    return group;
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}

export async function renameUserGroup(id: number, name: string): Promise<UserGroup> {
  library.error = "";
  try {
    const group = await renameGroup(id, name);
    library.groups = sortGroups(
      library.groups.map((value) => (value.id === id ? group : value)),
    );
    return group;
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}

export async function deleteUserGroup(id: number): Promise<void> {
  library.error = "";
  try {
    if (!(await deleteGroup(id))) return;
    library.groups = library.groups.filter((group) => group.id !== id);
    for (const [instanceId, groupIds] of Object.entries(library.memberships)) {
      if (groupIds.includes(id)) {
        library.memberships[instanceId] = groupIds.filter((groupId) => groupId !== id);
      }
    }
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}

export async function assignPalGroups(instanceId: string, groupIds: number[]): Promise<void> {
  library.error = "";
  try {
    library.memberships[instanceId] = await setPalGroups(instanceId, groupIds);
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}

export async function saveUserPreset(
  name: string,
  passiveCodes: string[],
  id: number | null = null,
): Promise<PassivePreset> {
  library.error = "";
  try {
    const preset = await savePassivePreset(name, passiveCodes, id);
    library.presets = sortPresets([
      ...library.presets.filter((value) => value.id !== preset.id),
      preset,
    ]);
    return preset;
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}

export async function deleteUserPreset(id: number): Promise<void> {
  library.error = "";
  try {
    if (await deletePassivePreset(id)) {
      library.presets = library.presets.filter((preset) => preset.id !== id);
    }
  } catch (error) {
    library.error = String(error);
    throw error;
  }
}
