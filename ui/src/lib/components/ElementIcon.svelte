<script lang="ts">
  import type { ElementName } from "$lib/data/types";
  import { elementIcon } from "$lib/data/icons";

  let {
    element,
    size = 18,
    decorative = true,
    muted = false,
  }: {
    element: ElementName;
    size?: number;
    /** Set false when the surrounding UI does not already name the element. */
    decorative?: boolean;
    muted?: boolean;
  } = $props();
</script>

<img
  class="element-icon"
  class:muted
  src={elementIcon(element)}
  alt={decorative ? "" : `${element} element`}
  title={element}
  width={size}
  height={size}
  draggable="false"
/>

<style>
  .element-icon {
    display: block;
    flex: none;
    object-fit: contain;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.55));
  }
  .element-icon.muted {
    opacity: 0.48;
    filter: saturate(0.65) drop-shadow(0 1px 2px rgba(0, 0, 0, 0.4));
  }
</style>
