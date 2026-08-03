<script lang="ts">
  import type { PassivePreset } from "$lib/data/engine";
  import { LIMITS } from "$lib/data/constants";
  import { resolvePassive } from "$lib/data/refdata.svelte";
  import {
    deleteUserPreset,
    library,
    saveUserPreset,
  } from "$lib/stores/library.svelte";
  import PassiveChip from "./PassiveChip.svelte";
  import PassiveSelector from "./PassiveSelector.svelte";

  let {
    disabled = false,
    currentPassiveCodes = [],
    onapply,
  }: {
    disabled?: boolean;
    currentPassiveCodes?: string[];
    onapply: (passiveCodes: string[]) => void;
  } = $props();

  let builderOpen = $state(false);
  let pickerOpen = $state(false);
  let pickerEditing = $state<number | null>(null);
  let draftId = $state<number | null>(null);
  let draftName = $state("");
  let draftCodes = $state<string[]>([]);
  let busy = $state(false);
  let error = $state("");

  function passiveName(code: string) {
    return resolvePassive(code)?.name ?? code;
  }

  function editPreset(preset?: PassivePreset) {
    draftId = preset?.id ?? null;
    draftName = preset?.name ?? "";
    draftCodes = [...(preset?.passiveCodes ?? [])];
    pickerEditing = null;
    error = "";
    builderOpen = true;
  }

  function openPicker(index: number | null) {
    pickerEditing = index;
    pickerOpen = true;
  }

  function copyCurrentPassives() {
    draftCodes = [...new Set(currentPassiveCodes)]
      .filter(Boolean)
      .slice(0, LIMITS.passivesMax);
    pickerEditing = null;
    error = "";
  }

  function choosePassive(code: string) {
    if (pickerEditing == null) {
      if (draftCodes.length < LIMITS.passivesMax && !draftCodes.includes(code)) draftCodes.push(code);
    } else if (!draftCodes.some((value, index) => value === code && index !== pickerEditing)) {
      draftCodes[pickerEditing] = code;
    }
  }

  async function save() {
    if (busy || !draftName.trim() || !draftCodes.length) return;
    busy = true;
    error = "";
    try {
      await saveUserPreset(draftName, [...draftCodes], draftId);
      builderOpen = false;
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  async function removePreset() {
    if (draftId == null || busy) return;
    const preset = library.presets.find((value) => value.id === draftId);
    let confirmed = false;
    try {
      const { ask } = await import("@tauri-apps/plugin-dialog");
      confirmed = await ask(
        `Delete the passive preset "${preset?.name ?? draftName}"?`,
        { title: "Delete passive preset", kind: "warning" },
      );
    } catch {
      confirmed = window.confirm(`Delete the passive preset "${preset?.name ?? draftName}"?`);
    }
    if (!confirmed) return;
    busy = true;
    error = "";
    try {
      await deleteUserPreset(draftId);
      builderOpen = false;
    } catch (value) {
      error = String(value);
    } finally {
      busy = false;
    }
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape" && !pickerOpen) builderOpen = false;
  }
</script>

<svelte:window onkeydown={builderOpen ? onKey : undefined} />

<div class="presets">
  {#each library.presets as preset (preset.id)}
    <div class="preset-wrap">
      <button
        type="button"
        class="preset"
        disabled={disabled}
        title={`Apply: ${preset.passiveCodes.map(passiveName).join(", ")}`}
        onclick={() => onapply([...preset.passiveCodes])}
      >
        <strong>{preset.name}</strong>
        <span>{preset.passiveCodes.length}/{LIMITS.passivesMax}</span>
      </button>
      <button
        type="button"
        class="configure"
        onclick={() => editPreset(preset)}
        aria-label="Edit {preset.name}"
        title="Edit preset"
      >✎</button>
    </div>
  {/each}
  <button type="button" class="build" onclick={() => editPreset()}>+ Build preset</button>
</div>

{#if builderOpen}
  <div class="scrim" role="presentation" onclick={() => (builderOpen = false)}></div>
  <div class="builder" role="dialog" aria-modal="true" aria-label="Passive preset builder">
    <header>
      <span class="diamond"></span>
      <div>
        <h2>{draftId == null ? "NEW PASSIVE PRESET" : "EDIT PASSIVE PRESET"}</h2>
        <p>Choose up to {LIMITS.passivesMax} passives from the full filtered reference list.</p>
      </div>
      <button class="close" onclick={() => (builderOpen = false)} aria-label="Close">×</button>
    </header>

    <label class="name-field">
      <span>PRESET NAME</span>
      <input bind:value={draftName} maxlength="80" placeholder="e.g. Perfect Base Worker" />
    </label>

    <div class="copy-row">
      <button
        class="copy-current"
        disabled={disabled || !currentPassiveCodes.length}
        onclick={copyCurrentPassives}
        title={disabled || !currentPassiveCodes.length
          ? "Select a Pal with passive skills first"
          : "Replace this draft with the selected Pal's current passive skills"}
      >⧉ Copy current Pal's passives</button>
    </div>

    <div class="slots">
      {#each draftCodes as code, index (index)}
        <div class="slot">
          <span class="slot-no">{index + 1}</span>
          <PassiveChip {code} onselect={() => openPicker(index)} />
          <button
            class="remove-passive"
            onclick={() => draftCodes.splice(index, 1)}
            aria-label="Remove {passiveName(code)}"
          >×</button>
        </div>
      {/each}
      {#if draftCodes.length < LIMITS.passivesMax}
        <button class="add-passive" onclick={() => openPicker(null)}>
          + Filter & add passive ({draftCodes.length}/{LIMITS.passivesMax})
        </button>
      {/if}
    </div>

    {#if error}<div class="error">{error}</div>{/if}

    <footer>
      {#if draftId != null}
        <button class="delete" disabled={busy} onclick={removePreset}>Delete preset</button>
      {/if}
      <span></span>
      <button class="cancel" onclick={() => (builderOpen = false)}>Cancel</button>
      <button
        class="save"
        disabled={busy || !draftName.trim() || !draftCodes.length}
        onclick={save}
      >{busy ? "Saving…" : "Save preset"}</button>
    </footer>
  </div>

  <PassiveSelector
    bind:open={pickerOpen}
    species=""
    selected={draftCodes}
    editing={pickerEditing}
    onpick={choosePassive}
  />
{/if}

<style>
  .presets { display: flex; flex-wrap: wrap; gap: 7px; }
  .preset-wrap { display: inline-flex; min-width: 0; }
  .preset, .configure, .build {
    min-height: var(--control-min);
    cursor: pointer;
    font-size: var(--type-caption);
  }
  .preset {
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 7px 9px 7px 11px;
    border-radius: 10px 0 0 10px;
    color: #d9e8df;
    background: rgba(53, 201, 165, .09);
    border: 1px solid rgba(53, 201, 165, .27);
    border-right: 0;
  }
  .preset strong { overflow: hidden; max-width: 150px; font: 600 14px var(--font-cond); text-overflow: ellipsis; white-space: nowrap; }
  .preset span { color: #7eb8a8; font: 700 var(--type-micro) var(--font-head); }
  .preset:hover:not(:disabled) { color: #effff8; background: rgba(53, 201, 165, .16); border-color: rgba(53, 201, 165, .52); }
  .preset:disabled { opacity: .42; cursor: default; }
  .configure {
    width: 34px;
    border-radius: 0 10px 10px 0;
    color: #91aa9f;
    background: rgba(53, 201, 165, .06);
    border: 1px solid rgba(53, 201, 165, .27);
  }
  .configure:hover { color: #d9fff1; background: rgba(53, 201, 165, .14); }
  .build {
    padding: 7px 11px;
    border-radius: 10px;
    color: #9fd8e6;
    background: rgba(63,199,224,.07);
    border: 1px dashed rgba(63,199,224,.35);
  }

  .scrim { position: fixed; inset: 0; z-index: 92; background: rgba(8, 9, 13, .68); backdrop-filter: blur(4px); }
  .builder {
    position: fixed;
    z-index: 93;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(720px, 94vw);
    max-height: min(84vh, 800px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 16px;
    color: #e6edf0;
    background: linear-gradient(155deg, rgba(21, 31, 39, .99), rgba(13, 17, 23, .99));
    border: 1px solid rgba(63,199,224,.38);
    box-shadow: 0 28px 90px rgba(0,0,0,.68), 0 0 42px rgba(63,199,224,.14);
  }
  header { display: flex; align-items: center; gap: 12px; padding: 16px 19px; border-bottom: 1px solid rgba(63,199,224,.19); }
  .diamond { width: 10px; height: 10px; flex: none; transform: rotate(45deg); background: var(--accent-cyan); box-shadow: 0 0 8px var(--accent-cyan); }
  h2 { margin: 0; color: #eafbff; font: 700 20px var(--font-head); letter-spacing: .13em; }
  header p { margin: 2px 0 0; color: #84939d; font-size: var(--type-caption); }
  .close { margin-left: auto; width: 36px; height: 36px; border-radius: 9px; cursor: pointer; color: #aebbc3; background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.12); font-size: 18px; }

  .name-field { display: flex; flex-direction: column; gap: 6px; padding: 16px 19px 0; }
  .name-field span { color: #8797a4; font: 600 var(--type-label) var(--font-head); letter-spacing: .1em; }
  .name-field input {
    min-height: 42px;
    padding: 8px 12px;
    color: #e7edf2;
    background: rgba(255,255,255,.045);
    border: 1px solid rgba(255,255,255,.11);
    border-radius: 9px;
    outline: 0;
    font-size: var(--type-control);
  }
  .name-field input:focus { border-color: rgba(63,199,224,.58); }

  .copy-row { display: flex; justify-content: flex-end; padding: 10px 19px 0; }
  .copy-current {
    min-height: 36px;
    padding: 7px 11px;
    border-radius: 9px;
    cursor: pointer;
    color: #bde8db;
    background: rgba(53,201,165,.08);
    border: 1px solid rgba(53,201,165,.3);
    font: 600 var(--type-caption) var(--font-head);
  }
  .copy-current:hover:not(:disabled) { color: #effff8; background: rgba(53,201,165,.16); border-color: rgba(53,201,165,.52); }
  .copy-current:disabled { opacity: .4; cursor: default; }

  .slots { min-height: 0; overflow: auto; display: flex; flex-direction: column; gap: 9px; padding: 16px 19px; }
  .slot { display: grid; grid-template-columns: 28px minmax(0, 1fr) 36px; align-items: center; gap: 8px; }
  .slot-no { color: #70828e; font: 700 15px var(--font-head); text-align: center; }
  .remove-passive {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    cursor: pointer;
    color: #dc9898;
    background: rgba(224,90,90,.08);
    border: 1px solid rgba(224,90,90,.25);
    font-size: 18px;
  }
  .add-passive {
    min-height: 42px;
    border-radius: 10px;
    cursor: pointer;
    color: #9fd8e6;
    background: rgba(63,199,224,.07);
    border: 1px dashed rgba(63,199,224,.35);
    font-size: var(--type-body);
  }
  .error { margin: 0 19px 14px; padding: 10px 12px; border-radius: 9px; color: #f0aaaa; background: rgba(224,90,90,.1); border: 1px solid rgba(224,90,90,.28); font-size: var(--type-caption); }

  footer { display: grid; grid-template-columns: auto 1fr auto auto; gap: 8px; padding: 13px 19px 17px; border-top: 1px solid rgba(255,255,255,.065); }
  footer button { min-height: 38px; padding: 8px 13px; border-radius: 9px; cursor: pointer; font-size: var(--type-caption); }
  footer button:disabled { opacity: .4; cursor: default; }
  footer .delete { color: #e99d9d; background: rgba(224,90,90,.08); border: 1px solid rgba(224,90,90,.28); }
  footer .cancel { color: #aab6be; background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.1); }
  footer .save { color: #eafbff; background: rgba(63,199,224,.15); border: 1px solid rgba(63,199,224,.43); }
</style>
