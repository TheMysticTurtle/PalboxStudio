<script lang="ts">
  let {
    label,
    value,
    max = undefined,
    color = "var(--accent-cyan)",
    boosted = false,
    showBar = true,
  }: {
    label: string;
    value: number;
    max?: number;
    color?: string;
    boosted?: boolean;
    showBar?: boolean;
  } = $props();

  let pct = $derived(max && max > 0 ? Math.min(100, (value / max) * 100) : null);
</script>

<div class="stat">
  <span class="label">{label}</span>
  {#if showBar && pct !== null}
    <div class="track"><div class="fill" style="width:{pct}%; --c:{color}"></div></div>
  {:else}
    <span class="spacer"></span>
  {/if}
  <span class="val">
    {value.toLocaleString()}{#if max}<span class="max">/{max.toLocaleString()}</span>{/if}
    {#if boosted}<span class="up" title="Boosted">▲</span>{/if}
  </span>
</div>

<style>
  .stat {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 0;
    font-size: 13px;
  }
  .label {
    color: var(--text-2);
    min-width: 92px;
  }
  .track {
    flex: 1;
    height: 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.07);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 6px;
    background: var(--c);
    box-shadow: 0 0 10px color-mix(in srgb, var(--c) 60%, transparent);
  }
  .spacer {
    flex: 1;
  }
  .val {
    color: var(--text-1);
    font-variant-numeric: tabular-nums;
  }
  .max {
    color: var(--text-muted);
  }
  .up {
    color: var(--stat-hp);
    margin-left: 4px;
    font-size: 11px;
  }
</style>
