<script lang="ts">
  import type { Gender } from "$lib/data/types";

  let {
    gender,
    size = 18,
  }: {
    gender: Gender;
    size?: number;
  } = $props();
</script>

{#if gender === "Male"}
  <svg class="icon" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="9" cy="15" r="5"></circle>
    <path d="M12.5 11.5 20 4M15 4h5v5"></path>
  </svg>
{:else if gender === "Female"}
  <svg class="icon" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="12" cy="8" r="5"></circle>
    <path d="M12 13v8M8.5 17.5h7"></path>
  </svg>
{:else}
  <svg class="icon" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <path d="M6 12h12"></path>
  </svg>
{/if}

<style>
  .icon {
    display: block;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
