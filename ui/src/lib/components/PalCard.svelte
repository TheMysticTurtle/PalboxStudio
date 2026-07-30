<script lang="ts">
  import type { Pal, ElementName } from "$lib/data/types";
  import {
    activeSkillDefaultControl,
    LIMITS,
    soulBonusPercent,
  } from "$lib/data/constants";
  import {
    moveSkill,
    type MoveDrag,
    type MoveList,
  } from "$lib/data/moveSlots";
  import { ref, resolveMove, resolveSpecies } from "$lib/data/refdata.svelte";
  import { APP_LOGO_ART, palIcon, variantIcon } from "$lib/data/icons";
  import {
    elementColor,
    nextGender,
    normalizeElement,
    presentBoxPal,
  } from "$lib/data/palPresentation";
  import { maxHpForPal, palToBoxPal } from "$lib/data/mapper";
  import { changeSelectedSpecies } from "$lib/stores/box.svelte";
  import SectionHeader from "./SectionHeader.svelte";
  import ElementPill from "./ElementPill.svelte";
  import GenderIcon from "./GenderIcon.svelte";
  import PassiveChip from "./PassiveChip.svelte";
  import WorkSuitRow from "./WorkSuitRow.svelte";
  import SpeciesSelector from "./SpeciesSelector.svelte";
  import PassiveSelector from "./PassiveSelector.svelte";
  import MoveSelector from "./MoveSelector.svelte";
  import PalArtwork from "./PalArtwork.svelte";
  import ElementIcon from "./ElementIcon.svelte";
  import GroupTags from "./GroupTags.svelte";
  import PassivePresets from "./PassivePresets.svelte";

  let { pal, empty = false }: { pal: Pal; empty?: boolean } = $props();

  // Species display name (tolerant of an alpha BOSS_ prefix) + the selector modal.
  let speciesOpen = $state(false);
  let passiveOpen = $state(false);
  let passiveEditing = $state<number | null>(null);
  let moveOpen = $state(false);
  const speciesName = $derived(empty ? "" : (resolveSpecies(pal.species)?.name ?? pal.species));

  const cardView = $derived(presentBoxPal(palToBoxPal(pal, -1)));
  const hpMax = $derived(empty ? 0 : maxHpForPal(pal));
  const hpPct = $derived(hpMax > 0 ? Math.min(100, (pal.stats.hp / hpMax) * 100) : 0);
  const sanityPct = $derived(
    ref.limits.sanityMax > ref.limits.sanityMin
      ? Math.max(
          0,
          Math.min(
            100,
            ((pal.stats.san - ref.limits.sanityMin)
              / (ref.limits.sanityMax - ref.limits.sanityMin)) * 100,
          ),
        )
      : 0,
  );
  const trustRankMin = $derived(pal.trust.minRank);
  const trustRankMax = $derived(pal.trust.maxRank);

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
    pal.stats.san = Math.round(
      Math.max(
        ref.limits.sanityMin,
        Math.min(ref.limits.sanityMax, finiteOr(value, pal.stats.san)),
      ),
    );
  }
  function setFoodPercent(value: number) {
    const percent = Math.max(0, Math.min(100, finiteOr(value, pal.stats.foodPct * 100)));
    pal.stats.foodPct = percent / 100;
  }
  function setTrustRank(value: number) {
    pal.trust.rank = Math.round(
      Math.max(
        trustRankMin,
        Math.min(trustRankMax, finiteOr(value, pal.trust.rank)),
      ),
    );
    if (pal.trust.rank === trustRankMax) pal.trust.progress = 1;
  }
  function setTrustProgress(value: number) {
    if (pal.trust.rank >= trustRankMax) {
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
  function toggleGender() {
    pal.gender = nextGender(pal.gender);
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
  function applyPassiveCodes(codes: string[]) {
    if (empty) return;
    pal.passives.splice(0, pal.passives.length, ...codes.slice(0, LIMITS.passivesMax));
  }

  // Moves: click or drag between/reorder the equipped and inactive zones.
  let emptySlots = $derived(Math.max(0, LIMITS.equippedMovesMax - pal.activeSkills.length));
  let dragTarget = $state<{ list: MoveList; index: number } | null>(null);
  let dragSource = $state<MoveDrag | null>(null);
  let suppressMoveClick = $state(false);
  let pointerDrag = $state<{
    pointerId: number;
    startX: number;
    startY: number;
    source: MoveDrag;
    engaged: boolean;
  } | null>(null);

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
  function beginPointerDrag(e: PointerEvent, list: MoveList, index: number) {
    if (e.button !== 0) return;
    const code = list === "active" ? pal.activeSkills[index] : pal.benchMoves[index];
    if (!code) return;
    pointerDrag = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      source: { code, list, index },
      engaged: false,
    };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function pointerDropTarget(e: PointerEvent) {
    const target = document
      .elementFromPoint(e.clientX, e.clientY)
      ?.closest<HTMLElement>("[data-move-drop-list]");
    const list = target?.dataset.moveDropList;
    const index = Number(target?.dataset.moveDropIndex);
    dragTarget = (
      (list === "active" || list === "bench")
      && Number.isInteger(index)
    )
      ? { list, index }
      : null;
  }

  function movePointerDrag(e: PointerEvent) {
    if (!pointerDrag || e.pointerId !== pointerDrag.pointerId) return;
    if (!pointerDrag.engaged) {
      const distance = Math.hypot(
        e.clientX - pointerDrag.startX,
        e.clientY - pointerDrag.startY,
      );
      if (distance < 5) return;
      pointerDrag.engaged = true;
      dragSource = pointerDrag.source;
      suppressMoveClick = true;
    }
    e.preventDefault();
    pointerDropTarget(e);
  }

  function applyMove(source: MoveDrag, targetList: MoveList, targetIndex: number) {
    const result = moveSkill(
      { active: pal.activeSkills, bench: pal.benchMoves },
      source,
      targetList,
      targetIndex,
      LIMITS.equippedMovesMax,
    );
    if (!result.moved) return;
    rememberLearned(source.code);
    if (result.displaced) rememberLearned(result.displaced);
    pal.activeSkills = result.active;
    pal.benchMoves = result.bench;
  }
  function finishDrag() {
    dragSource = null;
    dragTarget = null;
    window.setTimeout(() => (suppressMoveClick = false), 0);
  }

  function endPointerDrag(e: PointerEvent, apply: boolean) {
    if (!pointerDrag || e.pointerId !== pointerDrag.pointerId) return;
    const { source, engaged } = pointerDrag;
    if (engaged) {
      e.preventDefault();
      pointerDropTarget(e);
      if (apply && dragTarget) applyMove(source, dragTarget.list, dragTarget.index);
      finishDrag();
    }
    pointerDrag = null;
    const element = e.currentTarget as HTMLElement;
    if (element.hasPointerCapture(e.pointerId)) element.releasePointerCapture(e.pointerId);
  }
  function onMoveClick(code: string, list: MoveList) {
    if (suppressMoveClick) return;
    if (list === "active") unequip(code);
    else equip(code);
  }
  function onMoveKey(e: KeyboardEvent, code: string, list: MoveList) {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    if (list === "active") unequip(code);
    else equip(code);
  }

  // Resolve move codes -> display info from moves.json.
  const displayElement = (el: string): ElementName => normalizeElement(el);
  const asMove = (code: string) => {
    const m = resolveMove(code);
    return { code, name: m?.name ?? code, element: m?.element ?? "", power: m?.power ?? 0 };
  };
  let equipped = $derived(pal.activeSkills.map(asMove));
  let bench = $derived(pal.benchMoves.map(asMove));

  // Real pal portrait from PalEdit's icons; fall back to the #ERROR placeholder.
  const iconSrc = $derived(empty ? APP_LOGO_ART : palIcon(pal.species));
</script>

<div
  class="card"
  class:empty
  inert={empty}
  aria-disabled={empty}
  style="--primary:{cardView.primaryColor}; --secondary:{cardView.secondaryColor}"
>
  <section class="hero">
    <div class="visual-card">
      <div class="visual-grid"></div>
      <div class="visual-orbit one"></div>
      <div class="visual-orbit two"></div>
      <img class="badge alpha" class:hide={!pal.alpha} src={variantIcon("alpha")} alt="Alpha" />
      <img class="badge lucky" class:hide={!pal.lucky} src={variantIcon("lucky")} alt="Lucky" />
      <div class="art">
        <div class="art-shell">
          <PalArtwork src={iconSrc} alt={pal.name} zoom={empty ? 1 : 1.04} lazy={false} />
        </div>
      </div>
      <div class="visual-foot">
        <div>
          <div class="overline">CONDENSATION</div>
          <div class="stars">
            {#each Array(LIMITS.condensationMax) as _, i}<span class:on={i < pal.condensation}>★</span>{/each}
          </div>
        </div>
        <div class="soul-summary">
          <div class="overline">PAL SOULS</div>
          <div class="soul-values">
            <span title="HP soul enhancement">H {soulBonusPercent(pal.soulRanks.hp)}%</span>
            <span title="Attack soul enhancement">A {soulBonusPercent(pal.soulRanks.attack)}%</span>
            <span title="Defense soul enhancement">D {soulBonusPercent(pal.soulRanks.defense)}%</span>
            <span title="Work Speed soul enhancement">W {soulBonusPercent(pal.soulRanks.craftSpeed)}%</span>
          </div>
        </div>
      </div>
    </div>

    <div class="hero-copy">
      <div class="hero-toolbar">
        <button class="species" onclick={() => (speciesOpen = true)} title="Change species">
          <span class="species-copy">
            <span class="species-cap">SPECIES</span>
            <strong>{speciesName}</strong>
          </span>
          <span class="species-change">CHANGE</span>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <div class="variant-controls">
          <button class="variant alpha" class:on={pal.alpha} onclick={toggleAlpha} aria-pressed={pal.alpha} title="Toggle Alpha">
            <img src={variantIcon("alpha")} alt="" />
            <span>Alpha</span>
          </button>
          <button class="variant lucky" class:on={pal.lucky} onclick={toggleLucky} aria-pressed={pal.lucky} title="Toggle Lucky">
            <img src={variantIcon("lucky")} alt="" />
            <span>Lucky</span>
          </button>
        </div>
      </div>

      <div class="identity">
        <div class="paldex">{empty ? "" : `PALPEDIA ${pal.paldexNo}`}</div>
        <div class="nameline">
          <input class="name" bind:value={pal.name} spellcheck="false" aria-label="Pal name" />
          <button
            type="button"
            class="gender {pal.gender.toLowerCase()}"
            onclick={toggleGender}
            title="Change gender to {nextGender(pal.gender)}"
            aria-label="Gender: {pal.gender}. Change to {nextGender(pal.gender)}"
          >
            <GenderIcon gender={pal.gender} size={25} />
            <span class="gender-label">{pal.gender}</span>
          </button>
        </div>
        <div class="elements">
          {#each pal.elements as el}<ElementPill element={el} />{/each}
        </div>
      </div>

      <div class="level-and-stats">
        <div class="level">
          <button class="lvbtn" onclick={() => setLevel(pal.level - 1)} aria-label="Lower level">−</button>
          <label class="lvbox">
            <span class="lvcap">LEVEL</span>
            <input class="lvnum" inputmode="numeric" value={pal.level} onchange={(e) => setLevel(+e.currentTarget.value)} aria-label="Level" />
          </label>
          <button class="lvbtn" onclick={() => setLevel(pal.level + 1)} aria-label="Raise level">+</button>
        </div>
        <div class="combat-stats">
          <div class="combat-stat hp-stat">
            <span class="stat-glyph">♥</span>
            <span><small>MAX HP</small><strong>{cardView.stats.hp.toLocaleString()}</strong></span>
          </div>
          <div class="combat-stat attack-stat">
            <span class="stat-glyph">⚔</span>
            <span><small>ATTACK</small><strong>{cardView.stats.attack.toLocaleString()}</strong></span>
          </div>
          <div class="combat-stat defense-stat">
            <span class="stat-glyph">⬢</span>
            <span><small>DEFENSE</small><strong>{cardView.stats.defense.toLocaleString()}</strong></span>
          </div>
        </div>
      </div>

      <div class="vitals">
        <div class="vital hp-vital">
          <div class="vital-head">
            <span>HP</span>
            <span class="vital-value">
              <input
                class="stat-number hp"
                type="number"
                min="0"
                max={hpMax}
                value={pal.stats.hp}
                oninput={(event) => setHp(event.currentTarget.valueAsNumber)}
                aria-label="Current HP"
              />
              <span>/{hpMax.toLocaleString()}</span>
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

        <div class="vital">
          <div class="vital-head">
            <span>SAN</span>
            <input
              class="stat-number"
              type="number"
              min={ref.limits.sanityMin}
              max={ref.limits.sanityMax}
              value={pal.stats.san}
              oninput={(event) => setSanity(event.currentTarget.valueAsNumber)}
              aria-label="Sanity"
            />
          </div>
          <div class="track">
            <div class="fill san" style="width:{sanityPct}%"></div>
            <input
              class="track-control"
              type="range"
              min={ref.limits.sanityMin}
              max={ref.limits.sanityMax}
              value={pal.stats.san}
              oninput={(event) => setSanity(event.currentTarget.valueAsNumber)}
              aria-label="Sanity"
            />
          </div>
        </div>

        <div class="vital">
          <div class="vital-head">
            <span>FOOD</span>
            <span class="vital-value">
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
          <div class="track">
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

        <div class="vital trust-vital">
          <div class="vital-head">
            <span>TRUST</span>
            <span class="vital-value">
              <input
                class="stat-number trust-progress"
                type="number"
                min="0"
                max="100"
                disabled={pal.trust.rank >= trustRankMax}
                value={Math.round(pal.trust.progress * 100)}
                oninput={(event) => setTrustProgress(event.currentTarget.valueAsNumber)}
                aria-label="Trust progress percent"
              />%
            </span>
          </div>
          <div class="track">
            <div class="fill trust" style="width:{pal.trust.progress * 100}%"></div>
            <input
              class="track-control"
              type="range"
              min="0"
              max="100"
              disabled={pal.trust.rank >= trustRankMax}
              value={Math.round(pal.trust.progress * 100)}
              oninput={(event) => setTrustProgress(event.currentTarget.valueAsNumber)}
              aria-label="Trust progress"
            />
          </div>
          <label class="trust-rank">
            <span>RANK</span>
            <input
              class="stat-number"
              type="number"
              min={trustRankMin}
              max={trustRankMax}
              value={pal.trust.rank}
              oninput={(event) => setTrustRank(event.currentTarget.valueAsNumber)}
              aria-label="Trust rank"
            />
            <span>/ {trustRankMax}</span>
          </label>
        </div>
      </div>
    </div>
  </section>

  <section class="editor-grid">
    <div class="editor-panel traits-panel">
      <div>
        <SectionHeader title="PARTNER SKILL" />
        <div class="partner" style="--c:{elementColor(pal.partnerSkill.element ?? pal.elements[0] ?? 'Neutral')}">
          <div class="pname">
            <ElementIcon element={pal.partnerSkill.element ?? pal.elements[0] ?? "Neutral"} size={23} decorative={false} />
            {pal.partnerSkill.name} <span class="lv">Lv {pal.partnerSkill.level}</span>
          </div>
          <p class="pdesc">{pal.partnerSkill.description}</p>
          {#if pal.partnerSkill.rankEffect}
            <p class="prank">{pal.partnerSkill.rankEffect}</p>
          {/if}
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
      <div class="preset-block">
        <SectionHeader title="PASSIVE PRESETS" />
        <PassivePresets disabled={empty} onapply={applyPassiveCodes} />
      </div>
      <div class="group-block">
        <SectionHeader title="GROUP TAGS" />
        <GroupTags instanceId={pal.instanceId} disabled={empty} />
      </div>
    </div>

    <div class="editor-panel moves-panel">
      <SectionHeader title="ACTIVE SKILLS">
        {#snippet right()}tap or drag · {pal.activeSkills.length} / {LIMITS.equippedMovesMax}{/snippet}
      </SectionHeader>
      <div
        class="moveslots"
        role="group"
        aria-label="Equipped moves drop zone"
        class:dropzone={dragTarget?.list === "active"}
        data-move-drop-list="active"
        data-move-drop-index={pal.activeSkills.length}
      >
        {#each equipped as m, index (m.code)}
          <div
            class="move equipped"
            class:dragging={dragSource?.list === "active" && dragSource.index === index}
            class:drop-target={dragTarget?.list === "active" && dragTarget.index === index}
            draggable="false"
            data-move-code={m.code}
            data-move-list="active"
            data-move-index={index}
            data-move-drop-list="active"
            data-move-drop-index={index}
            role="button"
            tabindex="0"
            aria-label="{m.name}, equipped. Click or drag to move."
            onpointerdown={(event) => beginPointerDrag(event, "active", index)}
            onpointermove={movePointerDrag}
            onpointerup={(event) => endPointerDrag(event, true)}
            onpointercancel={(event) => endPointerDrag(event, false)}
            onclick={() => onMoveClick(m.code, "active")}
            onkeydown={(event) => onMoveKey(event, m.code, "active")}
            title="Click to unequip, or drag to reorder"
          >
            <span
              class="slot-control"
              title={`Default mounted control: ${activeSkillDefaultControl(index).label} (${activeSkillDefaultControl(index).action})`}
            >{activeSkillDefaultControl(index).short}</span>
            <span class="mgrip">⠿</span>
            <ElementIcon element={displayElement(m.element)} size={22} decorative={false} />
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
            data-move-drop-list="active"
            data-move-drop-index={pal.activeSkills.length}
          >
            <span
              class="slot-control"
              title={`Default mounted control: ${activeSkillDefaultControl(pal.activeSkills.length + i).label} (${activeSkillDefaultControl(pal.activeSkills.length + i).action})`}
            >{activeSkillDefaultControl(pal.activeSkills.length + i).short}</span>
            <span>empty slot — drag a move here</span>
          </div>
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
        data-move-drop-list="bench"
        data-move-drop-index={pal.benchMoves.length}
      >
        {#each bench as m, index (m.code)}
          <div
            class="move bench-move"
            class:dragging={dragSource?.list === "bench" && dragSource.index === index}
            class:drop-target={dragTarget?.list === "bench" && dragTarget.index === index}
            draggable="false"
            data-move-code={m.code}
            data-move-list="bench"
            data-move-index={index}
            data-move-drop-list="bench"
            data-move-drop-index={index}
            role="button"
            tabindex="0"
            aria-label="{m.name}, inactive. Click or drag to equip."
            onpointerdown={(event) => beginPointerDrag(event, "bench", index)}
            onpointermove={movePointerDrag}
            onpointerup={(event) => endPointerDrag(event, true)}
            onpointercancel={(event) => endPointerDrag(event, false)}
            onclick={() => onMoveClick(m.code, "bench")}
            onkeydown={(event) => onMoveKey(event, m.code, "bench")}
            title="Click to equip, or drag to equip/reorder"
          >
            <span class="mgrip">⠿</span>
            <ElementIcon element={displayElement(m.element)} size={20} decorative={false} />
            <span class="mname">{m.name}</span>
            <span class="mpwrcap">PWR</span>
            <span class="mpwr">{m.power}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="editor-panel work-panel">
      <SectionHeader title="WORK SUITABILITY" />
      <div class="worksuit">
        {#each pal.workSuit as s (s.name)}<WorkSuitRow suit={s} />{/each}
        {#if !empty && !pal.workSuit.length}<div class="none">None</div>{/if}
      </div>
    </div>
  </section>
</div>

<SpeciesSelector
  bind:open={speciesOpen}
  current={pal.species}
  onpick={(code) => void changeSelectedSpecies(code)}
/>
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
  .gender {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 82px;
    height: 38px;
    gap: 6px;
    padding: 0 10px 0 8px;
    border-radius: 11px;
    cursor: pointer;
    transition: filter 0.14s, transform 0.14s;
  }
  .gender:hover { filter: brightness(1.2); transform: scale(1.06); }
  .gender:focus-visible { outline: 2px solid rgba(143, 227, 242, 0.7); outline-offset: 2px; }
  .gender-label {
    font: 700 var(--type-caption) var(--font-head);
    letter-spacing: .04em;
  }
  .gender.male { background: rgba(63, 143, 224, 0.18); border: 1px solid rgba(63, 143, 224, 0.55); color: #8fbef2; }
  .gender.female { background: rgba(224, 95, 192, 0.18); border: 1px solid rgba(224, 95, 192, 0.55); color: #f2a0d8; }
  .gender.unknown { background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.14); color: #9aa6b2; }
  .species {
    display: inline-flex; align-items: center; gap: 9px;
    min-width: 220px; padding: 5px 10px 5px 6px; border-radius: 10px; cursor: pointer;
    color: #d6bef2; text-align: left;
    background: rgba(176, 96, 224, 0.13); border: 1px solid rgba(176, 96, 224, 0.48);
    box-shadow: inset 0 0 16px rgba(176, 96, 224, 0.05);
    transition: background 0.14s, border-color 0.14s, box-shadow 0.14s;
  }
  .species:hover { background: rgba(176, 96, 224, 0.23); border-color: rgba(176, 96, 224, 0.78); box-shadow: 0 0 14px rgba(176, 96, 224, 0.2); }
  .species-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; line-height: 1.05; }
  .species-cap { color: #a18caf; font: 600 var(--type-micro) var(--font-head); letter-spacing: 0.14em; }
  .species-copy strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #eadff4; font: 700 15px var(--font-cond); }
  .species-change { color: #c7b0db; font: 700 var(--type-micro) var(--font-head); letter-spacing: 0.08em; }
  .variant {
    width: 48px; height: 48px; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 0; cursor: pointer; border-radius: 10px; color: #83909a;
    background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.1);
  }
  .variant img { width: 25px; height: 25px; object-fit: contain; opacity: .42; filter: grayscale(.7); }
  .variant span { font: 600 var(--type-micro) var(--font-head); letter-spacing: .04em; }
  .variant:hover { border-color: rgba(255,255,255,.24); color: #b9c2c9; }
  .variant.alpha.on { color: #ffaaaa; border-color: rgba(255,70,70,.62); background: rgba(255,70,70,.12); box-shadow: 0 0 14px rgba(255,70,70,.2); }
  .variant.lucky.on { color: #9fddff; border-color: rgba(70,170,255,.62); background: rgba(70,170,255,.12); box-shadow: 0 0 14px rgba(70,170,255,.2); }
  .variant.on img { opacity: 1; filter: none; }
  /* Partner + passives */
  .partner { padding: 15px; border-radius: 11px; background: color-mix(in srgb, var(--c) 6%, transparent); border: 1px solid color-mix(in srgb, var(--c) 22%, transparent); }
  .pname { display: flex; align-items: center; gap: 8px; font-family: var(--font-head); font-weight: 700; font-size: 18px; color: #f3e4da; }
  .pname .lv { color: var(--text-muted); font-weight: 400; font-size: 13px; font-family: var(--font-body); }
  .pdesc { margin: 8px 0 0; font-size: 13px; line-height: 1.55; color: #b4a79c; }
  .prank { margin: 7px 0 0; color: color-mix(in srgb, var(--c) 78%, #fff); font: 600 var(--type-label) var(--font-head); }
  .passblock { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .passives { display: flex; flex-direction: column; gap: 10px; }
  .add { display: flex; align-items: center; justify-content: center; padding: 12px; border-radius: 10px; border: 1px dashed rgba(255, 255, 255, 0.16); background: transparent; color: #7c8894; cursor: pointer; font-size: 13.5px; }
  .add:hover { border-color: rgba(63, 199, 224, 0.5); color: #9fd8e6; }

  /* Portrait */
  .badge { position: absolute; z-index: 2; top: 10px; width: 42px; height: 42px; object-fit: contain; filter: drop-shadow(0 2px 5px rgba(0,0,0,.7)); }
  .badge.alpha { left: 10px; }
  .badge.lucky { right: 10px; width: 36px; height: 36px; }
  .badge.hide { display: none; }
  .art { position: absolute; inset: 0 0 52px; display: grid; place-items: center; }
  .stars { font-size: 18px; letter-spacing: 2px; color: rgba(255, 255, 255, 0.22); }
  .stars .on { color: var(--accent-amber); }

  /* Level */
  .level { flex: none; display: flex; align-items: center; justify-content: center; gap: 18px; }
  .lvbtn { width: 46px; height: 46px; border-radius: 12px; border: 1px solid rgba(63, 199, 224, 0.4); background: rgba(63, 199, 224, 0.12); color: #9fd8e6; cursor: pointer; font-size: 26px; line-height: 1; }
  .lvbtn:hover { background: rgba(63, 199, 224, 0.22); }
  .lvbox { text-align: center; min-width: 150px; }
  .lvcap { font-family: var(--font-head); font-weight: 600; font-size: 12px; letter-spacing: 0.24em; color: #8fe3f2; }
  .lvnum { width: 150px; text-align: center; font-family: var(--font-head); font-weight: 700; font-size: 46px; line-height: 1.05; color: #eafbff; background: transparent; border: 0; outline: none; border-bottom: 1px dashed rgba(63, 199, 224, 0.28); }
  .lvnum::-webkit-inner-spin-button, .lvnum::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }

  /* Moves */
  .moveslots { display: flex; flex-direction: column; gap: 8px; padding: 10px; border-radius: 12px; background: rgba(63, 199, 224, 0.06); border: 1px solid rgba(63, 199, 224, 0.22); }
  .move { display: flex; align-items: center; gap: 11px; width: 100%; text-align: left; padding: 10px 12px; border-radius: 9px; background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.08); color: inherit; font: inherit; cursor: grab; touch-action: none; user-select: none; }
  .move:active { cursor: grabbing; }
  .move:focus-visible { outline: 2px solid rgba(63, 199, 224, 0.72); outline-offset: 2px; }
  .move:hover { border-color: rgba(63, 199, 224, 0.5); }
  .move.dragging { opacity: 0.5; }
  .move.drop-target { border-top-color: #8fe3f2; box-shadow: 0 -3px 0 rgba(63, 199, 224, 0.78); }
  .moveslots.dropzone, .bench.dropzone { box-shadow: inset 0 0 0 1px rgba(63, 199, 224, 0.28); }
  .move.equipped { background: rgba(63, 199, 224, 0.05); border-color: rgba(63, 199, 224, 0.18); }
  .slot-control {
    min-width: 42px;
    height: 27px;
    display: inline-grid;
    place-items: center;
    flex: none;
    padding: 0 5px;
    border-radius: 6px;
    color: #b9f2fb;
    background: rgba(63, 199, 224, 0.11);
    border: 1px solid rgba(63, 199, 224, 0.3);
    font: 700 var(--type-label) var(--font-head);
    letter-spacing: 0.04em;
  }
  .emptyslot { display: flex; align-items: center; justify-content: flex-start; gap: 10px; padding: 11px; border-radius: 9px; border: 1px dashed rgba(255, 255, 255, 0.14); color: #6e7a86; font-size: 13px; }
  .bench-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 12px 2px 7px; }
  .bench-label { font-family: var(--font-head); font-weight: 600; font-size: var(--type-caption); letter-spacing: 0.14em; color: #84909b; }
  .browse-moves {
    padding: 5px 9px; border-radius: 8px; cursor: pointer; color: #9fd8e6;
    background: rgba(63,199,224,.09); border: 1px solid rgba(63,199,224,.3);
    font: 700 var(--type-micro) var(--font-head); letter-spacing: .08em;
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

  /* Editable vitals */
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
  .trust-progress { display: inline-flex; align-items: center; color: #c9b4e0; font-size: var(--type-label); }
  .stat-number.trust-progress { width: 44px; color: #d8c4ee; }
  .stat-number.trust-progress:disabled { cursor: default; opacity: .55; }
  .trust-rank {
    display: flex; align-items: center; justify-content: flex-end; gap: 6px;
    margin-top: 4px; color: #9c8ba9; font-size: var(--type-label);
  }
  .trust-rank .stat-number { width: 38px; color: #d8c4ee; }
  .rank-max { color: #655a70; font-variant-numeric: tabular-nums; }
  .track-control:disabled { cursor: default; }

  .worksuit { display: flex; flex-direction: column; gap: 7px; }

  /* Main editor card — one shared, data-driven hero above the editing workspace. */
  .card {
    position: relative;
    background:
      radial-gradient(70% 85% at 12% 12%, color-mix(in srgb, var(--primary) 13%, transparent), transparent 64%),
      radial-gradient(70% 85% at 88% 16%, color-mix(in srgb, var(--secondary) 11%, transparent), transparent 65%),
      linear-gradient(155deg, rgba(21, 27, 38, 0.98), rgba(10, 13, 20, 0.99));
    box-shadow: inset 0 1px rgba(255, 255, 255, 0.035);
  }
  .card::before {
    content: "";
    position: absolute;
    z-index: 0;
    inset: 0;
    pointer-events: none;
    opacity: 0.23;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.018) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.018) 1px, transparent 1px);
    background-size: 32px 32px;
    mask-image: linear-gradient(180deg, #000, transparent 72%);
  }
  .hero,
  .editor-grid { position: relative; z-index: 1; }

  .hero {
    flex: 0 0 clamp(355px, 39vh, 390px);
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(300px, 38%) minmax(0, 1fr);
    gap: 24px;
    padding: 20px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  }

  .visual-card {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--primary) 48%, var(--secondary));
    border-radius: 18px;
    background:
      radial-gradient(circle at 22% 48%, color-mix(in srgb, var(--primary) 28%, transparent), transparent 48%),
      radial-gradient(circle at 78% 48%, color-mix(in srgb, var(--secondary) 25%, transparent), transparent 52%),
      linear-gradient(145deg, rgba(15, 20, 29, 0.82), rgba(7, 9, 15, 0.96));
    box-shadow:
      inset 0 0 46px rgba(0, 0, 0, 0.48),
      0 0 24px color-mix(in srgb, var(--primary) 17%, transparent);
  }
  .visual-card::before {
    content: "";
    position: absolute;
    inset: 14px;
    border: 1px solid color-mix(in srgb, var(--secondary) 35%, rgba(255, 255, 255, 0.1));
    border-radius: 13px;
    box-shadow:
      inset 0 0 22px color-mix(in srgb, var(--primary) 14%, transparent),
      0 0 18px color-mix(in srgb, var(--secondary) 12%, transparent);
  }
  .visual-grid {
    position: absolute;
    inset: 0;
    opacity: 0.16;
    background-image:
      linear-gradient(color-mix(in srgb, var(--primary) 35%, transparent) 1px, transparent 1px),
      linear-gradient(90deg, color-mix(in srgb, var(--secondary) 35%, transparent) 1px, transparent 1px);
    background-size: 30px 30px;
    mask-image: radial-gradient(circle at center, #000, transparent 72%);
  }
  .visual-orbit {
    position: absolute;
    border: 1px solid color-mix(in srgb, var(--primary) 30%, transparent);
    border-radius: 50%;
    opacity: 0.5;
  }
  .visual-orbit.one { width: 250px; height: 250px; left: calc(50% - 125px); top: calc(50% - 135px); }
  .visual-orbit.two { width: 190px; height: 190px; left: calc(50% - 95px); top: calc(50% - 105px); border-color: color-mix(in srgb, var(--secondary) 32%, transparent); }
  .visual-card .art { inset: 8px 8px 61px; }
  .art-shell {
    position: relative;
    z-index: 1;
    width: min(70%, 250px);
    aspect-ratio: 1;
    transform: translateY(5px);
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--primary) 38%, rgba(255, 255, 255, 0.12));
    border-radius: 19px;
    background:
      radial-gradient(circle at 50% 30%, rgba(255, 255, 255, 0.1), transparent 58%),
      rgba(7, 10, 16, 0.5);
    box-shadow:
      0 14px 22px rgba(0, 0, 0, 0.62),
      0 0 15px color-mix(in srgb, var(--primary) 22%, transparent);
  }
  .visual-card .badge {
    z-index: 3;
    top: 20px;
    width: 46px;
    height: 46px;
  }
  .visual-card .badge.alpha { left: 20px; }
  .visual-card .badge.lucky { right: 20px; width: 40px; height: 40px; }
  .visual-foot {
    position: absolute;
    z-index: 2;
    inset: auto 0 0;
    min-height: 65px;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 14px 20px 13px;
    background: linear-gradient(0deg, rgba(6, 8, 13, 0.96), rgba(6, 8, 13, 0.72) 68%, transparent);
  }
  .overline {
    margin-bottom: 4px;
    color: #8d8498;
    font: 600 var(--type-micro) var(--font-head);
    letter-spacing: 0.17em;
  }
  .visual-foot .stars { font-size: 17px; }
  .soul-summary { text-align: right; }
  .soul-values {
    display: grid;
    grid-template-columns: repeat(2, auto);
    justify-content: end;
    gap: 1px 8px;
    color: #c789f0;
    font: 700 var(--type-caption) var(--font-head);
    font-variant-numeric: tabular-nums;
    text-shadow: 0 0 12px rgba(176, 96, 224, 0.45);
  }

  .hero-copy {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 10px;
  }
  .hero-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .species {
    min-width: 215px;
    max-width: 330px;
    padding: 7px 10px 7px 12px;
    border-color: color-mix(in srgb, var(--primary) 46%, rgba(255, 255, 255, 0.11));
    background: color-mix(in srgb, var(--primary) 10%, rgba(255, 255, 255, 0.025));
  }
  .species-copy strong { color: #efe9f4; font-size: 16px; }
  .species-change { margin-left: auto; }
  .variant-controls { display: flex; gap: 8px; }
  .variant { width: 54px; height: 50px; border-radius: 12px; }
  .variant img { width: 27px; height: 27px; }

  .identity { min-width: 0; }
  .paldex {
    min-height: 15px;
    margin-bottom: 2px;
    color: color-mix(in srgb, var(--primary) 46%, #a99ab4);
    font: 600 var(--type-caption) var(--font-head);
    letter-spacing: 0.18em;
  }
  .nameline { gap: 10px; }
  .name {
    width: min(100%, 520px);
    max-width: none;
    padding: 0 2px 3px;
    color: #fbfafc;
    font-size: clamp(33px, 3.2vw, 50px);
    text-shadow: 0 3px 18px rgba(0, 0, 0, 0.5);
  }
  .gender { min-width: 90px; height: 40px; }
  .elements { min-height: 27px; display: flex; align-items: center; flex-wrap: wrap; gap: 8px; margin-top: 8px; }

  .level-and-stats { min-width: 0; display: grid; grid-template-columns: 155px minmax(0, 1fr); gap: 14px; }
  .level {
    justify-content: flex-start;
    gap: 8px;
    padding: 8px 10px;
    border: 1px solid rgba(63, 199, 224, 0.14);
    border-radius: 13px;
    background: rgba(63, 199, 224, 0.045);
  }
  .lvbtn { width: 32px; height: 38px; border-radius: 9px; font-size: 20px; }
  .lvbox { min-width: 55px; display: flex; flex-direction: column; align-items: center; cursor: text; }
  .lvcap { font-size: var(--type-micro); letter-spacing: 0.18em; }
  .lvnum { width: 58px; font-size: 31px; border-bottom: 0; }
  .combat-stats {
    min-width: 0;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.075);
    border-radius: 13px;
    background: rgba(255, 255, 255, 0.025);
  }
  .combat-stat {
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 9px 10px;
    border-left: 1px solid rgba(255, 255, 255, 0.065);
  }
  .combat-stat:first-child { border-left: 0; }
  .combat-stat > span:last-child { min-width: 0; display: flex; flex-direction: column; }
  .combat-stat small {
    color: #7d7786;
    font: 600 var(--type-micro) var(--font-head);
    letter-spacing: 0.12em;
  }
  .combat-stat strong {
    overflow: hidden;
    color: #ece9ef;
    font: 700 clamp(18px, 1.8vw, 27px) var(--font-head);
    text-overflow: ellipsis;
  }
  .stat-glyph { font: 700 22px var(--font-head); }
  .hp-stat .stat-glyph { color: #6bdb76; }
  .attack-stat .stat-glyph { color: color-mix(in srgb, var(--primary) 58%, #ff9a58); }
  .defense-stat .stat-glyph { color: color-mix(in srgb, var(--secondary) 60%, #b98cff); }

  .vitals {
    display: grid;
    grid-template-columns: 1.22fr 0.85fr 0.85fr 1.05fr;
    gap: 8px;
  }
  .vital {
    min-width: 0;
    padding: 8px 9px;
    border: 1px solid rgba(255, 255, 255, 0.065);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.022);
  }
  .vital-head {
    min-height: 23px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 5px;
    margin-bottom: 5px;
    color: #918b99;
    font: 600 var(--type-label) var(--font-head);
    letter-spacing: 0.11em;
  }
  .vital-value { min-width: 0; display: inline-flex; align-items: center; color: #7d7785; letter-spacing: 0; }
  .vital .track { height: 6px; }
  .vital .stat-number { width: 47px; padding: 2px 4px; font-size: var(--type-body); }
  .vital .stat-number.hp { width: 69px; }
  .trust-rank {
    justify-content: flex-end;
    gap: 4px;
    margin-top: 5px;
    color: #756b80;
    font: 600 var(--type-micro) var(--font-head);
    letter-spacing: 0.08em;
  }
  .trust-rank .stat-number { width: 32px; color: #d8c4ee; }

  .editor-grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(250px, 0.95fr) minmax(350px, 1.35fr) minmax(220px, 0.78fr);
  }
  .editor-panel {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    padding: 17px 20px 22px;
  }
  .editor-panel + .editor-panel { border-left: 1px solid rgba(255, 255, 255, 0.055); }
  .traits-panel { display: flex; flex-direction: column; gap: 18px; }
  .passblock { flex: none; }
  .partner {
    padding: 12px 13px;
    border-radius: 12px;
    background:
      radial-gradient(circle at 8% 20%, color-mix(in srgb, var(--c) 12%, transparent), transparent 54%),
      rgba(255, 255, 255, 0.02);
  }
  .pname { font-size: 16px; }
  .pdesc { margin-top: 6px; font-size: var(--type-body); line-height: 1.45; }
  .passives { gap: 7px; }
  .add { padding: 9px; }

  .moves-panel { display: flex; flex-direction: column; }
  .moveslots { gap: 6px; padding: 8px; }
  .move { gap: 9px; padding: 8px 10px; }
  .emptyslot { padding: 9px; font-size: var(--type-caption); }
  .bench-head { margin-top: 10px; }
  .bench { gap: 6px; }
  .bench-move { padding: 7px 10px; }
  .mname { font-size: 16px; }
  .bench-move .mname { font-size: var(--type-control); }
  .mpwrcap { font-size: var(--type-micro); }
  .mpwr { font-size: 16px; }

  .work-panel { padding-left: 18px; padding-right: 18px; }
  .worksuit { gap: 6px; }
  .none { padding: 14px; color: #6e7a86; text-align: center; font-size: 12px; }

  .card.empty .art-shell {
    width: 120px;
    transform: none;
    border: 0;
    background: transparent;
    box-shadow: 0 0 24px rgba(176, 96, 224, 0.42);
    opacity: 0.62;
  }
  .card.empty .species { min-height: 48px; }

  @media (max-height: 820px) {
    .hero {
      flex-basis: 345px;
      gap: 18px;
      padding: 14px 18px;
    }
    .visual-card::before { inset: 10px; }
    .name { font-size: 34px; }
    .elements { margin-top: 5px; }
    .level-and-stats { grid-template-columns: 145px minmax(0, 1fr); }
    .vital { padding: 6px 8px; }
    .editor-panel { padding-top: 13px; }
  }

  @media (max-width: 1120px) {
    .hero { flex-basis: 385px; grid-template-columns: minmax(250px, 34%) minmax(0, 1fr); gap: 16px; }
    .vitals { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .editor-grid { grid-template-columns: minmax(220px, 0.9fr) minmax(320px, 1.35fr) minmax(200px, 0.75fr); }
    .combat-stat { gap: 5px; padding-inline: 6px; }
  }

  @media (max-width: 880px) {
    .card { overflow: auto; }
    .hero { flex: none; min-height: 640px; grid-template-columns: 1fr; }
    .visual-card { min-height: 300px; }
    .editor-grid { flex: none; grid-template-columns: 1fr; }
    .editor-panel { overflow: visible; border-left: 0 !important; border-top: 1px solid rgba(255, 255, 255, 0.055); }
  }
</style>
