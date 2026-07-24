<script lang="ts">
  import type { Pal } from "$lib/data/types";
  import { LIMITS, ELEMENT_COLOR } from "$lib/data/constants";
  import ElementPill from "./ElementPill.svelte";
  import PassiveChip from "./PassiveChip.svelte";
  import StatBar from "./StatBar.svelte";
  import WorkSuitRow from "./WorkSuitRow.svelte";

  let { pal }: { pal: Pal } = $props();

  const genderSymbol = $derived(
    pal.gender === "Male" ? "♂" : pal.gender === "Female" ? "♀" : "–",
  );
  const partnerColor = $derived(
    ELEMENT_COLOR[pal.partnerSkill.element ?? pal.elements[0] ?? "Neutral"],
  );

  function setLevel(v: number) {
    const n = Math.round(v);
    pal.level = Math.max(LIMITS.levelMin, Math.min(LIMITS.levelMax, Number.isFinite(n) ? n : LIMITS.levelMin));
  }
</script>

<div class="card">
  <!-- Header: identity -->
  <header class="head">
    <div class="idcol">
      <div class="nameline">
        <input class="name" bind:value={pal.name} spellcheck="false" aria-label="Pal name" />
        <span class="gender {pal.gender.toLowerCase()}">{genderSymbol}</span>
      </div>
      <div class="subline">
        {#each pal.elements as el}<ElementPill element={el} />{/each}
        <span class="pdx">{pal.paldexNo}</span>
      </div>
    </div>
    <div class="headactions">
      <button class="preset">✦ PRESETS</button>
      <button
        class="fav"
        class:on={pal.favorite}
        onclick={() => (pal.favorite = !pal.favorite)}
        aria-pressed={pal.favorite}
        aria-label="Favorite"
      >★</button>
    </div>
  </header>

  <!-- NEXT exp -->
  <div class="exp">
    <span class="cap">NEXT</span>
    <div class="track"><div class="fill" style="width:{pal.expPct * 100}%"></div></div>
    <span class="num">{pal.expToNext.toLocaleString()}</span>
  </div>

  <!-- Body: three columns -->
  <div class="body">
    <!-- Left: partner + passives -->
    <section class="col">
      <h4>Partner Skill</h4>
      <div class="partner" style="--c:{partnerColor}">
        <div class="pname">{pal.partnerSkill.name} <span class="lv">Lv {pal.partnerSkill.level}</span></div>
        <p class="pdesc">{pal.partnerSkill.description}</p>
      </div>

      <h4>Passive Skills</h4>
      <div class="passives">
        {#each pal.passives as p (p.id)}<PassiveChip passive={p} />{/each}
        {#if pal.passives.length < LIMITS.passivesMax}
          <button class="add">+ Add passive</button>
        {/if}
      </div>
    </section>

    <!-- Center: portrait + level + moves -->
    <section class="col center">
      <div class="portrait">
        <div class="badges">
          {#if pal.alpha}<span class="badge alpha">ALPHA</span>{/if}
          {#if pal.lucky}<span class="badge lucky">LUCKY</span>{/if}
        </div>
        <div class="art">pal art</div>
        <div class="poverlay">
          <span class="stars">
            {#each Array(4) as _, i}<span class="star" class:on={i < pal.condensation}>★</span>{/each}
          </span>
          <span class="souls">Souls +{pal.souls}</span>
        </div>
      </div>

      <div class="level">
        <button class="lvbtn" onclick={() => setLevel(pal.level - 1)} aria-label="Lower level">−</button>
        <div class="lvbox">
          <span class="lvcap">LEVEL</span>
          <input
            class="lvnum"
            type="number"
            min={LIMITS.levelMin}
            max={LIMITS.levelMax}
            value={pal.level}
            onchange={(e) => setLevel(+e.currentTarget.value)}
            aria-label="Level"
          />
        </div>
        <button class="lvbtn" onclick={() => setLevel(pal.level + 1)} aria-label="Raise level">+</button>
      </div>

      <div class="moves">
        <h4>Active Skills</h4>
        {#each pal.activeSkills as m (m.id)}
          <div class="move">
            <span class="mdia" style="--c:{ELEMENT_COLOR[m.element]}"></span>
            <span class="mname">{m.name}</span>
            <span class="mpwr">{m.power}</span>
          </div>
        {/each}
      </div>
    </section>

    <!-- Right: stats + work suitability -->
    <section class="col">
      <h4>Stats</h4>
      <StatBar label="HP" value={pal.stats.hp} max={pal.stats.hpMax} color="var(--stat-hp)" />
      <StatBar label="Attack" value={pal.stats.attack} showBar={false} boosted={pal.boosted.attack} />
      <StatBar label="Defense" value={pal.stats.defense} showBar={false} boosted={pal.boosted.defense} />
      <StatBar label="Work Speed" value={pal.stats.workSpeed} showBar={false} boosted={pal.boosted.workSpeed} />

      <h4>Work Suitability</h4>
      <div class="worksuit">
        {#each pal.workSuit as s (s.name)}<WorkSuitRow suit={s} />{/each}
      </div>
    </section>
  </div>
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    height: 100%;
    color: var(--text-1);
  }
  h4 {
    margin: 14px 0 8px;
    font-size: 11px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--text-2);
  }

  /* Header */
  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 16px 22px;
    border-bottom: 1px solid var(--hairline);
  }
  .nameline {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .name {
    background: none;
    border: none;
    border-bottom: 1px dashed transparent;
    color: var(--text-1);
    font-size: 30px;
    font-weight: 800;
    letter-spacing: 0.02em;
    padding: 0 2px;
    max-width: 320px;
  }
  .name:hover { border-bottom-color: rgba(255, 255, 255, 0.2); }
  .name:focus { outline: none; border-bottom-color: var(--accent-purple); }
  .gender { font-size: 20px; }
  .gender.male { color: #6db6ff; }
  .gender.female { color: #ff86c0; }
  .subline {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }
  .pdx {
    color: var(--text-muted);
    font-size: 12px;
  }
  .headactions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .preset {
    background: rgba(176, 96, 224, 0.14);
    border: 1px solid rgba(176, 96, 224, 0.45);
    color: #e6ccf7;
    padding: 8px 14px;
    border-radius: var(--radius-control);
    font-size: 12px;
    letter-spacing: 0.08em;
    cursor: pointer;
  }
  .fav {
    width: 42px;
    height: 42px;
    border-radius: 50%;
    border: 1px solid var(--hairline);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-muted);
    font-size: 20px;
    cursor: pointer;
  }
  .fav.on {
    color: var(--accent-amber);
    border-color: color-mix(in srgb, var(--accent-amber) 55%, transparent);
    box-shadow: 0 0 16px color-mix(in srgb, var(--accent-amber) 40%, transparent);
  }

  /* Exp */
  .exp {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 22px;
    border-bottom: 1px solid var(--hairline);
    font-size: 12px;
    color: var(--text-2);
  }
  .exp .track {
    flex: 1;
    height: 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.07);
    overflow: hidden;
  }
  .exp .fill {
    height: 100%;
    background: var(--accent-cyan);
    box-shadow: 0 0 10px color-mix(in srgb, var(--accent-cyan) 60%, transparent);
  }
  .exp .num { color: var(--text-1); font-variant-numeric: tabular-nums; }

  /* Body columns */
  .body {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(220px, 300px) minmax(320px, 1fr) minmax(260px, 340px);
    gap: 8px;
    padding: 4px 22px 18px;
    min-height: 0;
  }
  .col {
    overflow: auto;
    min-width: 0;
    padding-right: 6px;
  }
  .col.center {
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }

  /* Partner + passives */
  .partner {
    border-left: 3px solid var(--c);
    border-radius: 8px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--c) 10%, rgba(255, 255, 255, 0.02));
  }
  .pname { font-weight: 600; font-size: 13px; }
  .pname .lv { color: var(--text-muted); font-weight: 400; margin-left: 6px; }
  .pdesc { margin: 6px 0 0; color: var(--text-2); font-size: 11px; line-height: 1.5; }
  .passives { display: flex; flex-direction: column; gap: 8px; }
  .add {
    border: 1px dashed var(--hairline);
    background: none;
    color: var(--text-muted);
    padding: 8px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
  }
  .add:hover { color: var(--text-2); border-color: rgba(255, 255, 255, 0.2); }

  /* Portrait */
  .portrait {
    position: relative;
    height: min(30vh, 260px);
    border-radius: 12px;
    margin-top: 4px;
    background:
      repeating-linear-gradient(135deg, rgba(255, 255, 255, 0.03) 0 10px, transparent 10px 20px),
      linear-gradient(160deg, rgba(176, 96, 224, 0.12), rgba(63, 199, 224, 0.06));
    border: 1px solid var(--hairline);
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  .art { color: var(--text-muted); font-size: 12px; letter-spacing: 0.1em; }
  .badges { position: absolute; top: 8px; left: 8px; display: flex; gap: 6px; }
  .badge {
    font-size: 10px;
    letter-spacing: 0.1em;
    padding: 3px 7px;
    border-radius: 6px;
    font-weight: 700;
  }
  .badge.alpha { background: rgba(224, 90, 90, 0.85); color: #fff; }
  .badge.lucky { background: rgba(245, 200, 60, 0.9); color: #2a2306; }
  .poverlay {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background: linear-gradient(0deg, rgba(0, 0, 0, 0.55), transparent);
  }
  .star { color: rgba(255, 255, 255, 0.25); font-size: 14px; }
  .star.on { color: var(--accent-amber); text-shadow: 0 0 8px color-mix(in srgb, var(--accent-amber) 60%, transparent); }
  .souls { color: var(--accent-cyan-text); font-size: 12px; }

  /* Level editor */
  .level {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 14px;
    margin: 14px 0;
  }
  .lvbtn {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    border: 1px solid var(--hairline);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-1);
    font-size: 22px;
    cursor: pointer;
  }
  .lvbtn:hover { border-color: color-mix(in srgb, var(--accent-cyan) 50%, transparent); }
  .lvbox { text-align: center; }
  .lvcap { display: block; font-size: 10px; letter-spacing: 0.2em; color: var(--text-muted); }
  .lvnum {
    width: 90px;
    background: none;
    border: none;
    color: var(--text-1);
    font-size: 42px;
    font-weight: 800;
    text-align: center;
  }
  .lvnum:focus { outline: none; }
  /* hide number spinners */
  .lvnum::-webkit-inner-spin-button,
  .lvnum::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }

  /* Moves */
  .moves {
    background: color-mix(in srgb, var(--accent-cyan) 5%, transparent);
    border: 1px solid var(--hairline);
    border-radius: 10px;
    padding: 6px 12px 12px;
  }
  .move {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 6px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .move:last-child { border-bottom: none; }
  .mdia { width: 12px; height: 12px; transform: rotate(45deg); border-radius: 2px; background: var(--c); box-shadow: 0 0 8px var(--c); }
  .mname { flex: 1; font-size: 13px; }
  .mpwr { font-variant-numeric: tabular-nums; color: var(--text-2); font-weight: 700; }

  /* Work suitability */
  .worksuit { display: flex; flex-direction: column; gap: 2px; }
</style>
