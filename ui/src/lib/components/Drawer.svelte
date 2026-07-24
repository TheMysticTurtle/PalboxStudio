<script lang="ts">
  import type { Snippet } from "svelte";

  // Generic retractable side drawer. Overlays the stage (never pushes the card),
  // both sides can be open at once. The edge tab rides the panel's inner edge when
  // open. Reuse for the left "Global Box" and right "Advanced" drawers — adding a
  // new drawer is just another <Drawer> with its own content snippet.
  let {
    side = "left",
    label,
    width = 440,
    accent = "var(--accent-purple)",
    open = $bindable(false),
    children,
  }: {
    side?: "left" | "right";
    label: string;
    width?: number;
    accent?: string;
    open?: boolean;
    children?: Snippet;
  } = $props();

  const toggle = () => (open = !open);
</script>

<div
  class="drawer {side}"
  class:open
  style="--w:{width}px; --accent:{accent}"
>
  <aside class="panel" aria-hidden={!open}>
    <header class="panel-head">
      <span class="panel-title">{label}</span>
      <button class="close" onclick={toggle} aria-label="Close {label}">✕</button>
    </header>
    <div class="panel-body">
      {@render children?.()}
    </div>
  </aside>

  <button
    class="edge-tab"
    onclick={toggle}
    aria-expanded={open}
    aria-label={(open ? "Collapse " : "Expand ") + label}
  >
    <span class="tab-label">{label}</span>
    <span class="tab-arrow">
      {#if side === "left"}{open ? "‹" : "›"}{:else}{open ? "›" : "‹"}{/if}
    </span>
  </button>
</div>

<style>
  .drawer {
    position: fixed;
    top: var(--topbar-h);
    bottom: 0;
    z-index: 50;
    pointer-events: none; /* wrapper is transparent to clicks; children opt back in */
  }
  .drawer.left { left: 0; }
  .drawer.right { right: 0; }

  .panel {
    position: absolute;
    top: 0;
    bottom: 0;
    width: var(--w);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    background: linear-gradient(155deg, var(--surface-drawer-1), var(--surface-drawer-2));
    backdrop-filter: blur(20px);
    border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
    box-shadow: 0 0 44px color-mix(in srgb, var(--accent) 18%, transparent);
    transition: transform var(--drawer-dur) var(--drawer-ease);
  }
  .drawer.left .panel { left: 0; transform: translateX(-101%); border-radius: 0 14px 14px 0; }
  .drawer.right .panel { right: 0; transform: translateX(101%); border-radius: 14px 0 0 14px; }
  .drawer.open .panel { transform: translateX(0); }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--hairline);
  }
  .panel-title {
    font-weight: 700;
    letter-spacing: 0.16em;
    font-size: 13px;
    color: color-mix(in srgb, var(--accent) 55%, var(--text-1));
  }
  .close {
    background: none;
    border: none;
    color: var(--text-2);
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
  }
  .close:hover { color: var(--text-1); background: rgba(255, 255, 255, 0.06); }

  .panel-body {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .edge-tab {
    position: absolute;
    top: 50%;
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px 7px;
    cursor: pointer;
    color: var(--text-1);
    letter-spacing: 0.14em;
    font-size: 12px;
    font-weight: 600;
    background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 22%, #12151c), #12151c);
    border: 1px solid color-mix(in srgb, var(--accent) 45%, transparent);
    box-shadow: 0 0 14px color-mix(in srgb, var(--accent) 25%, transparent);
    transition: transform var(--drawer-dur) var(--drawer-ease), background 0.15s;
  }
  .edge-tab:hover { background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 34%, #12151c), #161a22); }
  .tab-label { writing-mode: vertical-rl; text-orientation: mixed; }
  .tab-arrow { font-size: 14px; color: color-mix(in srgb, var(--accent) 70%, var(--text-1)); }

  .drawer.left .edge-tab { left: 0; transform: translateY(-50%); border-radius: 0 10px 10px 0; border-left: none; }
  .drawer.left.open .edge-tab { transform: translate(var(--w), -50%); }
  .drawer.right .edge-tab { right: 0; transform: translateY(-50%); border-radius: 10px 0 0 10px; border-right: none; }
  .drawer.right.open .edge-tab { transform: translate(calc(-1 * var(--w)), -50%); }
</style>
