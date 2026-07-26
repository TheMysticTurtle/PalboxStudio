<script lang="ts">
  import { workIcon } from "$lib/data/icons";

  let {
    icon,
    name,
    level,
    size = 20,
    decorative = true,
    muted = false,
  }: {
    icon: string;
    name: string;
    level?: number;
    size?: number;
    decorative?: boolean;
    muted?: boolean;
  } = $props();

  const active = $derived(level == null || level > 0);
  const src = $derived(workIcon(icon, active));
</script>

<img
  class="work-icon"
  class:muted
  {src}
  alt={decorative ? "" : name}
  title={level == null ? name : `${name} Lv.${level}`}
  width={size}
  height={size}
  draggable="false"
/>

<style>
  .work-icon {
    display: block;
    flex: none;
    object-fit: contain;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.55));
  }
  .work-icon.muted { opacity: 0.48; filter: grayscale(0.35); }
</style>
