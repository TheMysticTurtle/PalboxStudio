<script lang="ts">
  import "$lib/styles/tokens.css";
  let { children } = $props();
</script>

{@render children()}

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }
  :global(body) {
    background: #0a0d12;
    color: var(--text-1);
    font-family: system-ui, "Segoe UI", sans-serif;
    overflow: hidden;
  }
  :global(*) {
    box-sizing: border-box;
  }
  :global(::-webkit-scrollbar) {
    width: 10px;
    height: 10px;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: rgba(176, 96, 224, 0.35);
    border-radius: 6px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
</style>
