<script lang="ts">
  import type { WorkSuit } from "$lib/data/types";
  import { LIMITS } from "$lib/data/constants";
  import WorkIcon from "./WorkIcon.svelte";

  // `suit` is a reactive object from the parent's $state pal.
  let { suit }: { suit: WorkSuit } = $props();
  const dec = () => (suit.level = Math.max(LIMITS.workSuitMin, suit.level - 1));
  const inc = () => (suit.level = Math.min(LIMITS.workSuitMax, suit.level + 1));
  const setLevel = (input: HTMLInputElement) => {
    const requested = input.valueAsNumber;
    suit.level = Number.isFinite(requested)
      ? Math.max(LIMITS.workSuitMin, Math.min(LIMITS.workSuitMax, Math.round(requested)))
      : suit.level;
    input.value = String(suit.level);
  };
  let active = $derived(suit.level > 0);
</script>

<div class="row" class:active>
  <span class="ic"><WorkIcon icon={suit.icon} name={suit.name} level={suit.level} size={34} /></span>
  <span class="name">{suit.name}</span>
  <input
    class="lvl"
    type="number"
    inputmode="numeric"
    min={LIMITS.workSuitMin}
    max={LIMITS.workSuitMax}
    value={suit.level}
    onchange={(event) => setLevel(event.currentTarget)}
    aria-label="{suit.name} level"
  />
  <div class="step">
    <button onclick={inc} disabled={suit.level >= LIMITS.workSuitMax} aria-label="Increase {suit.name}">▲</button>
    <button onclick={dec} disabled={suit.level <= LIMITS.workSuitMin} aria-label="Decrease {suit.name}">▼</button>
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 9px;
    border-radius: 10px;
    border: 1px solid transparent;
  }
  .row.active {
    background: rgba(245, 166, 35, 0.1);
    border-color: rgba(245, 166, 35, 0.28);
  }
  .ic { width: 34px; height: 34px; flex: none; opacity: 0.8; }
  .row.active .ic { opacity: 1; }
  .name { flex: 1; font-size: var(--type-control); color: #929da9; }
  .row.active .name { color: #ede3d6; }
  .lvl {
    width: 38px;
    padding: 2px 3px;
    border: 0;
    border-bottom: 1px dashed rgba(245, 166, 35, 0.28);
    outline: 0;
    background: transparent;
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 20px;
    min-width: 22px;
    text-align: center;
    color: #6e7a86;
  }
  .lvl:focus { border-bottom-color: rgba(143, 227, 242, 0.7); }
  .lvl::-webkit-inner-spin-button,
  .lvl::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
  .row.active .lvl { color: #f2f4f6; }
  .step { display: flex; flex-direction: column; gap: 2px; }
  .step button {
    width: 30px;
    height: 19px;
    display: grid;
    place-items: center;
    border-radius: 5px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: #9fd8e6;
    cursor: pointer;
    font-size: var(--type-micro);
  }
  .step button:hover:not(:disabled) { background: rgba(63, 199, 224, 0.2); }
  .step button:disabled { opacity: 0.3; cursor: default; }
</style>
