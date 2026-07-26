/** Group filters use intersection semantics, matching the shared species filter:
 * a Pal must belong to every selected group. */
export function matchesAllGroups(
  membership: Iterable<number>,
  selected: Iterable<number>,
): boolean {
  const assigned = membership instanceof Set ? membership : new Set(membership);
  for (const groupId of selected) {
    if (!assigned.has(groupId)) return false;
  }
  return true;
}
