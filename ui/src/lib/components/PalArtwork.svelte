<script lang="ts">
  import { onPalIconError } from "$lib/data/icons";

  let {
    src,
    alt = "",
    shape = "rounded",
    zoom = 1.04,
    lazy = true,
  }: {
    src: string;
    alt?: string;
    shape?: "circle" | "rounded";
    /** One shared, subtle crop for inconsistent transparent padding in source icons. */
    zoom?: number;
    lazy?: boolean;
  } = $props();
</script>

<span class="artwork" class:circle={shape === "circle"} style="--zoom:{zoom}">
  <img
    {src}
    {alt}
    loading={lazy ? "lazy" : "eager"}
    decoding="async"
    onerror={onPalIconError}
  />
</span>

<style>
  .artwork {
    width: 100%;
    height: 100%;
    display: block;
    overflow: hidden;
    border-radius: 18px;
  }
  .artwork.circle { border-radius: 50%; }
  img {
    width: 100%;
    height: 100%;
    display: block;
    object-fit: cover;
    transform: scale(var(--zoom));
    transform-origin: center;
  }
</style>
