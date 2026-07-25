<script lang="ts">
  import { resolvePassive } from "$lib/data/refdata.svelte";
  import { ratingTone } from "$lib/data/constants";

  // Takes a passive **code**; name / rating / description come from passives.json.
  let { code }: { code: string } = $props();

  let p = $derived(resolvePassive(code));
  let rating = $derived(p?.rating ?? 0);
  let chevrons = $derived(
    (rating >= 0 ? "▲" : "▼").repeat(Math.min(3, Math.max(1, Math.abs(rating) || 1))),
  );
</script>

<div class="chip" class:unknown={!p} style="--c:{ratingTone(rating)}">
  <div class="top">
    {#if p}<span class="rank">{chevrons}</span>{/if}
    <span class="name">{p?.name ?? code}</span>
  </div>
  <div class="eff">{p?.description ?? "unknown passive"}</div>
</div>

<style>
  .chip {
    padding: 11px 13px;
    border-radius: 10px;
    border-left: 3px solid var(--c);
    background: linear-gradient(90deg, color-mix(in srgb, var(--c) 14%, transparent), rgba(255, 255, 255, 0.02));
  }
  .chip.unknown { border-left-color: var(--text-muted); background: rgba(255, 255, 255, 0.03); }
  .top { display: flex; align-items: center; gap: 8px; }
  .rank { color: color-mix(in srgb, var(--c) 50%, #ffffff); font-size: 11px; letter-spacing: -1px; }
  .name { font-family: var(--font-cond); font-weight: 600; font-size: 15px; color: #eaf2ee; }
  .eff { font-size: 12px; color: color-mix(in srgb, var(--c) 38%, #c9d3cf); margin-top: 3px; }
  .chip.unknown .eff { color: var(--text-muted); }
</style>
