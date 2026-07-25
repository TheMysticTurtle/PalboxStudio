<script lang="ts">
  import Backdrop from "$lib/components/Backdrop.svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import Drawer from "$lib/components/Drawer.svelte";
  import PalCard from "$lib/components/PalCard.svelte";
  import AdvancedDrawer from "$lib/components/AdvancedDrawer.svelte";
  import GlobalBoxDrawer from "$lib/components/GlobalBoxDrawer.svelte";
  import BoxMatrix from "$lib/components/BoxMatrix.svelte";
  import { ui } from "$lib/stores/ui.svelte";
  import { box } from "$lib/stores/box.svelte";
  import { EMPTY_PAL } from "$lib/data/emptyPal";

  // Render a non-persistable zero display value until a real Pal is loaded from
  // the engine (box.pal), the card edits that — persisted on save.
</script>

<Backdrop />
<TopBar />

<main class="stage">
  <section class="cardframe">
    <PalCard pal={box.pal ?? EMPTY_PAL} empty={!box.pal} />
  </section>
</main>

<Drawer side="left" tone="box" label="GLOBAL BOX" tabLabel="BOX" width={440} bind:open={ui.leftOpen}>
  <GlobalBoxDrawer />
</Drawer>

<Drawer side="right" tone="advanced" label="ADVANCED" tabLabel="IV / STATUE" width={420} bind:open={ui.rightOpen}>
  <div class="advanced-state" inert={!box.pal}>
    <AdvancedDrawer pal={box.pal ?? EMPTY_PAL} />
  </div>
</Drawer>

{#if ui.boxExpanded}
  <BoxMatrix />
{/if}

<style>
  .stage {
    position: fixed;
    top: var(--topbar-h);
    left: 0;
    right: 0;
    bottom: 0;
    padding: 14px var(--stage-pad-x);
  }
  .cardframe {
    width: 100%;
    height: 100%;
    border-radius: 16px;
    padding: 1px;
    background: linear-gradient(150deg, rgba(176, 96, 224, 0.55), rgba(63, 199, 224, 0.18) 40%, rgba(176, 96, 224, 0.3));
    box-shadow: 0 0 60px rgba(176, 96, 224, 0.2), 0 24px 60px rgba(0, 0, 0, 0.55);
  }
  .advanced-state { min-height: 100%; }
</style>
