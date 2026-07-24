<script lang="ts">
  import type { WorkSuit } from "$lib/data/types";
  import { LIMITS, workSuitCode } from "$lib/data/constants";

  // `suit` is a reactive object from the parent's $state pal.
  let { suit }: { suit: WorkSuit } = $props();
  const dec = () => (suit.level = Math.max(LIMITS.workSuitMin, suit.level - 1));
  const inc = () => (suit.level = Math.min(LIMITS.workSuitMax, suit.level + 1));
  let active = $derived(suit.level > 0);
</script>

<div class="row" class:active>
  <div class="ic">{workSuitCode(suit.name)}</div>
  <span class="name">{suit.name}</span>
  <span class="lvl">{suit.level}</span>
  <div class="step">
    <button onclick={inc} disabled={suit.level >= LIMITS.workSuitMax} aria-label="Increase {suit.name}">▲</button>
    <button onclick={dec} disabled={suit.level <= LIMITS.workSuitMin} aria-label="Decrease {suit.name}">▼</button>
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 6px 8px;
    border-radius: 10px;
    border: 1px solid transparent;
  }
  .row.active {
    background: rgba(245, 166, 35, 0.1);
    border-color: rgba(245, 166, 35, 0.28);
  }
  .ic {
    width: 34px;
    height: 34px;
    flex: none;
    border-radius: 9px;
    display: grid;
    place-items: center;
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 13px;
    color: #6e7a86;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .row.active .ic {
    color: #2a2306;
    background: var(--accent-amber);
    border-color: transparent;
  }
  .name {
    flex: 1;
    font-size: 13.5px;
    color: #8892a0;
  }
  .row.active .name {
    color: #ede3d6;
  }
  .lvl {
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 18px;
    min-width: 22px;
    text-align: center;
    color: #6e7a86;
  }
  .row.active .lvl {
    color: #f2f4f6;
  }
  .step {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .step button {
    width: 26px;
    height: 16px;
    display: grid;
    place-items: center;
    border-radius: 5px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: #9fd8e6;
    cursor: pointer;
    font-size: 8px;
  }
  .step button:hover:not(:disabled) {
    background: rgba(63, 199, 224, 0.2);
  }
  .step button:disabled {
    opacity: 0.3;
    cursor: default;
  }
</style>
