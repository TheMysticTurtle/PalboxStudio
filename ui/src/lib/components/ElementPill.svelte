<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { ELEMENT_COLOR } from "$lib/data/constants";
  let { element }: { element: ElementName } = $props();
</script>

<span class="pill" style="--c:{ELEMENT_COLOR[element]}">
  <span class="diamond"></span>{element}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 8px;
    border-radius: var(--radius-pill);
    font-size: 12px;
    color: var(--text-1);
    border: 1px solid color-mix(in srgb, var(--c) 55%, transparent);
    background: color-mix(in srgb, var(--c) 16%, transparent);
  }
  .diamond {
    width: 9px;
    height: 9px;
    transform: rotate(45deg);
    border-radius: 2px;
    background: var(--c);
    box-shadow: 0 0 8px var(--c);
  }
</style>
