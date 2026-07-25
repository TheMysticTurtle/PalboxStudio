<script lang="ts">
  import type { Pal } from "$lib/data/types";
  import { LIMITS, ELEMENT_COLOR } from "$lib/data/constants";
  import SectionHeader from "./SectionHeader.svelte";
  import ElementPill from "./ElementPill.svelte";
  import PassiveChip from "./PassiveChip.svelte";
  import WorkSuitRow from "./WorkSuitRow.svelte";

  let { pal }: { pal: Pal } = $props();

  const genderSymbol = $derived(pal.gender === "Male" ? "♂" : pal.gender === "Female" ? "♀" : "–");
  const hpPct = $derived(Math.min(100, (pal.stats.hp / pal.stats.hpMax) * 100));

  function setLevel(v: number) {
    const n = Math.round(v);
    pal.level = Math.max(LIMITS.levelMin, Math.min(LIMITS.levelMax, Number.isFinite(n) ? n : LIMITS.levelMin));
  }

  // Moves: click or drag between the equipped zone and the bench.
  let emptySlots = $derived(Math.max(0, LIMITS.equippedMovesMax - pal.activeSkills.length));

  function equip(id: string) {
    const i = pal.benchMoves.findIndex((m) => m.id === id);
    if (i < 0) return;
    const [m] = pal.benchMoves.splice(i, 1);
    if (pal.activeSkills.length >= LIMITS.equippedMovesMax) {
      const dropped = pal.activeSkills.shift();
      if (dropped) pal.benchMoves.push(dropped); // swap the oldest out
    }
    pal.activeSkills.push(m);
  }
  function unequip(id: string) {
    const i = pal.activeSkills.findIndex((m) => m.id === id);
    if (i < 0) return;
    const [m] = pal.activeSkills.splice(i, 1);
    pal.benchMoves.push(m);
  }
  function onDragStart(e: DragEvent, id: string) {
    e.dataTransfer?.setData("text/plain", id);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }
  const allowDrop = (e: DragEvent) => e.preventDefault();
  function dropEquip(e: DragEvent) {
    e.preventDefault();
    const id = e.dataTransfer?.getData("text/plain");
    if (id) equip(id);
  }
  function dropBench(e: DragEvent) {
    e.preventDefault();
    const id = e.dataTransfer?.getData("text/plain");
    if (id) unequip(id);
  }
</script>

<div class="card">
  <!-- Header -->
  <div class="head">
    <div class="idcol">
      <div class="nameline">
        <input class="name" bind:value={pal.name} spellcheck="false" aria-label="Pal name" />
        <button class="pencil" aria-label="Rename">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none"><path d="m14.5 5.5 4 4M4 20l1-4L16 5l3 3L8 19l-4 1Z" stroke="#9FD8E6" stroke-width="1.7" stroke-linejoin="round"/></svg>
        </button>
        <span class="gender {pal.gender.toLowerCase()}">{genderSymbol}</span>
      </div>
      <div class="subline">
        {#each pal.elements as el}<ElementPill element={el} />{/each}
        <span class="pdx">Palpedia {pal.paldexNo}</span>
      </div>
    </div>
    <div class="headactions">
      <button class="preset">◈ PRESETS</button>
      <button class="fav" onclick={() => (pal.favorite = !pal.favorite)} aria-pressed={pal.favorite} aria-label="Favorite">
        <svg width="22" height="22" viewBox="0 0 24 24" fill={pal.favorite ? "#F5A623" : "none"}>
          <path d="M12 2.6l2.9 5.9 6.5.95-4.7 4.6 1.1 6.45L12 21.5 6.1 20.5l1.1-6.45-4.7-4.6 6.5-.95L12 2.6Z" stroke={pal.favorite ? "#F5A623" : "#9AA6B2"} stroke-width="1.4" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- NEXT exp -->
  <div class="exp">
    <span class="cap">NEXT</span>
    <div class="track"><div class="fill" style="width:{pal.expPct * 100}%"></div></div>
    <span class="num">{pal.expToNext.toLocaleString()} EXP</span>
  </div>

  <!-- Body -->
  <div class="body">
    <!-- Left: partner + passives -->
    <div class="col left">
      <div>
        <SectionHeader title="PARTNER SKILL" />
        <div class="partner" style="--c:{ELEMENT_COLOR[pal.partnerSkill.element ?? pal.elements[0] ?? 'Neutral']}">
          <div class="pname"><span class="pdia"></span>{pal.partnerSkill.name} <span class="lv">Lv {pal.partnerSkill.level}</span></div>
          <p class="pdesc">{pal.partnerSkill.description}</p>
        </div>
      </div>
      <div class="passblock">
        <SectionHeader title="PASSIVE SKILLS">
          {#snippet right()}{pal.passives.length} / {LIMITS.passivesMax}{/snippet}
        </SectionHeader>
        <div class="passives">
          {#each pal.passives as p (p.id)}<PassiveChip passive={p} />{/each}
          {#if pal.passives.length < LIMITS.passivesMax}
            <button class="add">+ Add passive</button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Center: portrait + level + moves -->
    <div class="col center">
      <div class="portrait">
        <span class="badge alpha" class:hide={!pal.alpha}>ALPHA</span>
        <span class="badge lucky" class:hide={!pal.lucky}>✦ LUCKY</span>
        <div class="art">
          <svg width="70" height="70" viewBox="0 0 24 24" fill="none" style="opacity:.55"><circle cx="12" cy="9" r="4.4" stroke="#D6BEF2" stroke-width="1.3"/><path d="M4 21c0-4.4 3.6-7 8-7s8 2.6 8 7" stroke="#D6BEF2" stroke-width="1.3"/></svg>
          <span class="artcap">pal art / icon</span>
        </div>
        <div class="poverlay">
          <div>
            <div class="olabel">CONDENSATION</div>
            <div class="stars">
              {#each Array(4) as _, i}<span class:on={i < pal.condensation}>★</span>{/each}
            </div>
          </div>
          <div class="soulcol">
            <div class="olabel">PAL SOULS</div>
            <div class="soulval">+{pal.souls}</div>
          </div>
        </div>
      </div>

      <div class="level">
        <button class="lvbtn" onclick={() => setLevel(pal.level - 1)} aria-label="Lower level">−</button>
        <div class="lvbox">
          <div class="lvcap">LEVEL</div>
          <input class="lvnum" inputmode="numeric" value={pal.level} onchange={(e) => setLevel(+e.currentTarget.value)} aria-label="Level" />
        </div>
        <button class="lvbtn" onclick={() => setLevel(pal.level + 1)} aria-label="Raise level">+</button>
      </div>

      <div class="moves">
        <SectionHeader title="ACTIVE SKILLS">
          {#snippet right()}tap or drag · {pal.activeSkills.length} / {LIMITS.equippedMovesMax}{/snippet}
        </SectionHeader>
        <div class="moveslots" ondragover={allowDrop} ondrop={dropEquip}>
          {#each pal.activeSkills as m (m.id)}
            <button type="button" class="move equipped" draggable="true" ondragstart={(e) => onDragStart(e, m.id)} onclick={() => unequip(m.id)} title="Click or drag to unequip">
              <span class="mgrip">⠿</span>
              <span class="mdia" style="--c:{ELEMENT_COLOR[m.element]}"></span>
              <span class="mname">{m.name}</span>
              <span class="mpwrcap">PWR</span>
              <span class="mpwr">{m.power}</span>
            </button>
          {/each}
          {#each Array(emptySlots) as _, i (i)}
            <div class="emptyslot">empty slot — drag a move here</div>
          {/each}
        </div>
        <div class="bench-label">AVAILABLE MOVES</div>
        <div class="bench" ondragover={allowDrop} ondrop={dropBench}>
          {#each pal.benchMoves as m (m.id)}
            <button type="button" class="move bench-move" draggable="true" ondragstart={(e) => onDragStart(e, m.id)} onclick={() => equip(m.id)} title="Click or drag to equip">
              <span class="mgrip">⠿</span>
              <span class="mdia small" style="--c:{ELEMENT_COLOR[m.element]}"></span>
              <span class="mname">{m.name}</span>
              <span class="mpwrcap">PWR</span>
              <span class="mpwr">{m.power}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Right: stats + work suitability -->
    <div class="col right">
      <div>
        <SectionHeader title="STATS" />
        <div class="stats">
          <div class="barstat">
            <div class="brow"><span class="blabel">HP</span><span class="bval hp">{pal.stats.hp.toLocaleString()}<span class="bmax">/{pal.stats.hpMax.toLocaleString()}</span></span></div>
            <div class="track"><div class="fill hp" style="width:{hpPct}%"></div></div>
          </div>
          <div class="valrow"><span>Attack</span><span class="v">{pal.stats.attack}{#if pal.boosted.attack}<span class="up">▲</span>{/if}</span></div>
          <div class="valrow"><span>Defense</span><span class="v">{pal.stats.defense}</span></div>
          <div class="valrow"><span>Work Speed</span><span class="v">{pal.stats.workSpeed}{#if pal.boosted.workSpeed}<span class="up">▲</span>{/if}</span></div>
          <div class="barstat">
            <div class="brow"><span class="blabel">SAN</span><span class="bval">{pal.stats.san}</span></div>
            <div class="track thin"><div class="fill san" style="width:{pal.stats.san}%"></div></div>
          </div>
          <div class="barstat">
            <div class="brow"><span class="blabel">Food</span><span class="bval food">{Math.round(pal.stats.foodPct * 100)}%</span></div>
            <div class="track thin"><div class="fill food" style="width:{pal.stats.foodPct * 100}%"></div></div>
          </div>
          <div class="trust">
            <span class="tlabel">TRUST</span>
            <div class="track thick"><div class="fill trust" style="width:{pal.trust.pct * 100}%"></div></div>
            <span class="trank">Rank {pal.trust.rank}</span>
          </div>
        </div>
      </div>

      <div>
        <SectionHeader title="WORK SUITABILITY" />
        <div class="worksuit">
          {#each pal.workSuit as s (s.name)}<WorkSuitRow suit={s} />{/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .card {
    height: 100%;
    border-radius: 15px;
    background: linear-gradient(155deg, rgba(27, 39, 51, 0.9), rgba(15, 20, 27, 0.93));
    backdrop-filter: blur(18px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--text-1);
  }

  /* Header */
  .head {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 26px 15px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.045), transparent);
  }
  .idcol { flex: 1; min-width: 0; }
  .nameline { display: flex; align-items: center; gap: 11px; }
  .name {
    font-family: var(--font-head);
    font-weight: 700;
    font-size: 34px;
    line-height: 1;
    letter-spacing: 0.02em;
    color: #f7f9fb;
    background: transparent;
    border: 0;
    border-bottom: 1px dashed rgba(255, 255, 255, 0.16);
    padding: 0 2px 2px;
    max-width: 340px;
    outline: none;
  }
  .name:focus { border-bottom-color: var(--accent-cyan); }
  .pencil {
    width: 34px;
    height: 34px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
  }
  .pencil:hover { border-color: rgba(63, 199, 224, 0.5); }
  .gender {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    font-size: 17px;
  }
  .gender.male { background: rgba(63, 143, 224, 0.18); border: 1px solid rgba(63, 143, 224, 0.55); color: #8fbef2; }
  .gender.female { background: rgba(224, 95, 192, 0.18); border: 1px solid rgba(224, 95, 192, 0.55); color: #f2a0d8; }
  .gender.unknown { background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.14); color: #9aa6b2; }
  .subline { display: flex; align-items: center; gap: 9px; margin-top: 11px; flex-wrap: wrap; }
  .pdx { font-size: 12.5px; color: #6e7a86; margin-left: 4px; }
  .headactions { display: flex; align-items: center; gap: 10px; }
  .preset {
    padding: 10px 15px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #cbd3db;
    cursor: pointer;
    font-family: var(--font-head);
    font-weight: 600;
    font-size: 14px;
    letter-spacing: 0.08em;
  }
  .preset:hover { border-color: rgba(63, 199, 224, 0.5); color: #eafbff; }
  .fav {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
  }
  .fav[aria-pressed="true"] { box-shadow: 0 0 16px rgba(245, 166, 35, 0.35); border-color: rgba(245, 166, 35, 0.5); }

  /* Exp */
  .exp {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 11px 26px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  .exp .cap { font-family: var(--font-head); font-weight: 600; font-size: 12.5px; letter-spacing: 0.18em; color: #9aa6b2; }
  .exp .track { flex: 1; height: 8px; border-radius: 5px; background: rgba(255, 255, 255, 0.06); overflow: hidden; border: 1px solid rgba(255, 255, 255, 0.06); }
  .exp .fill { height: 100%; background: linear-gradient(90deg, #3fc7e0, #7fe0f2); box-shadow: 0 0 10px rgba(63, 199, 224, 0.7); }
  .exp .num { font-size: 12.5px; color: #8fa0ac; font-variant-numeric: tabular-nums; }

  /* Body grid */
  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(220px, 320px) minmax(340px, 1fr) minmax(250px, 350px);
  }
  .col { padding: 20px 22px; overflow: auto; min-width: 0; display: flex; flex-direction: column; }
  .col.left { gap: 22px; border-right: 1px solid rgba(255, 255, 255, 0.05); }
  .col.center { gap: 16px; }
  .col.right { gap: 20px; border-left: 1px solid rgba(255, 255, 255, 0.05); }

  /* Partner + passives */
  .partner { padding: 15px; border-radius: 11px; background: color-mix(in srgb, var(--c) 6%, transparent); border: 1px solid color-mix(in srgb, var(--c) 22%, transparent); }
  .pname { display: flex; align-items: center; gap: 8px; font-family: var(--font-head); font-weight: 700; font-size: 18px; color: #f3e4da; }
  .pdia { width: 10px; height: 10px; transform: rotate(45deg); background: var(--c); box-shadow: 0 0 8px color-mix(in srgb, var(--c) 70%, transparent); }
  .pname .lv { color: var(--text-muted); font-weight: 400; font-size: 13px; font-family: var(--font-body); }
  .pdesc { margin: 8px 0 0; font-size: 13px; line-height: 1.55; color: #b4a79c; }
  .passblock { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .passives { display: flex; flex-direction: column; gap: 10px; }
  .add { display: flex; align-items: center; justify-content: center; padding: 12px; border-radius: 10px; border: 1px dashed rgba(255, 255, 255, 0.16); background: transparent; color: #7c8894; cursor: pointer; font-size: 13.5px; }
  .add:hover { border-color: rgba(63, 199, 224, 0.5); color: #9fd8e6; }

  /* Portrait */
  .portrait {
    position: relative;
    flex: none;
    width: 100%;
    height: min(32vh, 290px);
    border-radius: 14px;
    overflow: hidden;
    border: 1px solid rgba(176, 96, 224, 0.35);
    background:
      repeating-linear-gradient(135deg, rgba(176, 96, 224, 0.07) 0 12px, rgba(176, 96, 224, 0.02) 12px 24px),
      radial-gradient(120% 90% at 50% 12%, rgba(240, 116, 58, 0.12), transparent 55%),
      radial-gradient(120% 100% at 50% 100%, rgba(155, 95, 224, 0.2), transparent 60%),
      linear-gradient(180deg, #141a22, #0e131a);
    box-shadow: inset 0 0 50px rgba(0, 0, 0, 0.5), inset 0 0 30px rgba(176, 96, 224, 0.15);
  }
  .badge { position: absolute; top: 12px; font-family: var(--font-head); font-weight: 700; font-size: 12px; letter-spacing: 0.1em; padding: 3px 10px; border-radius: 5px; }
  .badge.alpha { left: 12px; background: rgba(224, 90, 90, 0.85); color: #fff; box-shadow: 0 0 12px rgba(224, 90, 90, 0.5); }
  .badge.lucky { right: 12px; color: #f5c97a; background: rgba(245, 201, 122, 0.18); border: 1px solid rgba(245, 201, 122, 0.5); }
  .badge.hide { display: none; }
  .art { position: absolute; inset: 0 0 60px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; }
  .artcap { font-family: ui-monospace, Menlo, monospace; font-size: 12px; letter-spacing: 0.04em; color: rgba(214, 190, 242, 0.6); }
  .poverlay { position: absolute; left: 0; right: 0; bottom: 0; padding: 11px 14px; display: flex; align-items: flex-end; justify-content: space-between; background: linear-gradient(0deg, rgba(8, 10, 14, 0.86), transparent); }
  .olabel { font-family: var(--font-head); font-weight: 600; font-size: 10.5px; letter-spacing: 0.18em; color: #b99ad6; margin-bottom: 3px; }
  .soulcol { text-align: right; }
  .stars { font-size: 18px; letter-spacing: 2px; color: rgba(255, 255, 255, 0.22); }
  .stars .on { color: var(--accent-amber); }
  .soulval { font-family: var(--font-head); font-weight: 700; font-size: 19px; color: #9b5fe0; text-shadow: 0 0 12px rgba(155, 95, 224, 0.6); }

  /* Level */
  .level { flex: none; display: flex; align-items: center; justify-content: center; gap: 18px; }
  .lvbtn { width: 46px; height: 46px; border-radius: 12px; border: 1px solid rgba(63, 199, 224, 0.4); background: rgba(63, 199, 224, 0.12); color: #9fd8e6; cursor: pointer; font-size: 26px; line-height: 1; }
  .lvbtn:hover { background: rgba(63, 199, 224, 0.22); }
  .lvbox { text-align: center; min-width: 150px; }
  .lvcap { font-family: var(--font-head); font-weight: 600; font-size: 12px; letter-spacing: 0.24em; color: #8fe3f2; }
  .lvnum { width: 150px; text-align: center; font-family: var(--font-head); font-weight: 700; font-size: 46px; line-height: 1.05; color: #eafbff; background: transparent; border: 0; outline: none; border-bottom: 1px dashed rgba(63, 199, 224, 0.28); }
  .lvnum::-webkit-inner-spin-button, .lvnum::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }

  /* Moves */
  .moves { flex: none; }
  .moveslots { display: flex; flex-direction: column; gap: 8px; padding: 10px; border-radius: 12px; background: rgba(63, 199, 224, 0.06); border: 1px solid rgba(63, 199, 224, 0.22); }
  .move { display: flex; align-items: center; gap: 11px; width: 100%; text-align: left; padding: 10px 12px; border-radius: 9px; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.08); color: inherit; font: inherit; cursor: grab; }
  .move:active { cursor: grabbing; }
  .move:hover { border-color: rgba(63, 199, 224, 0.5); }
  .move.equipped { background: rgba(63, 199, 224, 0.05); border-color: rgba(63, 199, 224, 0.18); }
  .emptyslot { display: flex; align-items: center; justify-content: center; padding: 11px; border-radius: 9px; border: 1px dashed rgba(255, 255, 255, 0.14); color: #6e7a86; font-size: 13px; }
  .bench-label { font-family: var(--font-head); font-weight: 600; font-size: 11.5px; letter-spacing: 0.14em; color: #6e7a86; margin: 12px 2px 7px; }
  .bench { display: flex; flex-direction: column; gap: 7px; }
  .bench-move { padding: 9px 12px; }
  .mgrip { color: #7c8894; font-size: 15px; letter-spacing: -2px; }
  .mdia { width: 11px; height: 11px; flex: none; transform: rotate(45deg); background: var(--c); box-shadow: 0 0 6px var(--c); }
  .mdia.small { width: 10px; height: 10px; box-shadow: none; }
  .mname { flex: 1; font-family: var(--font-cond); font-weight: 600; font-size: 15px; color: #ede7df; }
  .bench-move .mname { font-size: 14px; color: #c6cfd7; }
  .mpwrcap { font-size: 11px; color: #8fa0ac; }
  .mpwr { font-family: var(--font-head); font-weight: 700; font-size: 17px; color: #b7c0c8; min-width: 30px; text-align: right; }
  .bench-move .mpwr { font-size: 15px; }

  /* Stats */
  .stats { display: flex; flex-direction: column; gap: 12px; }
  .brow { display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 4px; }
  .blabel { color: #9aa6b2; }
  .bval { font-family: var(--font-head); font-weight: 600; font-size: 14px; color: #cbd3db; font-variant-numeric: tabular-nums; }
  .bval.hp { color: #cfebd2; font-size: 15px; }
  .bval.food { color: #f0c39a; }
  .bmax { color: var(--text-muted); }
  .track { height: 8px; border-radius: 5px; background: rgba(255, 255, 255, 0.06); overflow: hidden; }
  .track.thin { height: 7px; }
  .track.thick { height: 9px; flex: 1; }
  .fill { height: 100%; }
  .fill.hp { background: linear-gradient(90deg, #5fd16a, #84e08d); box-shadow: 0 0 8px rgba(95, 209, 106, 0.5); }
  .fill.san { background: linear-gradient(90deg, #3fc7e0, #7fe0f2); }
  .fill.food { background: linear-gradient(90deg, #e8963a, #f2b06a); }
  .fill.trust { background: linear-gradient(90deg, #b060e0, #d89af0); box-shadow: 0 0 10px rgba(176, 96, 224, 0.6); }
  .valrow { display: flex; align-items: center; justify-content: space-between; font-size: 14px; }
  .valrow > span:first-child { color: #c6cfd7; }
  .valrow .v { display: inline-flex; align-items: center; gap: 5px; font-family: var(--font-head); font-weight: 700; font-size: 18px; color: #f2f4f6; font-variant-numeric: tabular-nums; }
  .valrow .up { color: #5fd16a; font-size: 13px; }
  .trust { display: flex; align-items: center; gap: 11px; margin-top: 2px; }
  .tlabel { font-family: var(--font-head); font-weight: 600; font-size: 12.5px; letter-spacing: 0.14em; color: #9aa6b2; }
  .trank { font-size: 12.5px; color: #c9b4e0; font-variant-numeric: tabular-nums; }

  .worksuit { display: flex; flex-direction: column; gap: 7px; }
</style>
