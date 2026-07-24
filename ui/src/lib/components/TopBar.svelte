<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // Confirm the UI <-> core bridge (shows nothing if run outside the app).
  let coreVersion = $state<string | null>(null);
  $effect(() => {
    invoke<string>("core_version")
      .then((v) => (coreVersion = v))
      .catch(() => {});
  });
</script>

<header class="topbar">
  <div class="brand">
    <img class="mark" src="/logo.png" alt="" />
    <span class="word">PALBOX&nbsp;STUDIO</span>
    <span class="chip identity">GLOBAL PALBOX</span>
  </div>
  <div class="right">
    <span class="chip safe"><span class="dot"></span> Editing a copy · backed up</span>
    {#if coreVersion}<span class="core">core v{coreVersion}</span>{/if}
  </div>
</header>

<style>
  .topbar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: var(--topbar-h);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 60;
    background: linear-gradient(180deg, rgba(27, 39, 51, 0.92), rgba(18, 24, 32, 0.92));
    border-bottom: 1px solid var(--hairline);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .mark {
    width: 26px;
    height: 26px;
    border-radius: 7px;
    box-shadow: 0 0 14px rgba(176, 96, 224, 0.5);
  }
  .word {
    font-weight: 800;
    letter-spacing: 0.14em;
    font-size: 15px;
  }
  .chip {
    font-size: 11px;
    letter-spacing: 0.12em;
    padding: 4px 10px;
    border-radius: var(--radius-pill);
  }
  .identity {
    color: #d9b6f2;
    border: 1px solid rgba(176, 96, 224, 0.5);
    background: rgba(176, 96, 224, 0.12);
  }
  .right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .safe {
    color: #a9e6b4;
    border: 1px solid rgba(95, 209, 106, 0.4);
    background: rgba(95, 209, 106, 0.1);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--stat-hp);
    box-shadow: 0 0 6px var(--stat-hp);
  }
  .core {
    color: var(--accent-cyan-text);
    font-size: 11px;
    letter-spacing: 0.06em;
  }
</style>
