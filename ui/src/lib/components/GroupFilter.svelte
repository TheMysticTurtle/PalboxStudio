<script lang="ts">
  import { library } from "$lib/stores/library.svelte";

  let {
    selected = $bindable(new Set<number>()),
  }: {
    selected?: Set<number>;
  } = $props();

  function toggle(id: number) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selected = next;
  }

  $effect(() => {
    const validIds = new Set(library.groups.map((group) => group.id));
    if ([...selected].some((groupId) => !validIds.has(groupId))) {
      selected = new Set([...selected].filter((groupId) => validIds.has(groupId)));
    }
  });
</script>

{#if library.groups.length}
  <div class="group-filter" aria-label="Filter by groups">
    <span class="label">GROUP</span>
    <div class="chips">
      {#each library.groups as group (group.id)}
        <button
          type="button"
          class:on={selected.has(group.id)}
          aria-pressed={selected.has(group.id)}
          onclick={() => toggle(group.id)}
        >{group.name}</button>
      {/each}
      {#if selected.size}
        <button type="button" class="clear" onclick={() => (selected = new Set())}>Clear</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .group-filter { display: flex; align-items: flex-start; gap: 10px; }
  .label {
    min-width: 52px;
    padding-top: 8px;
    color: #9585a5;
    font: 600 var(--type-label) var(--font-head);
    letter-spacing: .1em;
  }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  button {
    min-height: var(--control-min);
    padding: 7px 12px;
    border-radius: 17px;
    cursor: pointer;
    color: #baaac7;
    background: rgba(176, 96, 224, .055);
    border: 1px solid rgba(176, 96, 224, .2);
    font-size: var(--type-caption);
  }
  button:hover { border-color: rgba(176, 96, 224, .48); color: #e2d4ef; }
  button.on {
    color: #ead9f7;
    background: rgba(176, 96, 224, .2);
    border-color: rgba(176, 96, 224, .68);
    box-shadow: 0 0 12px rgba(176, 96, 224, .14);
  }
  button.clear { color: #998ba5; background: transparent; border-style: dashed; }
</style>
