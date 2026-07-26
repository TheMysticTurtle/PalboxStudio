<script lang="ts">
  import type { BoxPal } from "$lib/data/types";
  import CompactPalTile from "./CompactPalTile.svelte";
  import ExpandedPalCard from "./ExpandedPalCard.svelte";

  let {
    pal,
    size = "sm",
    selected = false,
    onselect,
  }: {
    pal: BoxPal;
    size?: "sm" | "lg";
    selected?: boolean;
    onselect?: (slot: number) => void;
  } = $props();
</script>

{#if size === "lg"}
  <ExpandedPalCard {pal} {selected} {onselect} />
{:else}
  <CompactPalTile {pal} {selected} {onselect} />
{/if}
