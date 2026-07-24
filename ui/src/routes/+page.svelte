<script lang="ts">
  import Backdrop from "$lib/components/Backdrop.svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import Drawer from "$lib/components/Drawer.svelte";
  import PalCard from "$lib/components/PalCard.svelte";
  import { ui } from "$lib/stores/ui.svelte";
  import { samplePal } from "$lib/data/samplePal";

  // Deep-cloned so edits don't mutate the module constant; $state makes it
  // deeply reactive so the card's editors update the model live.
  let pal = $state(structuredClone(samplePal));
</script>

<Backdrop />
<TopBar />

<main class="stage">
  <section class="cardframe">
    <PalCard {pal} />
  </section>
</main>

<Drawer side="left" label="BOX" width={440} accent="var(--accent-purple)" bind:open={ui.leftOpen}>
  <div class="placeholder">
    <h3>Global Box</h3>
    <p>Tiles, search / filter / sort, groups &amp; tags, add · clone · delete.</p>
  </div>
</Drawer>

<Drawer side="right" label="IV / STATUE" width={420} accent="var(--accent-cyan)" bind:open={ui.rightOpen}>
  <div class="placeholder">
    <h3>Advanced</h3>
    <p>IV / breeding traits, Statue of Power (Pal Souls), Condensation.</p>
  </div>
</Drawer>

<style>
  .stage {
    position: fixed;
    top: var(--topbar-h);
    left: 0;
    right: 0;
    bottom: 0;
    padding: 14px var(--stage-pad-x);
    display: flex;
  }
  .cardframe {
    flex: 1;
    position: relative;
    border-radius: var(--radius-card);
    background: linear-gradient(155deg, var(--surface-card-1), var(--surface-card-2));
    backdrop-filter: blur(18px);
    box-shadow:
      0 0 60px rgba(176, 96, 224, 0.18),
      0 24px 60px rgba(0, 0, 0, 0.55),
      inset 0 0 0 1px rgba(176, 96, 224, 0.28);
    overflow: hidden;
    display: flex;
  }
  .placeholder {
    text-align: center;
    color: var(--text-2);
  }
  .placeholder h3 {
    margin: 0 0 6px;
    color: var(--text-1);
    letter-spacing: 0.14em;
    font-size: 14px;
  }
  .placeholder p {
    margin: 0;
    line-height: 1.5;
    font-size: 13px;
  }
</style>
