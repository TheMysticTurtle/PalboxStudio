<script lang="ts">
  import type { Snippet } from "svelte";
  // Accent-bar section heading, like the in-game panels.
  let {
    title,
    accent = "var(--accent-cyan)",
    color = "#9FD8E6",
    right,
  }: { title: string; accent?: string; color?: string; right?: Snippet } = $props();
</script>

<div class="sh">
  <span class="bar" style="--a:{accent}"></span>
  <h3 style="--c:{color}">{title}</h3>
  {#if right}<span class="right">{@render right()}</span>{/if}
</div>

<style>
  .sh {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 11px;
  }
  .bar {
    width: 3px;
    height: 15px;
    border-radius: 2px;
    background: var(--a);
    box-shadow: 0 0 8px color-mix(in srgb, var(--a) 60%, transparent);
  }
  h3 {
    margin: 0;
    font-family: var(--font-head);
    font-weight: 600;
    font-size: 14px;
    letter-spacing: 0.16em;
    color: var(--c);
  }
  .right {
    margin-left: auto;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
