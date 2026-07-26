export type MoveList = "active" | "bench";

export interface MoveDrag {
  code: string;
  list: MoveList;
  index: number;
}

export interface MoveSlotState {
  active: string[];
  bench: string[];
}

export interface MoveSlotResult extends MoveSlotState {
  moved: boolean;
  /** Active move evicted from slot three when a bench move is inserted into a full set. */
  displaced: string | null;
}

/**
 * Move one skill between (or within) the active and bench lists.
 *
 * This is intentionally independent of the DOM and Svelte state. Native drag
 * events, pointer input, and keyboard controls can all use the same tested
 * ordering rules without duplicating mutation logic.
 */
export function moveSkill(
  state: MoveSlotState,
  source: MoveDrag,
  targetList: MoveList,
  rawTargetIndex: number,
  activeLimit = 3,
): MoveSlotResult {
  const active = [...state.active];
  const bench = [...state.bench];
  const sourceItems = source.list === "active" ? active : bench;
  const expectedIndex = sourceItems[source.index] === source.code
    ? source.index
    : sourceItems.indexOf(source.code);

  if (expectedIndex < 0) return { active, bench, moved: false, displaced: null };

  sourceItems.splice(expectedIndex, 1);
  let targetIndex = Number.isFinite(rawTargetIndex) ? Math.trunc(rawTargetIndex) : 0;
  if (source.list === targetList && expectedIndex < targetIndex) targetIndex -= 1;

  let displaced: string | null = null;
  if (targetList === "active") {
    const lastInsertIndex = source.list === "bench" && active.length >= activeLimit
      ? Math.max(0, activeLimit - 1)
      : active.length;
    targetIndex = Math.max(0, Math.min(lastInsertIndex, targetIndex));
    active.splice(targetIndex, 0, source.code);

    if (active.length > activeLimit) {
      displaced = active.pop() ?? null;
      if (displaced && displaced !== source.code && !bench.includes(displaced)) {
        bench.push(displaced);
      }
    }
  } else {
    targetIndex = Math.max(0, Math.min(bench.length, targetIndex));
    if (!bench.includes(source.code)) bench.splice(targetIndex, 0, source.code);
  }

  return { active, bench, moved: true, displaced };
}
