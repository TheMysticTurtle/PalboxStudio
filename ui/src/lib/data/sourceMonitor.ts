export type MonitoredSourceState = "unchanged" | "changed" | "unavailable";
export type BoxConflictKind = "" | "external" | "post-save";

export function classifySourceConflict(
  sourceState: MonitoredSourceState,
  lastSavedAt: number,
  now: number,
  postSaveWindowMs = 30_000,
): BoxConflictKind {
  if (sourceState === "unchanged") return "";
  const sinceSave = now - lastSavedAt;
  return lastSavedAt > 0 && sinceSave >= 0 && sinceSave <= postSaveWindowMs
    ? "post-save"
    : "external";
}
