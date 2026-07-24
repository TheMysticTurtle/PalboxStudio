<script lang="ts">
  import type { WorkSuit } from "$lib/data/types";
  import { LIMITS, workSuitCode } from "$lib/data/constants";

  // `suit` is a reactive object from the parent's $state pal, so mutating
  // suit.level here updates the model directly (deep reactivity).
  let { suit }: { suit: WorkSuit } = $props();

  const dec = () => (suit.level = Math.max(LIMITS.workSuitMin, suit.level - 1));
  const inc = () => (suit.level = Math.min(LIMITS.workSuitMax, suit.level + 1));
  let active = $derived(suit.level > 0);
</script>

<div class="row" class:active>
  <span class="ic">{workSuitCode(suit.name)}</span>
  <span class="name">{suit.name}</span>
  <span class="lvl">{suit.level}</span>
  <span class="step">
    <button onclick={inc} disabled={suit.level >= LIMITS.workSuitMax} aria-label="Increase {suit.name}">▲</button>
    <button onclick={dec} disabled={suit.level <= LIMITS.workSuitMin} aria-label="Decrease {suit.name}">▼</button>
  </span>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 8px;
    border-radius: 8px;
    opacity: 0.45;
  }
  .row.active {
    opacity: 1;
    background: color-mix(in srgb, var(--accent-amber) 8%, transparent);
  }
  .ic {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-2);
    background: rgba(255, 255, 255, 0.06);
  }
  .row.active .ic {
    color: #1a1206;
    background: var(--accent-amber);
  }
  .name {
    flex: 1;
    font-size: 12.5px;
    color: var(--text-1);
  }
  .lvl {
    font-variant-numeric: tabular-nums;
    font-weight: 700;
    color: var(--text-1);
    min-width: 18px;
    text-align: right;
  }
  .step {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .step button {
    width: 20px;
    height: 13px;
    line-height: 1;
    font-size: 8px;
    border: 1px solid var(--hairline);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-2);
    cursor: pointer;
    border-radius: 4px;
  }
  .step button:hover:not(:disabled) {
    color: var(--text-1);
    border-color: color-mix(in srgb, var(--accent-amber) 50%, transparent);
  }
  .step button:disabled {
    opacity: 0.3;
    cursor: default;
  }
</style>
