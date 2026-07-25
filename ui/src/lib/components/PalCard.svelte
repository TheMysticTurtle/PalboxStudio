<script lang="ts">
  import type { Pal, ElementName } from "$lib/data/types";
  import { LIMITS, ELEMENT_COLOR } from "$lib/data/constants";
  import { resolveMove, resolveSpecies } from "$lib/data/refdata.svelte";
  import { palIcon, onPalIconError } from "$lib/data/icons";
  import { maxHpForPal, reSpecies } from "$lib/data/mapper";
  import SectionHeader from "./SectionHeader.svelte";
  import ElementPill from "./ElementPill.svelte";
  import PassiveChip from "./PassiveChip.svelte";
  import WorkSuitRow from "./WorkSuitRow.svelte";
  import SpeciesSelector from "./SpeciesSelector.svelte";
  import PassiveSelector from "./PassiveSelector.svelte";
  import MoveSelector from "./MoveSelector.svelte";
  import ElementIcon from "./ElementIcon.svelte";

  let { pal, empty = false }: { pal: Pal; empty?: boolean } = $props();

  // Species display name (tolerant of an alpha BOSS_ prefix) + the selector modal.
  let speciesOpen = $state(false);
  let passiveOpen = $state(false);
  let passiveEditing = $state<number | null>(null);
  let moveOpen = $state(false);
  const speciesName = $derived(empty ? "" : (resolveSpecies(pal.species)?.name ?? pal.species));

  const genderSymbol = $derived(pal.gender === "Male" ? "♂" : pal.gender === "Female" ? "♀" : "–");
  const hpMax = $derived(empty ? 0 : maxHpForPal(pal));
  const hpPct = $derived(hpMax > 0 ? Math.min(100, (pal.stats.hp / hpMax) * 100) : 0);
  const soulTotal = $derived(
    pal.soulRanks.hp + pal.soulRanks.attack + pal.soulRanks.defense + pal.soulRanks.craftSpeed,
  );

  function setLevel(v: number) {
    const n = Math.round(v);
    pal.level = Math.max(LIMITS.levelMin, Math.min(LIMITS.levelMax, Number.isFinite(n) ? n : LIMITS.levelMin));
  }
  function finiteOr(value: number, fallback: number) {
    return Number.isFinite(value) ? value : fallback;
  }
  function setHp(value: number) {
    pal.stats.hp = Math.round(Math.max(0, Math.min(hpMax, finiteOr(value, pal.stats.hp))));
  }
  function setSanity(value: number) {
    pal.stats.san = Math.round(Math.max(0, Math.min(100, finiteOr(value, pal.stats.san))));
  }
  function setFoodPercent(value: number) {
    const percent = Math.max(0, Math.min(100, finiteOr(value, pal.stats.foodPct * 100)));
    pal.stats.foodPct = percent / 100;
  }
  function setTrustRank(value: number) {
    pal.trust.rank = Math.round(Math.max(0, Math.min(10, finiteOr(value, pal.trust.rank))));
    if (pal.trust.rank === 10) pal.trust.progress = 1;
  }
  function setTrustProgress(value: number) {
    if (pal.trust.rank >= 10) {
      pal.trust.progress = 1;
      return;
    }
    pal.trust.progress = Math.max(0, Math.min(100, finiteOr(value, pal.trust.progress * 100))) / 100;
  }

  function toggleAlpha() {
    pal.alpha = !pal.alpha;
    if (pal.alpha) pal.lucky = false;
  }
  function toggleLucky() {
    pal.lucky = !pal.lucky;
    if (pal.lucky) pal.alpha = false;
  }

  function openPassive(index: number | null) {
    passiveEditing = index;
    passiveOpen = true;
  }
  function choosePassive(code: string) {
    if (passiveEditing == null) {
      if (pal.passives.length < LIMITS.passivesMax && !pal.passives.includes(code)) pal.passives.push(code);
    } else if (!pal.passives.some((value, index) => value === code && index !== passiveEditing)) {
      pal.passives[passiveEditing] = code;
    }
  }
  function removePassive() {
    if (passiveEditing != null) pal.passives.splice(passiveEditing, 1);
  }

  // Moves: click or drag between/reorder the equipped and inactive zones.
  type MoveList = "active" | "bench";
  interface MoveDrag {
    code: string;
    list: MoveList;
    index: number;
  }

  let emptySlots = $derived(Math.max(0, LIMITS.equippedMovesMax - pal.activeSkills.length));
  let dragTarget = $state<{ list: MoveList; index: number } | null>(null);

  function isNaturalMove(code: string) {
    return resolveSpecies(pal.species)?.moves.includes(code) ?? false;
  }
  function rememberLearned(code: string) {
    if (!isNaturalMove(code) && !pal.learnedMoves.includes(code)) pal.learnedMoves.push(code);
  }

  function equip(code: string) {
    if (pal.activeSkills.includes(code)) return;
    rememberLearned(code);
    const i = pal.benchMoves.indexOf(code);
    if (i >= 0) pal.benchMoves.splice(i, 1);
    if (pal.activeSkills.length >= LIMITS.equippedMovesMax) {
      const dropped = pal.activeSkills.pop();
      if (dropped) pal.benchMoves.push(dropped); // swap the oldest out
    }
    pal.activeSkills.push(code);
  }
  function unequip(code: string) {
    const i = pal.activeSkills.indexOf(code);
    if (i < 0) return;
    pal.activeSkills.splice(i, 1);
    rememberLearned(code);
    if (!pal.benchMoves.includes(code)) pal.benchMoves.push(code);
  }
  function addMove(code: string) {
    rememberLearned(code);
    equip(code);
  }
  function onDragStart(e: DragEvent, list: MoveList, index: number) {
    const code = list === "active" ? pal.activeSkills[index] : pal.benchMoves[index];
    if (!code || !e.dataTransfer) return;
    const payload: MoveDrag = { code, list, index };
    e.dataTransfer.setData("application/x-palbox-move", JSON.stringify(payload));
    e.dataTransfer.setData("text/plain", code);
    e.dataTransfer.effectAllowed = "move";
  }
  function allowDrop(e: DragEvent, list: MoveList, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragTarget = { list, index };
  }
  function readDrag(e: DragEvent): MoveDrag | null {
    const raw = e.dataTransfer?.getData("application/x-palbox-move");
    if (!raw) return null;
    try {
      return JSON.parse(raw) as MoveDrag;
    } catch {
      return null;
    }
  }
  function moveSkill(source: MoveDrag, targetList: MoveList, rawTargetIndex: number) {
    const sourceItems = source.list === "active" ? pal.activeSkills : pal.benchMoves;
    let sourceIndex = sourceItems[source.index] === source.code
      ? source.index
      : sourceItems.indexOf(source.code);
    if (sourceIndex < 0) return;

    let targetIndex = rawTargetIndex;
    sourceItems.splice(sourceIndex, 1);
    if (source.list === targetList && sourceIndex < targetIndex) targetIndex -= 1;

    if (targetList === "active") {
      rememberLearned(source.code);
      targetIndex = Math.max(
        0,
        Math.min(
          source.list === "bench" && pal.activeSkills.length >= LIMITS.equippedMovesMax
            ? LIMITS.equippedMovesMax - 1
            : pal.activeSkills.length,
          targetIndex,
        ),
      );
      pal.activeSkills.splice(targetIndex, 0, source.code);
      if (pal.activeSkills.length > LIMITS.equippedMovesMax) {
        const displaced = pal.activeSkills.pop();
        if (displaced && displaced !== source.code && !pal.benchMoves.includes(displaced)) {
          rememberLearned(displaced);
          pal.benchMoves.push(displaced);
        }
      }
    } else {
      rememberLearned(source.code);
      targetIndex = Math.max(0, Math.min(pal.benchMoves.length, targetIndex));
      if (!pal.benchMoves.includes(source.code)) pal.benchMoves.splice(targetIndex, 0, source.code);
    }
  }
  function dropMove(e: DragEvent, list: MoveList, index: number) {
    e.preventDefault();
    e.stopPropagation();
    const source = readDrag(e);
    if (source) moveSkill(source, list, index);
    dragTarget = null;
  }
  function onMoveKey(e: KeyboardEvent, code: string, list: MoveList) {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    if (list === "active") unequip(code);
    else equip(code);
  }

  // Resolve move codes -> display info from moves.json.
  const displayElement = (el: string): ElementName =>
    el && el in ELEMENT_COLOR ? el as ElementName : "Neutral";
  const asMove = (code: string) => {
    const m = resolveMove(code);
    return { code, name: m?.name ?? code, element: m?.element ?? "", power: m?.power ?? 0 };
  };
  let equipped = $derived(pal.activeSkills.map(asMove));
  let bench = $derived(pal.benchMoves.map(asMove));

  // Real pal portrait from PalEdit's icons; fall back to the #ERROR placeholder.
  const iconSrc = $derived(empty ? "/logo.png" : palIcon(pal.species));
</script>

<div class="card" class:empty inert={empty} aria-disabled={empty}>
  <!-- Header -->
  <div class="head">
    <div class="idcol">
      <div class="nameline">
        <input class="name" bind:value={pal.name} spellcheck="false" aria-label="Pal name" />
        <span class="gender {pal.gender.toLowerCase()}">{genderSymbol}</span>
      </div>
      <div class="subline">
        {#each pal.elements as el}<ElementPill element={el} />{/each}
        <span class="pdx">Palpedia {pal.paldexNo}</span>
      </div>
    </div>
    <div class="headactions">
      <button class="species" onclick={() => (speciesOpen = true)} title="Change species">
        <span class="species-port"><img src={iconSrc} alt="" onerror={onPalIconError} /></span>
        <span class="species-copy">
          <span class="species-cap">SPECIES</span>
          <strong>{speciesName}</strong>
        </span>
        <span class="species-change">CHANGE</span>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none"><path d="M6 9l6 6 6-6" stroke="#c9b4e0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </button>
      <button class="variant alpha" class:on={pal.alpha} onclick={toggleAlpha} aria-pressed={pal.alpha} title="Toggle Alpha">
        <img src="/icons/variants/alpha.webp" alt="" />
        <span>Alpha</span>
      </button>
      <button class="variant lucky" class:on={pal.lucky} onclick={toggleLucky} aria-pressed={pal.lucky} title="Toggle Lucky">
        <img src="/icons/variants/lucky.webp" alt="" />
        <span>Lucky</span>
      </button>
    </div>
  </div>

  <!-- Body -->
  <div class="body">
    <!-- Left: partner + passives -->
    <div class="col left">
      <div>
        <SectionHeader title="PARTNER SKILL" />
        <div class="partner" style="--c:{ELEMENT_COLOR[pal.partnerSkill.element ?? pal.elements[0] ?? 'Neutral']}">
          <div class="pname">
            <ElementIcon element={pal.partnerSkill.element ?? pal.elements[0] ?? "Neutral"} size={20} decorative={false} />
            {pal.partnerSkill.name} <span class="lv">Lv {pal.partnerSkill.level}</span>
          </div>
          <p class="pdesc">{pal.partnerSkill.description}</p>
        </div>
      </div>
      <div class="passblock">
        <SectionHeader title="PASSIVE SKILLS">
          {#snippet right()}{pal.passives.length} / {LIMITS.passivesMax}{/snippet}
        </SectionHeader>
        <div class="passives">
          {#each pal.passives as code, index (code)}<PassiveChip {code} onselect={() => openPassive(index)} />{/each}
          {#if pal.passives.length < LIMITS.passivesMax}
            <button class="add" onclick={() => openPassive(null)}>+ Filter & add passive</button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Center: portrait + level + moves -->
    <div class="col center">
      <div class="portrait">
        <img class="badge alpha" class:hide={!pal.alpha} src="/icons/variants/alpha.webp" alt="Alpha" />
        <img class="badge lucky" class:hide={!pal.lucky} src="/icons/variants/lucky.webp" alt="Lucky" />
        <div class="art">
          <img class="palimg" src={iconSrc} alt={pal.name} onerror={onPalIconError} />
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
            <div class="soulval">+{soulTotal}</div>
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
        <div
          class="moveslots"
          role="group"
          aria-label="Equipped moves drop zone"
          class:dropzone={dragTarget?.list === "active"}
          ondragover={(event) => allowDrop(event, "active", pal.activeSkills.length)}
          ondragleave={() => (dragTarget = null)}
          ondrop={(event) => dropMove(event, "active", pal.activeSkills.length)}
        >
          {#each equipped as m, index (m.code)}
            <div
              class="move equipped"
              class:drop-target={dragTarget?.list === "active" && dragTarget.index === index}
              draggable="true"
              role="button"
              tabindex="0"
              aria-label="{m.name}, equipped. Click or drag to move."
              ondragstart={(event) => onDragStart(event, "active", index)}
              ondragover={(event) => allowDrop(event, "active", index)}
              ondrop={(event) => dropMove(event, "active", index)}
              ondragend={() => (dragTarget = null)}
              onclick={() => unequip(m.code)}
              onkeydown={(event) => onMoveKey(event, m.code, "active")}
              title="Click to unequip, or drag to reorder"
            >
              <span class="mgrip">⠿</span>
              <ElementIcon element={displayElement(m.element)} size={19} decorative={false} />
              <span class="mname">{m.name}</span>
              <span class="mpwrcap">PWR</span>
              <span class="mpwr">{m.power}</span>
            </div>
          {/each}
          {#each Array(emptySlots) as _, i (i)}
            <div
              class="emptyslot"
              role="group"
              aria-label="Empty equipped move slot"
              ondragover={(event) => allowDrop(event, "active", pal.activeSkills.length)}
              ondrop={(event) => dropMove(event, "active", pal.activeSkills.length)}
            >empty slot — drag a move here</div>
          {/each}
        </div>
        <div class="bench-head">
          <div class="bench-label">INACTIVE / AVAILABLE MOVES</div>
          <button class="browse-moves" onclick={() => (moveOpen = true)}>FILTER & ADD</button>
        </div>
        <div
          class="bench"
          role="group"
          aria-label="Inactive moves drop zone"
          class:dropzone={dragTarget?.list === "bench"}
          ondragover={(event) => allowDrop(event, "bench", pal.benchMoves.length)}
          ondragleave={() => (dragTarget = null)}
          ondrop={(event) => dropMove(event, "bench", pal.benchMoves.length)}
        >
          {#each bench as m, index (m.code)}
            <div
              class="move bench-move"
              class:drop-target={dragTarget?.list === "bench" && dragTarget.index === index}
              draggable="true"
              role="button"
              tabindex="0"
              aria-label="{m.name}, inactive. Click or drag to equip."
              ondragstart={(event) => onDragStart(event, "bench", index)}
              ondragover={(event) => allowDrop(event, "bench", index)}
              ondrop={(event) => dropMove(event, "bench", index)}
              ondragend={() => (dragTarget = null)}
              onclick={() => equip(m.code)}
              onkeydown={(event) => onMoveKey(event, m.code, "bench")}
              title="Click to equip, or drag to equip/reorder"
            >
              <span class="mgrip">⠿</span>
              <ElementIcon element={displayElement(m.element)} size={17} decorative={false} />
              <span class="mname">{m.name}</span>
              <span class="mpwrcap">PWR</span>
              <span class="mpwr">{m.power}</span>
            </div>
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
            <div class="brow">
              <span class="blabel">HP</span>
              <span class="bval hp">
                <input
                  class="stat-number hp"
                  type="number"
                  min="0"
                  max={hpMax}
                  value={pal.stats.hp}
                  oninput={(event) => setHp(event.currentTarget.valueAsNumber)}
                  aria-label="Current HP"
                />
                <span class="bmax">/{hpMax.toLocaleString()}</span>
              </span>
            </div>
            <div class="track">
              <div class="fill hp" style="width:{hpPct}%"></div>
              <input
                class="track-control"
                type="range"
                min="0"
                max={hpMax}
                value={pal.stats.hp}
                oninput={(event) => setHp(event.currentTarget.valueAsNumber)}
                aria-label="Current HP"
              />
            </div>
          </div>
          <div class="barstat">
            <div class="brow">
              <span class="blabel">SAN</span>
              <input
                class="stat-number"
                type="number"
                min="0"
                max="100"
                value={pal.stats.san}
                oninput={(event) => setSanity(event.currentTarget.valueAsNumber)}
                aria-label="Sanity"
              />
            </div>
            <div class="track thin">
              <div class="fill san" style="width:{pal.stats.san}%"></div>
              <input
                class="track-control"
                type="range"
                min="0"
                max="100"
                value={pal.stats.san}
                oninput={(event) => setSanity(event.currentTarget.valueAsNumber)}
                aria-label="Sanity"
              />
            </div>
          </div>
          <div class="barstat">
            <div class="brow">
              <span class="blabel">Food</span>
              <span class="bval food">
                <input
                  class="stat-number food"
                  type="number"
                  min="0"
                  max="100"
                  value={Math.round(pal.stats.foodPct * 100)}
                  oninput={(event) => setFoodPercent(event.currentTarget.valueAsNumber)}
                  aria-label="Food percent"
                />%
              </span>
            </div>
            <div class="track thin">
              <div class="fill food" style="width:{pal.stats.foodPct * 100}%"></div>
              <input
                class="track-control"
                type="range"
                min="0"
                max="100"
                value={Math.round(pal.stats.foodPct * 100)}
                oninput={(event) => setFoodPercent(event.currentTarget.valueAsNumber)}
                aria-label="Food percent"
              />
            </div>
          </div>
          <div class="barstat trust-block">
            <div class="brow">
              <span class="tlabel">TRUST</span>
              <label class="trust-progress">
                <input
                  class="stat-number trust-progress"
                  type="number"
                  min="0"
                  max="100"
                  disabled={pal.trust.rank >= 10}
                  value={Math.round(pal.trust.progress * 100)}
                  oninput={(event) => setTrustProgress(event.currentTarget.valueAsNumber)}
                  aria-label="Trust progress percent"
                />%
              </label>
            </div>
            <div class="track thin">
              <div class="fill trust" style="width:{pal.trust.progress * 100}%"></div>
              <input
                class="track-control"
                type="range"
                min="0"
                max="100"
                disabled={pal.trust.rank >= 10}
                value={Math.round(pal.trust.progress * 100)}
                oninput={(event) => setTrustProgress(event.currentTarget.valueAsNumber)}
                aria-label="Trust progress"
              />
            </div>
            <label class="trust-rank">
              <span>Rank</span>
              <input
                class="stat-number"
                type="number"
                min="0"
                max="10"
                value={pal.trust.rank}
                oninput={(event) => setTrustRank(event.currentTarget.valueAsNumber)}
                aria-label="Trust rank"
              />
              <span class="rank-max">/ 10</span>
            </label>
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

<SpeciesSelector bind:open={speciesOpen} current={pal.species} onpick={(code) => reSpecies(pal, code)} />
<PassiveSelector
  bind:open={passiveOpen}
  species={pal.species}
  selected={pal.passives}
  editing={passiveEditing}
  onpick={choosePassive}
  onremove={removePassive}
/>
<MoveSelector bind:open={moveOpen} species={pal.species} equipped={pal.activeSkills} onpick={addMove} />

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
  .card.empty .palimg {
    width: 96px; height: 96px; opacity: .72;
    filter: drop-shadow(0 0 20px rgba(176, 96, 224, .4));
  }
  .card.empty .species-port img { width: 24px; height: 24px; opacity: .7; }

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
  .species {
    display: inline-flex; align-items: center; gap: 9px;
    min-width: 220px; padding: 5px 10px 5px 6px; border-radius: 10px; cursor: pointer;
    color: #d6bef2; text-align: left;
    background: rgba(176, 96, 224, 0.13); border: 1px solid rgba(176, 96, 224, 0.48);
    box-shadow: inset 0 0 16px rgba(176, 96, 224, 0.05);
    transition: background 0.14s, border-color 0.14s, box-shadow 0.14s;
  }
  .species:hover { background: rgba(176, 96, 224, 0.23); border-color: rgba(176, 96, 224, 0.78); box-shadow: 0 0 14px rgba(176, 96, 224, 0.2); }
  .species-port { width: 34px; height: 34px; flex: none; display: grid; place-items: center; overflow: hidden; border-radius: 8px; background: rgba(8, 7, 12, 0.42); }
  .species-port img { width: 100%; height: 100%; object-fit: contain; }
  .species-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; line-height: 1.05; }
  .species-cap { color: #8f79a4; font: 600 9px var(--font-head); letter-spacing: 0.14em; }
  .species-copy strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #eadff4; font: 700 15px var(--font-cond); }
  .species-change { color: #bca5d1; font: 700 9.5px var(--font-head); letter-spacing: 0.08em; }
  .pdx { font-size: 12.5px; color: #6e7a86; margin-left: 4px; }
  .headactions { display: flex; align-items: center; gap: 10px; }
  .variant {
    width: 48px; height: 48px; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 0; cursor: pointer; border-radius: 10px; color: #83909a;
    background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.1);
  }
  .variant img { width: 25px; height: 25px; object-fit: contain; opacity: .42; filter: grayscale(.7); }
  .variant span { font: 600 9px var(--font-head); letter-spacing: .04em; }
  .variant:hover { border-color: rgba(255,255,255,.24); color: #b9c2c9; }
  .variant.alpha.on { color: #ffaaaa; border-color: rgba(255,70,70,.62); background: rgba(255,70,70,.12); box-shadow: 0 0 14px rgba(255,70,70,.2); }
  .variant.lucky.on { color: #9fddff; border-color: rgba(70,170,255,.62); background: rgba(70,170,255,.12); box-shadow: 0 0 14px rgba(70,170,255,.2); }
  .variant.on img { opacity: 1; filter: none; }
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

  /* Exp */
  .exp {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 11px 26px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

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
  .badge { position: absolute; z-index: 2; top: 10px; width: 42px; height: 42px; object-fit: contain; filter: drop-shadow(0 2px 5px rgba(0,0,0,.7)); }
  .badge.alpha { left: 10px; }
  .badge.lucky { right: 10px; width: 36px; height: 36px; }
  .badge.hide { display: none; }
  .art { position: absolute; inset: 0 0 52px; display: grid; place-items: center; }
  .palimg { max-width: 78%; max-height: 92%; object-fit: contain; filter: drop-shadow(0 6px 18px rgba(0, 0, 0, 0.5)); }
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
  .move:focus-visible { outline: 2px solid rgba(63, 199, 224, 0.72); outline-offset: 2px; }
  .move:hover { border-color: rgba(63, 199, 224, 0.5); }
  .move.drop-target { border-top-color: #8fe3f2; box-shadow: 0 -3px 0 rgba(63, 199, 224, 0.78); }
  .moveslots.dropzone, .bench.dropzone { box-shadow: inset 0 0 0 1px rgba(63, 199, 224, 0.28); }
  .move.equipped { background: rgba(63, 199, 224, 0.05); border-color: rgba(63, 199, 224, 0.18); }
  .emptyslot { display: flex; align-items: center; justify-content: center; padding: 11px; border-radius: 9px; border: 1px dashed rgba(255, 255, 255, 0.14); color: #6e7a86; font-size: 13px; }
  .bench-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 12px 2px 7px; }
  .bench-label { font-family: var(--font-head); font-weight: 600; font-size: 11.5px; letter-spacing: 0.14em; color: #6e7a86; }
  .browse-moves {
    padding: 5px 9px; border-radius: 8px; cursor: pointer; color: #9fd8e6;
    background: rgba(63,199,224,.09); border: 1px solid rgba(63,199,224,.3);
    font: 700 9.5px var(--font-head); letter-spacing: .08em;
  }
  .browse-moves:hover { background: rgba(63,199,224,.17); border-color: rgba(63,199,224,.52); }
  .bench { display: flex; flex-direction: column; gap: 7px; }
  .bench-move { padding: 9px 12px; }
  .mgrip { color: #7c8894; font-size: 15px; letter-spacing: -2px; }
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
  .track { position: relative; height: 8px; border-radius: 5px; background: rgba(255, 255, 255, 0.06); }
  .track.thin { height: 7px; }
  .track.thick { height: 9px; flex: 1; }
  .fill { height: 100%; border-radius: inherit; overflow: hidden; pointer-events: none; }
  .track-control {
    position: absolute; z-index: 1; inset: -7px 0; width: 100%; height: calc(100% + 14px);
    margin: 0; cursor: ew-resize; opacity: 0;
  }
  .fill.hp { background: linear-gradient(90deg, #5fd16a, #84e08d); box-shadow: 0 0 8px rgba(95, 209, 106, 0.5); }
  .fill.san { background: linear-gradient(90deg, #3fc7e0, #7fe0f2); }
  .fill.food { background: linear-gradient(90deg, #e8963a, #f2b06a); }
  .fill.trust { background: linear-gradient(90deg, #b060e0, #d89af0); box-shadow: 0 0 10px rgba(176, 96, 224, 0.6); }
  .valrow { display: flex; align-items: center; justify-content: space-between; font-size: 14px; }
  .stat-number {
    width: 48px; padding: 1px 4px; color: #dce5eb; text-align: right;
    font: 600 14px var(--font-head); font-variant-numeric: tabular-nums;
    border: 1px solid rgba(255,255,255,.12); border-radius: 5px;
    outline: 0; background: rgba(255,255,255,.045);
  }
  .stat-number:focus { border-color: rgba(63,199,224,.65); background: rgba(63,199,224,.08); }
  .stat-number.hp { width: 76px; color: #cfebd2; }
  .stat-number.food { color: #f0c39a; }
  .stat-number::-webkit-inner-spin-button, .stat-number::-webkit-outer-spin-button {
    -webkit-appearance: none; margin: 0;
  }
  .trust-block { gap: 4px; margin-top: 2px; }
  .tlabel { font-family: var(--font-head); font-weight: 600; font-size: 12.5px; letter-spacing: 0.14em; color: #9aa6b2; }
  .trust-progress { display: inline-flex; align-items: center; color: #c9b4e0; font-size: 11px; }
  .stat-number.trust-progress { width: 44px; color: #d8c4ee; }
  .stat-number.trust-progress:disabled { cursor: default; opacity: .55; }
  .trust-rank {
    display: flex; align-items: center; justify-content: flex-end; gap: 6px;
    margin-top: 4px; color: #8f819b; font-size: 11px;
  }
  .trust-rank .stat-number { width: 38px; color: #d8c4ee; }
  .rank-max { color: #655a70; font-variant-numeric: tabular-nums; }
  .track-control:disabled { cursor: default; }

  .worksuit { display: flex; flex-direction: column; gap: 7px; }
</style>
