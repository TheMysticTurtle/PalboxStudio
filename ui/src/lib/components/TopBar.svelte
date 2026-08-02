<script lang="ts">
  import { box, saveToFile } from "$lib/stores/box.svelte";
  import { APP_LOGO_ART } from "$lib/data/icons";

  // Frameless custom title bar (window decorations are off in tauri.conf.json).
  // The bar is the drag handle; the controls drive the OS window.
  async function win(action: "min" | "max" | "close") {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const w = getCurrentWindow();
      if (action === "min") await w.minimize();
      else if (action === "max") await w.toggleMaximize();
      else await w.close();
    } catch {
      /* running outside the app (browser preview) — no-op */
    }
  }

  async function revealBackup() {
    if (!box.lastBackupPath) return;
    try {
      const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
      await revealItemInDir(box.lastBackupPath);
    } catch (error) {
      box.error = `Could not open the backup folder: ${String(error)}`;
    }
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <img class="mark" src={APP_LOGO_ART} alt="" />
    <span class="word">PALBOX&nbsp;STUDIO</span>
  </div>

  <div class="identity">
    <span class="diamond"></span>
    <span class="idtext">GLOBAL&nbsp;PALBOX</span>
  </div>

  <div class="spacer" data-tauri-drag-region></div>

  {#if box.open}
    {#if box.saveMsg}<span class="savemsg">{box.saveMsg}</span>{/if}
    {#if box.lastBackupPath}
      <button
        class="backupbtn"
        onclick={revealBackup}
        title={box.lastBackupPath}
      >Open backup</button>
    {/if}
    <button
      class="savebtn"
      class:blocked={Boolean(box.conflict)}
      onclick={saveToFile}
      disabled={Boolean(box.conflict) || box.loading}
      title={box.conflict
        ? "Reload the externally changed Global Palbox before saving"
        : "Backup the original, then write the edited box"}
    >💾 Save Box</button>
  {/if}

  {#if box.open}
    <div class="safe" class:conflict={Boolean(box.conflict)}>
      <svg width="14" height="15" viewBox="0 0 24 26" fill="none" aria-hidden="true">
        <path d="M12 1 22 5v9c0 6.5-4.3 10-10 11C6.3 24 2 20.5 2 14V5l10-4Z" fill="rgba(95,209,106,.18)" stroke="#5FD16A" stroke-width="1.6" />
        <path d="m7.5 13 3 3 6-6.5" stroke="#5FD16A" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <span>{box.conflict ? "Source changed" : box.dirty ? "Unsaved edits" : "Working copy"}</span>
    </div>
  {/if}

  <div class="wincontrols">
    <button class="wc" onclick={() => win("min")} aria-label="Minimize">—</button>
    <button class="wc" onclick={() => win("max")} aria-label="Maximize">▢</button>
    <button class="wc close" onclick={() => win("close")} aria-label="Close">✕</button>
  </div>
</header>

<style>
  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: var(--topbar-h);
    z-index: 60;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 8px 0 16px;
    background: linear-gradient(180deg, rgba(27, 39, 51, 0.92), rgba(18, 24, 32, 0.92));
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .mark {
    width: 22px;
    height: 22px;
    border-radius: 5px;
    box-shadow: 0 0 12px rgba(176, 96, 224, 0.55);
  }
  .word {
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 17px;
    letter-spacing: 0.14em;
    color: var(--text-1);
  }
  .identity {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 5px;
    background: rgba(176, 96, 224, 0.14);
    border: 1px solid rgba(176, 96, 224, 0.42);
    box-shadow: inset 0 0 14px rgba(176, 96, 224, 0.18);
  }
  .diamond {
    width: 8px;
    height: 8px;
    transform: rotate(45deg);
    background: var(--accent-purple);
    box-shadow: 0 0 8px rgba(176, 96, 224, 0.8);
  }
  .idtext {
    font-family: var(--font-head);
    font-weight: 600;
    font-size: var(--type-label);
    letter-spacing: 0.16em;
    color: #d9b8f0;
  }
  .spacer {
    flex: 1;
  }
  .backupbtn {
    min-height: 30px;
    padding: 5px 9px;
    border-radius: 7px;
    cursor: pointer;
    color: #b9d8c0;
    background: rgba(95, 209, 106, .08);
    border: 1px solid rgba(95, 209, 106, .28);
    font: 600 var(--type-label) var(--font-head);
  }
  .backupbtn:hover {
    color: #dcf8e1;
    background: rgba(95, 209, 106, .15);
    border-color: rgba(95, 209, 106, .5);
  }
  .safe {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 5px 12px;
    border-radius: 6px;
    background: rgba(95, 209, 106, 0.1);
    border: 1px solid rgba(95, 209, 106, 0.3);
    font-size: 12.5px;
    color: #9ad6a0;
    letter-spacing: 0.02em;
  }
  .safe.conflict {
    color: #e9a1a1;
    background: rgba(224, 90, 90, 0.1);
    border-color: rgba(224, 90, 90, 0.38);
  }
  .safe.conflict svg { filter: hue-rotate(115deg) saturate(1.5); }
  .savebtn {
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid rgba(63, 199, 224, 0.45);
    background: rgba(63, 199, 224, 0.14);
    color: #eafbff;
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 600;
  }
  .savebtn:hover { background: rgba(63, 199, 224, 0.24); }
  .savebtn:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  .savebtn.blocked {
    color: #e9a1a1;
    border-color: rgba(224, 90, 90, 0.4);
    background: rgba(224, 90, 90, 0.1);
  }
  .savemsg { font-size: 11.5px; color: #9ad6a0; }
  .wincontrols {
    display: flex;
    gap: 2px;
    margin-left: 8px;
  }
  .wc {
    width: 34px;
    height: 30px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: #9aa6b2;
    cursor: pointer;
    font-size: 13px;
  }
  .wc:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-1);
  }
  .wc.close:hover {
    background: rgba(224, 90, 90, 0.85);
    color: #fff;
  }
</style>
