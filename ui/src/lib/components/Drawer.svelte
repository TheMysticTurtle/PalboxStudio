<script lang="ts">
  import type { Snippet } from "svelte";

  // Generic retractable side drawer, styled to the prototype. `tone` picks the
  // purple "box" or cyan "advanced" identity. Overlays the card; both sides can
  // be open at once; the edge tab rides the panel's inner edge on open.
  let {
    side = "left",
    tone = "box",
    label,
    tabLabel,
    width = 440,
    open = $bindable(false),
    children,
  }: {
    side?: "left" | "right";
    tone?: "box" | "advanced";
    label: string;
    tabLabel: string;
    width?: number;
    open?: boolean;
    children?: Snippet;
  } = $props();

  const toggle = () => (open = !open);
  let arrow = $derived(side === "left" ? (open ? "‹" : "›") : open ? "›" : "‹");
  let closeGlyph = $derived(side === "left" ? "◂" : "▸");
</script>

<div class="drawer {side} tone-{tone}" class:open style="--w:{width}px">
  <div class="panel">
    <div class="panel-inner">
      <header>
        <span class="hdiamond"></span>
        <h2 class="htitle">{label}</h2>
        <button class="close" onclick={toggle} aria-label="Close {label}">{closeGlyph}</button>
      </header>
      <div class="body">{@render children?.()}</div>
    </div>
  </div>

  <button class="edge-tab" onclick={toggle} aria-expanded={open} aria-label={(open ? "Collapse " : "Expand ") + tabLabel}>
    <span class="arw">{arrow}</span>
    <span class="lbl">{tabLabel}</span>
  </button>
</div>

<style>
  .tone-box {
    --grad: linear-gradient(160deg, rgba(176, 96, 224, 0.6), rgba(176, 96, 224, 0.15));
    --surf: linear-gradient(155deg, rgba(24, 17, 32, 0.96), rgba(15, 13, 22, 0.97));
    --accent: #b060e0;
    --headborder: rgba(176, 96, 224, 0.22);
    --tabgrad: linear-gradient(180deg, rgba(155, 95, 224, 0.3), rgba(155, 95, 224, 0.14));
    --tabborder: rgba(176, 96, 224, 0.5);
    --tabglow: rgba(176, 96, 224, 0.2);
    --tabfg: #e7daf4;
    --arrowfg: #d6bef2;
  }
  .tone-advanced {
    --grad: linear-gradient(200deg, rgba(63, 199, 224, 0.5), rgba(63, 199, 224, 0.12));
    --surf: linear-gradient(155deg, rgba(20, 29, 38, 0.96), rgba(13, 18, 25, 0.97));
    --accent: #3fc7e0;
    --headborder: rgba(63, 199, 224, 0.2);
    --tabgrad: linear-gradient(180deg, rgba(63, 199, 224, 0.26), rgba(245, 166, 35, 0.14));
    --tabborder: rgba(63, 199, 224, 0.5);
    --tabglow: rgba(63, 199, 224, 0.18);
    --tabfg: #cbeaf2;
    --arrowfg: #9fd8e6;
  }

  .drawer {
    position: fixed;
    top: var(--topbar-h);
    bottom: 0;
    z-index: 50;
    pointer-events: none;
  }
  .drawer.left { left: 0; }
  .drawer.right { right: 0; }

  .panel {
    position: absolute;
    top: 0;
    bottom: 0;
    width: var(--w);
    max-width: 90vw;
    pointer-events: auto;
    padding: 1px;
    background: var(--grad);
    transition: transform var(--drawer-dur) var(--drawer-ease);
  }
  .drawer.left .panel {
    left: 0;
    border-radius: 0 16px 16px 0;
    transform: translateX(-101%);
    box-shadow: 24px 0 70px rgba(0, 0, 0, 0.6), 0 0 50px var(--tabglow);
  }
  .drawer.right .panel {
    right: 0;
    border-radius: 16px 0 0 16px;
    transform: translateX(101%);
    box-shadow: -24px 0 70px rgba(0, 0, 0, 0.6), 0 0 44px var(--tabglow);
  }
  .drawer.open .panel { transform: translateX(0); }

  .panel-inner {
    height: 100%;
    background: var(--surf);
    backdrop-filter: blur(20px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .drawer.left .panel-inner { border-radius: 0 15px 15px 0; }
  .drawer.right .panel-inner { border-radius: 15px 0 0 15px; }

  header {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 16px 18px 13px;
    border-bottom: 1px solid var(--headborder);
  }
  .hdiamond {
    width: 10px;
    height: 10px;
    transform: rotate(45deg);
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .htitle {
    margin: 0;
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 18px;
    letter-spacing: 0.14em;
    color: var(--tabfg);
  }
  .close {
    margin-left: auto;
    width: 34px;
    height: 34px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.05);
    color: var(--arrowfg);
    cursor: pointer;
    font-size: 15px;
  }
  .close:hover { background: color-mix(in srgb, var(--accent) 20%, transparent); }
  .body { flex: 1; overflow: auto; padding: 16px; }

  .edge-tab {
    position: absolute;
    top: 50%;
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 24px 11px;
    cursor: pointer;
    background: var(--tabgrad);
    border: 1px solid var(--tabborder);
    box-shadow: 0 0 22px var(--tabglow);
    transition: transform var(--drawer-dur) var(--drawer-ease), background 0.15s;
  }
  .edge-tab .lbl {
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 15px;
    letter-spacing: 0.18em;
    color: var(--tabfg);
    writing-mode: vertical-rl;
  }
  .edge-tab .arw { font-size: 16px; color: var(--arrowfg); }
  .edge-tab:hover { filter: brightness(1.18); }

  .drawer.left .edge-tab { left: 0; border-left: 0; border-radius: 0 13px 13px 0; transform: translateY(-50%); }
  .drawer.left.open .edge-tab { transform: translate(var(--w), -50%); }
  .drawer.right .edge-tab { right: 0; border-right: 0; border-radius: 13px 0 0 13px; transform: translateY(-50%); }
  .drawer.right.open .edge-tab { transform: translate(calc(-1 * var(--w)), -50%); }
</style>
