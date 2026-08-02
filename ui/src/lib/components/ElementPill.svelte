<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { elementColor } from "$lib/data/palPresentation";
  import ElementIcon from "./ElementIcon.svelte";
  let { element, large = false }: { element: ElementName; large?: boolean } = $props();
</script>

<span class="pill" class:large style="--c:{elementColor(element)}">
  <ElementIcon {element} size={large ? 23 : 18} />
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
  .pill.large { gap: 9px; padding: 7px 16px 7px 12px; }
  .pill.large .label { font-size: 16px; }
</style>
