<script lang="ts">
  import type { Pal } from "$lib/data/types";
  import { LIMITS } from "$lib/data/constants";
  import SectionHeader from "./SectionHeader.svelte";

  let { pal }: { pal: Pal } = $props();

  const clampIV = (v: number) => Math.max(LIMITS.ivMin, Math.min(LIMITS.ivMax, Math.round(v) || 0));

  const ivRows = [
    { key: "hp", label: "HP", note: "", color: "var(--stat-hp)" },
    { key: "shot", label: "Attack", note: "single talent · 1.0", color: "var(--accent-amber)" },
    { key: "defense", label: "Defense", note: "", color: "var(--accent-cyan)" },
  ] as const;

  const soulStats = [
    { key: "hp", label: "HP", color: "var(--stat-hp)" },
    { key: "attack", label: "ATK", color: "var(--accent-amber)" },
    { key: "defense", label: "DEF", color: "var(--accent-cyan)" },
    { key: "craftSpeed", label: "WS", color: "var(--accent-purple)" },
  ] as const;

  const decSoul = (k: (typeof soulStats)[number]["key"]) =>
    (pal.soulRanks[k] = Math.max(LIMITS.soulsMin, pal.soulRanks[k] - 1));
  const incSoul = (k: (typeof soulStats)[number]["key"]) =>
    (pal.soulRanks[k] = Math.min(LIMITS.soulsMax, pal.soulRanks[k] + 1));
  const setSoul = (k: (typeof soulStats)[number]["key"], i: number) =>
    (pal.soulRanks[k] = pal.soulRanks[k] === i + 1 ? i : i + 1);

  const setCondensation = (i: number) =>
    (pal.condensation = pal.condensation === i + 1 ? i : i + 1);
</script>

<div class="adv">
  <!-- IV / breeding -->
  <div>
    <SectionHeader title="IV / BREEDING TRAITS" />
    <div class="ivs">
      {#each ivRows as r (r.key)}
        <div class="iv">
          <div class="ivhead">
            <span class="ivlabel">{r.label}{#if r.note}<span class="note">({r.note})</span>{/if}</span>
            <input
              class="ivnum"
              inputmode="numeric"
              value={pal.ivs[r.key]}
              onchange={(e) => (pal.ivs[r.key] = clampIV(+e.currentTarget.value))}
              style="--c:{r.color}"
              aria-label="{r.label} IV"
            />
          </div>
          <input
            class="slider"
            type="range"
            min={LIMITS.ivMin}
            max={LIMITS.ivMax}
            bind:value={pal.ivs[r.key]}
            style="--pct:{pal.ivs[r.key]}%; --c:{r.color}"
            aria-label="{r.label} IV slider"
          />
        </div>
      {/each}
    </div>
  </div>

  <!-- Statue of Power -->
  <div>
    <SectionHeader title="STATUE OF POWER" accent="var(--accent-purple)" color="#D6BEF2" />
    <div class="statue">
      <div class="statueimg">
        <span class="statueemoji">🗿</span>
        <span class="statuecap">statue of power</span>
      </div>
      <div class="souls">
        <div class="soulhead"><span>Pal Soul Enhancement</span><span class="muted">rank 0–10 · +3%/rank</span></div>
        {#each soulStats as s (s.key)}
          <div class="soulrow" style="--c:{s.color}">
            <span class="soullabel">{s.label} <b>+{pal.soulRanks[s.key]}</b></span>
            <button class="sbtn" onclick={() => decSoul(s.key)} disabled={pal.soulRanks[s.key] <= LIMITS.soulsMin} aria-label="Lower {s.label} soul">−</button>
            <div class="pips">
              {#each Array(LIMITS.soulsMax) as _, i (i)}
                <button class="pip" class:on={i < pal.soulRanks[s.key]} onclick={() => setSoul(s.key, i)} aria-label="{s.label} soul rank {i + 1}"></button>
              {/each}
            </div>
            <button class="sbtn" onclick={() => incSoul(s.key)} disabled={pal.soulRanks[s.key] >= LIMITS.soulsMax} aria-label="Raise {s.label} soul">+</button>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Condensation -->
  <div>
    <SectionHeader title="CONDENSATION" accent="var(--accent-amber)" color="#F5C97A" />
    <div class="cond">
      <div class="stars">
        {#each Array(LIMITS.condensationMax) as _, i (i)}
          <button class="star" class:on={i < pal.condensation} onclick={() => setCondensation(i)} aria-label="Condensation {i + 1} stars">★</button>
        {/each}
      </div>
      <div class="condnote">{pal.condensation} / {LIMITS.condensationMax} stars · +{pal.condensation * 5}% HP·Atk·Def</div>
    </div>
  </div>

  <!-- Safety note -->
  <div class="warn">
    <span class="warnicon">⚠</span>
    <p>Editing genetic data rewrites the save. Your original is safely <b>backed up</b> before any write.</p>
  </div>
</div>

<style>
  .adv { display: flex; flex-direction: column; gap: 22px; }

  /* IV sliders */
  .ivs { display: flex; flex-direction: column; gap: 17px; }
  .ivhead { display: flex; align-items: center; justify-content: space-between; margin-bottom: 7px; }
  .ivlabel { font-size: 14px; color: #c6cfd7; }
  .note { color: #6e7a86; font-size: 11.5px; margin-left: 5px; }
  .ivnum {
    width: 54px; text-align: right; background: transparent; border: 0; outline: none;
    font-family: var(--font-head); font-weight: 700; font-size: 17px; color: var(--c);
    border-bottom: 1px dashed color-mix(in srgb, var(--c) 40%, transparent);
  }
  .ivnum::-webkit-inner-spin-button, .ivnum::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
  .slider { -webkit-appearance: none; appearance: none; width: 100%; height: 8px; border-radius: 5px; cursor: pointer;
    background: linear-gradient(90deg, var(--c) var(--pct), rgba(255, 255, 255, 0.08) var(--pct)); }
  .slider::-webkit-slider-thumb { -webkit-appearance: none; width: 18px; height: 18px; border-radius: 50%;
    background: #f2f6f8; border: 2px solid var(--c); box-shadow: 0 0 8px color-mix(in srgb, var(--c) 60%, transparent); cursor: pointer; }

  /* Statue */
  .statue { display: flex; gap: 14px; align-items: stretch; }
  .statueimg {
    width: 100px; flex: none; border-radius: 12px; position: relative; overflow: hidden;
    border: 1px solid rgba(176, 96, 224, 0.4);
    background: repeating-linear-gradient(135deg, rgba(176, 96, 224, 0.09) 0 10px, rgba(176, 96, 224, 0.02) 10px 20px), linear-gradient(180deg, #181120, #0f0d16);
    display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px;
    box-shadow: inset 0 0 30px rgba(176, 96, 224, 0.2);
  }
  .statueemoji { font-size: 40px; }
  .statuecap { font-family: ui-monospace, Menlo, monospace; font-size: 9.5px; color: rgba(214, 190, 242, 0.6); text-align: center; }
  .souls { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 11px; }
  .soulhead { display: flex; align-items: center; justify-content: space-between; font-family: var(--font-cond); font-weight: 600; font-size: 12.5px; color: #b99ad6; }
  .muted { color: #6e7a86; font-weight: 400; font-size: 11px; }
  .soulrow { display: flex; align-items: center; gap: 8px; }
  .soullabel { width: 58px; font-size: 13px; color: color-mix(in srgb, var(--c) 45%, #ffffff); }
  .soullabel b { color: var(--c); font-family: var(--font-head); font-size: 15px; }
  .sbtn { width: 24px; height: 24px; flex: none; border-radius: 7px; border: 1px solid color-mix(in srgb, var(--c) 35%, transparent); background: color-mix(in srgb, var(--c) 10%, transparent); color: color-mix(in srgb, var(--c) 45%, #ffffff); cursor: pointer; font-size: 14px; line-height: 1; }
  .sbtn:disabled { opacity: 0.35; cursor: default; }
  .pips { flex: 1; display: flex; gap: 2px; }
  .pip { flex: 1; height: 10px; border: 0; border-radius: 2px; background: rgba(255, 255, 255, 0.08); cursor: pointer; padding: 0; }
  .pip.on { background: var(--c); box-shadow: 0 0 6px color-mix(in srgb, var(--c) 55%, transparent); }

  /* Condensation */
  .cond { padding: 14px; border-radius: 12px; background: rgba(245, 166, 35, 0.07); border: 1px solid rgba(245, 166, 35, 0.26); text-align: center; }
  .stars { display: flex; justify-content: center; gap: 7px; }
  .star { background: none; border: 0; cursor: pointer; font-size: 30px; line-height: 1; color: rgba(255, 255, 255, 0.2); padding: 0; }
  .star.on { color: var(--accent-amber); text-shadow: 0 0 10px color-mix(in srgb, var(--accent-amber) 60%, transparent); }
  .condnote { font-size: 11px; color: #6e7a86; margin-top: 9px; }

  /* Warning */
  .warn { display: flex; gap: 10px; padding: 13px 14px; border-radius: 11px; background: rgba(245, 166, 35, 0.08); border: 1px solid rgba(245, 166, 35, 0.28); }
  .warnicon { color: var(--accent-amber); font-size: 17px; line-height: 1.2; }
  .warn p { margin: 0; font-size: 13px; line-height: 1.55; color: #d9c39a; }
  .warn b { color: #f5c97a; }
</style>
