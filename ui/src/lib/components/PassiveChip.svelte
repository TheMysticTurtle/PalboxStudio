<script lang="ts">
  import type { Passive } from "$lib/data/types";
  import { ratingTone } from "$lib/data/constants";
  let { passive }: { passive: Passive } = $props();

  let chevrons = $derived(
    (passive.rating >= 0 ? "▲" : "▼").repeat(Math.min(3, Math.max(1, Math.abs(passive.rating)))),
  );
</script>

<div class="chip" style="--c:{ratingTone(passive.rating)}">
  <div class="top">
    <span class="rank">{chevrons}</span>
    <span class="name">{passive.name}</span>
  </div>
  <div class="eff">{passive.effects}</div>
</div>

<style>
  .chip {
    padding: 11px 13px;
    border-radius: 10px;
    border-left: 3px solid var(--c);
    background: linear-gradient(90deg, color-mix(in srgb, var(--c) 14%, transparent), rgba(255, 255, 255, 0.02));
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .rank {
    color: color-mix(in srgb, var(--c) 50%, #ffffff);
    font-size: 11px;
    letter-spacing: -1px;
  }
  .name {
    font-family: var(--font-cond);
    font-weight: 600;
    font-size: 15px;
    color: #eaf2ee;
  }
  .eff {
    font-size: 12px;
    color: color-mix(in srgb, var(--c) 38%, #c9d3cf);
    margin-top: 3px;
  }
</style>
