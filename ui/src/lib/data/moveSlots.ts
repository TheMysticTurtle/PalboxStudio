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
  /** Retained for API compatibility; always null now that drops swap in place. */
  displaced: string | null;
}

/**
 * Move a skill by swapping it with whatever occupies the drop slot.
 *
 * Dropping onto an occupied slot exchanges the two entries in place, so the rest
 * of the list never shifts; dropping onto an empty active slot or the end of a
 * list simply moves the skill there. Independent of the DOM and Svelte state, so
 * pointer, native drag, and keyboard input all share one rule set.
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

  const targetItems = targetList === "active" ? active : bench;
  const targetIndex = Number.isFinite(rawTargetIndex)
    ? Math.trunc(rawTargetIndex)
    : targetItems.length;
  const targetOccupied = targetIndex >= 0 && targetIndex < targetItems.length;

  // Dropped onto its own slot: nothing to do.
  if (source.list === targetList && targetOccupied && targetIndex === expectedIndex) {
    return { active, bench, moved: false, displaced: null };
  }

  if (targetOccupied) {
    // Swap the two entries in place, keeping both positions — nothing else shifts.
    const targetCode = targetItems[targetIndex];
    sourceItems[expectedIndex] = targetCode;
    targetItems[targetIndex] = source.code;
    return { active, bench, moved: true, displaced: null };
  }

  // Empty target slot (an unused active slot, or the end of a list): move it there.
  if (targetList === "active" && active.length >= activeLimit) {
    return { active, bench, moved: false, displaced: null };
  }
  sourceItems.splice(expectedIndex, 1);
  if (!targetItems.includes(source.code)) targetItems.push(source.code);
  return { active, bench, moved: true, displaced: null };
}
