<script lang="ts">
  import type { PalCardPresentation } from "$lib/data/palPresentation";
  import { onPalIconError, variantIcon } from "$lib/data/icons";

  let {
    card,
    size = 72,
  }: {
    card: PalCardPresentation;
    size?: number;
  } = $props();
</script>

<span
  class="portrait"
  style="--size:{size}px; --primary:{card.primaryColor}; --secondary:{card.secondaryColor}"
>
  <span class="portrait-inner">
    <img class="pal-art" src={card.portrait} alt="" loading="lazy" decoding="async" onerror={onPalIconError} />
  </span>
  {#if card.alpha}
    <img class="variant alpha" src={variantIcon("alpha")} alt="Alpha" title="Alpha" />
  {/if}
  {#if card.lucky}
    <img class="variant lucky" src={variantIcon("lucky")} alt="Lucky" title="Lucky" />
  {/if}
</span>

<style>
  .portrait {
    position: relative;
    width: var(--size);
    height: var(--size);
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 50%;
    padding: 2px;
    background: conic-gradient(from 180deg, var(--primary) 0 50%, var(--secondary) 50% 100%);
    box-shadow:
      0 0 15px color-mix(in srgb, var(--primary) 30%, transparent),
      0 0 18px color-mix(in srgb, var(--secondary) 22%, transparent);
  }
  .portrait-inner {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: inherit;
    background:
      radial-gradient(circle at 50% 32%, rgba(255, 255, 255, 0.13), transparent 52%),
      linear-gradient(145deg, color-mix(in srgb, var(--primary) 15%, #111722), color-mix(in srgb, var(--secondary) 12%, #0c1119));
    box-shadow: inset 0 0 18px rgba(0, 0, 0, 0.48);
  }
  .pal-art { width: 94%; height: 94%; object-fit: contain; }
  .variant {
    position: absolute;
    top: -7%;
    width: 34%;
    height: 34%;
    object-fit: contain;
    filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.82));
  }
  .variant.alpha { left: -7%; }
  .variant.lucky { right: -7%; width: 31%; height: 31%; }
</style>
