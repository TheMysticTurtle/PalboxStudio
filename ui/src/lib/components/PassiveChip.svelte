<script lang="ts">
  import { resolvePassive } from "$lib/data/refdata.svelte";
  import { ratingTone } from "$lib/data/constants";

  // Takes a passive **code**; name / rating / description come from passives.json.
  let { code, onselect }: { code: string; onselect?: () => void } = $props();

  let p = $derived(resolvePassive(code));
  let rating = $derived(p?.rating ?? 0);
  let chevrons = $derived(
    (rating >= 0 ? "▲" : "▼").repeat(Math.min(3, Math.max(1, Math.abs(rating) || 1))),
  );
</script>

<button type="button" class="chip" class:unknown={!p} class:editable={!!onselect} style="--c:{ratingTone(rating)}" onclick={onselect}>
  <div class="top">
    {#if p}<span class="rank">{chevrons}</span>{/if}
    <span class="name">{p?.name ?? code}</span>
  </div>
  <div class="eff">{p?.description ?? "unknown passive"}</div>
</button>

<style>
  .chip {
    display: block;
    width: 100%;
    color: inherit;
    text-align: left;
    padding: 11px 13px;
    border-radius: 10px;
    border-top: 0;
    border-right: 0;
    border-bottom: 0;
    border-left: 3px solid var(--c);
    background: linear-gradient(90deg, color-mix(in srgb, var(--c) 14%, transparent), rgba(255, 255, 255, 0.02));
  }
  .chip.editable { cursor: pointer; }
  .chip.editable:hover { filter: brightness(1.12); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--c) 28%, transparent); }
  .chip.unknown { border-left-color: var(--text-muted); background: rgba(255, 255, 255, 0.03); }
  .top { display: flex; align-items: center; gap: 8px; }
  .rank { color: color-mix(in srgb, var(--c) 50%, #ffffff); font-size: var(--type-caption); letter-spacing: -1px; }
  .name { font-family: var(--font-cond); font-weight: 600; font-size: 17px; color: #eaf2ee; }
  .eff { font-size: var(--type-body); line-height: 1.35; color: color-mix(in srgb, var(--c) 38%, #c9d3cf); margin-top: 4px; }
  .chip.unknown .eff { color: var(--text-muted); }
</style>
