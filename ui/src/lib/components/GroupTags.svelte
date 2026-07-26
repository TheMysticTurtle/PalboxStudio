<script lang="ts">
  import type { UserGroup } from "$lib/data/engine";
  import {
    assignPalGroups,
    createUserGroup,
    deleteUserGroup,
    groupIdsFor,
    groupNamesFor,
    library,
    renameUserGroup,
  } from "$lib/stores/library.svelte";

  let {
    instanceId,
    disabled = false,
  }: {
    instanceId: string;
    disabled?: boolean;
  } = $props();

  let root: HTMLDivElement;
  let popover = $state<HTMLDivElement>();
  let open = $state(false);
  let managing = $state(false);
  let newName = $state("");
  let editingId = $state<number | null>(null);
  let editingName = $state("");
  let busy = $state(false);
  let error = $state("");

  const assignedIds = $derived(new Set(groupIdsFor(instanceId)));
  const assignedNames = $derived(groupNamesFor(instanceId));

  function close() {
    open = false;
    managing = false;
    editingId = null;
    error = "";
  }

  function openPicker() {
    if (disabled) return;
    open = !open;
    managing = false;
    editingId = null;
    error = "";
  }

  function onWindowPointerDown(event: PointerEvent) {
    const target = event.target as Node;
    if (open && root && !root.contains(target) && !popover?.contains(target)) close();
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (managing) {
        managing = false;
        editingId = null;
        error = "";
      } else {
        close();
      }
    }
  }

  async function toggle(groupId: number) {
    if (busy || disabled || !instanceId) return;
    busy = true;
    error = "";
    const next = new Set(assignedIds);
    if (next.has(groupId)) next.delete(groupId);
    else next.add(groupId);
    try {
      await assignPalGroups(instanceId, [...next]);
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  async function create() {
    const name = newName.trim();
    if (!name || busy) return;
    busy = true;
    error = "";
    try {
      const group = await createUserGroup(name);
      newName = "";
      if (instanceId && !disabled) {
        await assignPalGroups(instanceId, [...assignedIds, group.id]);
      }
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  function beginRename(group: UserGroup) {
    editingId = group.id;
    editingName = group.name;
  }

  async function saveRename() {
    const name = editingName.trim();
    if (editingId == null || !name || busy) return;
    busy = true;
    error = "";
    try {
      await renameUserGroup(editingId, name);
      editingId = null;
      editingName = "";
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  async function remove(group: UserGroup) {
    if (busy) return;
    let confirmed = false;
    try {
      const { ask } = await import("@tauri-apps/plugin-dialog");
      confirmed = await ask(
        `Delete the tag "${group.name}"?\n\nIt will be removed from every Pal. No Palworld save data is changed.`,
        { title: "Delete tag", kind: "warning" },
      );
    } catch {
      confirmed = window.confirm(`Delete the tag "${group.name}"?`);
    }
    if (!confirmed) return;
    busy = true;
    error = "";
    try {
      await deleteUserGroup(group.id);
      if (editingId === group.id) editingId = null;
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node);

    function position() {
      const anchor = root.getBoundingClientRect();
      const margin = 12;
      const width = Math.min(320, window.innerWidth - margin * 2);
      const visibleRows = Math.min(Math.max(library.groups.length, 3), 6);
      const listHeight = visibleRows * 39 + Math.max(0, visibleRows - 1) * 5 + 16;
      const preferredHeight = library.groups.length
        ? 54 + listHeight + (managing ? 57 : 50)
        : managing ? 220 : 190;
      const naturalHeight = Math.min(
        Math.max(node.scrollHeight, preferredHeight),
        window.innerHeight * 0.72,
      );
      const below = window.innerHeight - anchor.bottom - margin;
      const above = anchor.top - margin;
      const placeAbove = below < naturalHeight && above > below;
      const available = Math.max(160, placeAbove ? above : below);
      const height = Math.min(naturalHeight, available);
      const left = Math.min(
        window.innerWidth - width - margin,
        Math.max(margin, anchor.right - width),
      );

      node.style.width = `${width}px`;
      node.style.left = `${left}px`;
      node.style.top = placeAbove
        ? `${Math.max(margin, anchor.top - height - 8)}px`
        : `${anchor.bottom + 8}px`;
      node.style.height = `${height}px`;
      node.style.maxHeight = `${height}px`;
    }

    const observer = new ResizeObserver(position);
    observer.observe(node);
    position();
    window.addEventListener("resize", position);
    window.addEventListener("scroll", position, true);

    return {
      destroy() {
        observer.disconnect();
        window.removeEventListener("resize", position);
        window.removeEventListener("scroll", position, true);
        node.remove();
      },
    };
  }
</script>

<svelte:window
  onpointerdown={open ? onWindowPointerDown : undefined}
  onkeydown={open ? onKey : undefined}
/>

<div class="tag-control" bind:this={root}>
  <div class="summary">
    <div class="assigned">
      {#each assignedNames as name (name)}<span>{name}</span>{/each}
      {#if !assignedNames.length}<em>No tags assigned</em>{/if}
    </div>
    <button
      type="button"
      class="pick"
      class:on={open}
      disabled={disabled}
      aria-haspopup="listbox"
      aria-expanded={open}
      onclick={openPicker}
    >Tags <span aria-hidden="true">⌄</span></button>
  </div>

  {#if open}
    <div class="popover" bind:this={popover} use:portal>
      {#if managing}
        <div class="popover-head">
          <button class="back" type="button" onclick={() => { managing = false; editingId = null; error = ""; }}>‹</button>
          <div>
            <strong>CREATE & MANAGE TAGS</strong>
            <small>Reusable across every Palbox</small>
          </div>
        </div>

        <form class="create" onsubmit={(event) => { event.preventDefault(); create(); }}>
          <input
            bind:value={newName}
            maxlength="80"
            placeholder="New tag name…"
            aria-label="New tag name"
          />
          <button type="submit" disabled={busy || !newName.trim()}>Create</button>
        </form>

        <div class="manage-list">
          {#each library.groups as group (group.id)}
            <div class="manage-row">
              {#if editingId === group.id}
                <input
                  class="rename-input"
                  bind:value={editingName}
                  maxlength="80"
                  aria-label="Rename {group.name}"
                  onkeydown={(event) => {
                    if (event.key === "Enter") { event.preventDefault(); saveRename(); }
                  }}
                />
                <button class="mini save" disabled={busy || !editingName.trim()} onclick={saveRename}>Save</button>
                <button class="mini" onclick={() => (editingId = null)}>Cancel</button>
              {:else}
                <span class="tag-name">{group.name}</span>
                <button class="mini" disabled={busy} onclick={() => beginRename(group)}>Rename</button>
                <button class="mini delete" disabled={busy} onclick={() => remove(group)}>Delete</button>
              {/if}
            </div>
          {/each}
          {#if !library.groups.length}<div class="empty">Create your first reusable tag above.</div>{/if}
        </div>
      {:else}
        <div class="popover-head">
          <div>
            <strong>SELECT TAGS</strong>
            <small>{instanceId ? "Choose any that apply to this Pal" : "Select a Pal to assign tags"}</small>
          </div>
        </div>

        <div class="tag-list" role="listbox" aria-label="Pal tags" aria-multiselectable="true">
          {#each library.groups as group (group.id)}
            <button
              type="button"
              class:on={assignedIds.has(group.id)}
              disabled={busy || !instanceId}
              role="option"
              aria-selected={assignedIds.has(group.id)}
              onclick={() => toggle(group.id)}
            >
              <span class="check">{assignedIds.has(group.id) ? "✓" : ""}</span>
              <span>{group.name}</span>
            </button>
          {/each}
          {#if !library.groups.length}<div class="empty">No tags created yet.</div>{/if}
        </div>

        <button class="manage" type="button" onclick={() => { managing = true; error = ""; }}>
          + Create or manage tags
        </button>
      {/if}

      {#if error}<div class="error">{error}</div>{/if}
    </div>
  {/if}
</div>

<style>
  .tag-control { position: relative; min-width: 0; }
  .summary { display: flex; align-items: center; gap: 8px; }
  .assigned { min-width: 0; flex: 1; display: flex; flex-wrap: wrap; gap: 6px; }
  .assigned span {
    padding: 5px 9px;
    border-radius: 12px;
    color: #d9c4eb;
    background: rgba(176, 96, 224, .13);
    border: 1px solid rgba(176, 96, 224, .28);
    font-size: var(--type-caption);
  }
  .assigned em { color: #747d86; font-size: var(--type-caption); }
  .pick {
    min-height: var(--control-min);
    padding: 7px 11px;
    flex: none;
    border-radius: 9px;
    cursor: pointer;
    color: #d6bef2;
    background: rgba(176, 96, 224, .11);
    border: 1px solid rgba(176, 96, 224, .35);
    font-size: var(--type-caption);
  }
  .pick.on, .pick:hover { color: #f0e3f9; border-color: rgba(176, 96, 224, .7); }
  .pick:disabled { opacity: .42; cursor: default; }

  .popover {
    position: fixed;
    z-index: 96;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    color: #e9e2ee;
    background: linear-gradient(155deg, rgba(29, 21, 38, .995), rgba(15, 16, 23, .995));
    border: 1px solid rgba(176, 96, 224, .45);
    box-shadow: 0 18px 50px rgba(0, 0, 0, .62), 0 0 28px rgba(176, 96, 224, .12);
  }
  .popover-head {
    min-height: 54px;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 10px 12px;
    border-bottom: 1px solid rgba(176, 96, 224, .18);
  }
  .popover-head strong { display: block; color: #e8daef; font: 700 var(--type-body) var(--font-head); letter-spacing: .09em; }
  .popover-head small { display: block; margin-top: 2px; color: #8f8398; font-size: var(--type-label); }
  .back {
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 8px;
    cursor: pointer;
    color: #d7c7e3;
    background: rgba(255,255,255,.045);
    border: 1px solid rgba(255,255,255,.1);
    font-size: 22px;
  }

  .tag-list, .manage-list {
    flex: 1 1 auto;
    min-height: 0;
    overflow: auto;
    padding: 8px;
  }
  .tag-list { display: flex; flex-direction: column; gap: 5px; }
  .tag-list button {
    min-height: 39px;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 6px 9px;
    border-radius: 8px;
    cursor: pointer;
    color: #bdb2c5;
    text-align: left;
    background: rgba(255,255,255,.025);
    border: 1px solid rgba(255,255,255,.075);
    font: 600 var(--type-body) var(--font-cond);
  }
  .tag-list button:hover, .tag-list button.on {
    color: #eadcf4;
    background: rgba(176,96,224,.12);
    border-color: rgba(176,96,224,.4);
  }
  .check {
    width: 22px;
    height: 22px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: #f0ddff;
    background: rgba(176,96,224,.08);
    border: 1px solid rgba(176,96,224,.32);
    font-weight: 700;
  }
  .manage {
    flex: none;
    min-height: 42px;
    margin: 0 8px 8px;
    border-radius: 8px;
    cursor: pointer;
    color: #d9c1ec;
    background: rgba(176,96,224,.09);
    border: 1px dashed rgba(176,96,224,.4);
    font-size: var(--type-caption);
  }

  .create { flex: none; display: flex; gap: 6px; padding: 8px; border-bottom: 1px solid rgba(255,255,255,.06); }
  .create input, .rename-input {
    min-width: 0;
    color: #eee8f2;
    background: rgba(255,255,255,.055);
    border: 1px solid rgba(255,255,255,.12);
    border-radius: 8px;
    outline: 0;
    font-size: var(--type-control);
  }
  .create input { flex: 1; padding: 7px 9px; }
  .create input:focus, .rename-input:focus { border-color: rgba(176, 96, 224, .7); }
  .create button {
    padding: 7px 10px;
    border-radius: 8px;
    cursor: pointer;
    color: #e6d4f3;
    background: rgba(176,96,224,.16);
    border: 1px solid rgba(176,96,224,.4);
    font-size: var(--type-caption);
  }
  button:disabled { opacity: .4; cursor: default; }

  .manage-list { display: flex; flex-direction: column; gap: 5px; }
  .manage-row {
    min-height: 38px;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 6px 5px 9px;
    border-radius: 8px;
    background: rgba(255,255,255,.025);
    border: 1px solid rgba(255,255,255,.07);
  }
  .tag-name {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    color: #d8cedd;
    font: 600 var(--type-body) var(--font-cond);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rename-input { flex: 1; padding: 6px 7px; }
  .mini {
    min-height: 29px;
    padding: 4px 7px;
    border-radius: 6px;
    cursor: pointer;
    color: #aaa0b2;
    background: rgba(255,255,255,.035);
    border: 1px solid rgba(255,255,255,.09);
    font-size: var(--type-label);
  }
  .mini.save { color: #bff3fb; border-color: rgba(63,199,224,.35); }
  .mini.delete { color: #db9292; border-color: rgba(224,90,90,.24); }
  .empty { padding: 20px 9px; color: #807586; text-align: center; font-size: var(--type-caption); }
  .error { flex: none; margin: 0 8px 8px; padding: 8px 9px; border-radius: 8px; color: #f0aaaa; background: rgba(224,90,90,.1); border: 1px solid rgba(224,90,90,.28); font-size: var(--type-label); }

</style>
