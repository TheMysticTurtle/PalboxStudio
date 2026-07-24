<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let coreVersion = $state<string | null>(null);
  let bridgeError = $state<string | null>(null);

  // Smoke test: ask the Rust core for its version so we can see the bridge is live.
  // In a plain browser (no Tauri runtime) this rejects — that's expected outside the app.
  $effect(() => {
    invoke<string>("core_version")
      .then((v) => (coreVersion = v))
      .catch((e) => (bridgeError = String(e)));
  });
</script>

<main>
  <div class="brand">
    <img class="mark" src="/logo.png" alt="" />
    <h1>PALBOX&nbsp;STUDIO</h1>
    <span class="chip">GLOBAL PALBOX</span>
  </div>
  <p class="tagline">Palworld 1.0 global Pal box editor — project scaffold</p>

  <div class="status">
    {#if coreVersion}
      <span class="ok">●</span> core engine connected
      <span class="ver">palbox-core v{coreVersion}</span>
    {:else if bridgeError}
      <span class="wait">○</span> run inside the app to connect the core
    {:else}
      <span class="wait">○</span> connecting to core…
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    background:
      radial-gradient(1200px 700px at 18% -8%, rgba(176, 96, 224, 0.2), transparent 60%),
      radial-gradient(1100px 800px at 92% 108%, rgba(63, 199, 224, 0.13), transparent 55%),
      linear-gradient(160deg, #0b0f15, #0a0d12 55%, #080a0e);
    color: #f2f4f6;
    font-family: system-ui, "Segoe UI", sans-serif;
  }
  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .mark {
    width: 42px;
    height: 42px;
    border-radius: 10px;
    box-shadow: 0 0 22px rgba(176, 96, 224, 0.55);
  }
  h1 {
    margin: 0;
    font-size: 30px;
    letter-spacing: 0.16em;
    font-weight: 800;
  }
  .chip {
    font-size: 11px;
    letter-spacing: 0.14em;
    padding: 4px 10px;
    border-radius: 20px;
    color: #d9b6f2;
    border: 1px solid rgba(176, 96, 224, 0.5);
    background: rgba(176, 96, 224, 0.12);
  }
  .tagline {
    margin: 0;
    color: #9aa6b2;
    letter-spacing: 0.02em;
  }
  .status {
    margin-top: 8px;
    font-size: 14px;
    color: #c7d0d8;
  }
  .ok {
    color: #5fd16a;
  }
  .wait {
    color: #e8963a;
  }
  .ver {
    color: #9fd8e6;
    margin-left: 6px;
  }
</style>
