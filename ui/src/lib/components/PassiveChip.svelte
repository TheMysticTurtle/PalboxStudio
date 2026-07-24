<script lang="ts">
  import type { Passive } from "$lib/data/types";
  import { ratingTone } from "$lib/data/constants";
  let { passive }: { passive: Passive } = $props();

  // Chevron count reflects magnitude (1..3 shown), direction by sign.
  let chevrons = $derived(
    (passive.rating >= 0 ? "▲" : "▼").repeat(Math.min(3, Math.max(1, Math.abs(passive.rating)))),
  );
</script>

<div class="chip" style="--c:{ratingTone(passive.rating)}">
  <div class="top">
    <span class="name">{passive.name}</span>
    <span class="rank">{chevrons}</span>
  </div>
  <div class="eff">{passive.effects}</div>
</div>

<style>
  .chip {
    border-left: 3px solid var(--c);
    border-radius: 8px;
    padding: 8px 10px;
    background: color-mix(in srgb, var(--c) 10%, rgba(255, 255, 255, 0.02));
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .name {
    color: var(--text-1);
    font-weight: 600;
    font-size: 13px;
  }
  .rank {
    color: var(--c);
    font-size: 11px;
    letter-spacing: 1px;
  }
  .eff {
    margin-top: 3px;
    color: var(--text-2);
    font-size: 11px;
    line-height: 1.4;
  }
</style>
