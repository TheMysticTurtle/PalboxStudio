<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { ELEMENT_COLOR } from "$lib/data/constants";
  import ElementIcon from "./ElementIcon.svelte";
  let { element }: { element: ElementName } = $props();
</script>

<span class="pill" style="--c:{ELEMENT_COLOR[element]}">
  <ElementIcon {element} size={18} />
  <span class="label">{element}</span>
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 5px 13px 5px 10px;
    border-radius: 20px;
    background: color-mix(in srgb, var(--c) 16%, transparent);
    border: 1px solid color-mix(in srgb, var(--c) 50%, transparent);
  }
  .label {
    font-family: var(--font-cond);
    font-weight: 600;
    font-size: 14px;
    letter-spacing: 0.04em;
    color: color-mix(in srgb, var(--c) 45%, #ffffff);
  }
</style>
