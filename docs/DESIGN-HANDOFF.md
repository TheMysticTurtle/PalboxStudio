# Palbox Studio — Design Handoff

> **For:** the UI/UX design pass. **From:** the build side (implements in Svelte + Tauri).
> **Goal of this doc:** give you everything to explore a beautiful UI and hand back concrete
> mockups + a token spec we can implement directly. Play freely *within* the non-negotiables.
> Companion docs: [DIRECTION.md](DIRECTION.md) (vision), [QUICKREF.md](QUICKREF.md) (data facts),
> `docs/reference/` (in-game screenshots — local only).

---

## 1. What we're designing

A desktop app to edit a Palworld 1.0 **Global Pal box** save. Think "official companion
tool," not "modder utility." One pal is selected at a time and shown on a large, editable
**card**; the box of pals and the deep tuning options live in **retractable side drawers**.

**It must feel like Palworld** — same color world, same UI grammar — with a cleaner, more
intuitive layout of our own.

## 2. Non-negotiables vs. where to play

**Non-negotiable (please keep):**
- Palworld's visual language: dark frosted panels, cyan/amber accents, the **purple "genetic
  data" identity** for the Global box, element/rating color coding.
- **Official in-game terminology** everywhere (see §8 glossary). No fan slang.
- The structural layout: **center card, always visible; left drawer = box; right drawer =
  advanced (IV / breeding traits + Statue of Power).** Drawers pop out and retract.
- Everything **clearly labeled** and legible; dense data stays scannable.
- Theme-aware is a bonus but the game is dark-first — **design dark as primary.**

**Play freely here:**
- The actual tile design, card composition, motion, glow/texture treatment, spacing rhythm.
- How drawers animate (overlay vs. push), edge-tab styling, empty/hover/selected states.
- Iconography style for badges, the groups/tags UI, the search/filter panel.
- Anything that makes it more beautiful or more intuitive than the in-game screens.

## 3. Layout — three key states (wireframes are spatial, not visual)

Window is resizable desktop (target ~1280×800 min; looks great at 1080p+).

**State A — default (both drawers retracted): the hero card.**
```
┌─────────────────────────────────────────────────────────────────────┐
│  [ Palbox Studio ]                                     _ □ ✕         │
│┌─┐                                                               ┌─┐ │
││◄│                    ┌───────────────────────────┐              │►│ │
││ │                    │      PAL CARD (center)    │              │ │ │
││B│   LEVEL / name     │   hero portrait / art     │  stats       │I│ │
││O│   element pills    │                           │  work suit   │V│ │
││X│   partner skill    │                           │  active      │·│ │
││ │   passives         │                           │  skills →    │▲│ │
│└─┘                    └───────────────────────────┘              └─┘ │
│  edge tab: "BOX"                                    edge tab: "IV / STATUE"
└─────────────────────────────────────────────────────────────────────┘
```

**State B — left drawer open (Global Box Explorer).**
```
┌─────────────────────────────────────────────────────────────────────┐
│┌───────────────────────────┐                                        │
││ GLOBAL BOX            ✕/◄  │        ┌──────────────────┐            │
││ [search] [filter] [sort]  │        │   PAL CARD        │  (card    │
││ [ Group ▾ ]  [+tag]       │        │   (still visible, │   stays,  │
││ ┌──┐ ┌──┐ ┌──┐ ┌──┐       │        │    compressed or  │   drawer  │
││ │██│ │██│ │██│ │██│  tiles │        │    underlapped)   │   overlays│
││ └──┘ └──┘ └──┘ └──┘       │        └──────────────────┘   left)    │
││ collapsed list  ⇄  matrix │                                        │
││ [+ Add] [Clone] [Delete]  │                                        │
│└───────────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────────┘
```

**State C — right drawer open (Advanced: IV / breeding + Statue).**
```
┌─────────────────────────────────────────────────────────────────────┐
│                                     ┌────────────────────────────────┐│
│      ┌──────────────────┐           │  ADVANCED             ►/✕      ││
│      │   PAL CARD        │          │  ── IV / Breeding Traits ──    ││
│      │                   │          │  HP     [====|----] 100        ││
│      │                   │          │  Attack [======|--] 100        ││
│      └──────────────────┘           │  Defense[===|-----]  60        ││
│                                     │  ── Statue of Power ──         ││
│                                     │  [ 🗿 statue image ]  Souls +N ││
│                                     │  Condensation ★★★☆☆           ││
│                                     └────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```
Decide whether opening one drawer nudges/compresses the card or overlays it — your call;
the card must never be fully hidden, and both drawers must be openable one at a time (both
at once is optional — propose what feels best).

## 4. Component specs

> **Authoritative value ranges (see [SPECS-1.0.md](SPECS-1.0.md)) — the prototype got some of
> these wrong; use these:** Level **1–80** · IV/breeding traits **0–100** · **Work Suitability
> steppers 0–10** (not 0–4) · **Pal Souls 0–10** per stat · **Condensation 0–4** stars ·
> Passives up to **4** (rank −3..5) · Active Skills (moves) up to **3** equipped.

### 4a. Pal Card (center) — the star of the show
Model: the in-game Party **"Pal Stats"** card (`docs/reference/Large Main Card Reference.webp`),
re-composed so **moves/Active Skills sit on the card's right** and deep IV/statue tuning moves
to the right drawer. Every field below is **editable** and clearly labeled:
- **Identity:** hero portrait/art; **LEVEL** (cyan block); **Name** (+ inline edit); **gender**;
  **element pills** (1–2); `NEXT` exp; **Trust**; **Favorite**; condensation **stars** (0–4)
  and **Pal Souls `+N`** readouts (full editing of these two lives in the right drawer).
- **Core stats:** Attack, Defense, Work Speed (+ HP, SAN, hunger/food) — adjustable; show the
  in-game "boosted" up-arrow treatment.
- **Partner Skill** (may be locked, e.g. "Requires X Saddle" — show a locked treatment).
- **Passive Skills:** up to 4, as rating-colored chips (2×2); add/remove/filter affordance.
- **Work Suitability:** all 12 jobs; greyed when N/A; active shows `Lv.N` + fill.
- **Active Skills (= moves):** on the card's right; element-colored rows with power values;
  add/swap affordance.
- **Presets:** "apply preset" + author/save (passives and/or full builds).

### 4b. Global Box Explorer (left drawer)
- **Themed tiles, not bare dots.** Each tile = round portrait **inside a tile that matches the
  theme**, plus a brief overview: level, element, and a quick glance at skills/passives.
  Design **collapsed** (list rhythm) and **matrix** (grid, "peering into the box") variants,
  and the toggle between them.
- **Badges** on tiles (reuse a consistent slot system): shiny/lucky sparkle, alpha marker,
  element mark, and our own group/tag indicator. (In-game padlock tiers are just favoriting —
  we replace that concept with real groups; you may still show a "favorite" star.)
- **Controls header:** search field, filter, sort. Add / Clone / Delete actions.
- **Selected / hover / empty-slot** states all needed.

### 4c. Advanced drawer (right)
- **IV / Breeding Traits:** HP / Attack / Defense talent values (0–100), clearly labeled,
  slider + number. (Palworld 1.0 has a single attack talent — no separate melee/ranged.)
- **Statue of Power:** **Pal Souls** rank editor (`+N`) and **Condensation** stars (0–4).
  **Include an illustrative image of the Statue of Power** here. Label with official terms.

### 4d. Groups & tags (lives in/around the box explorer)
- Groups are **user-named, fully customizable** and behave like a **filter**.
- You **tag** pals into a group; a pal can be in several. Selecting a group filters the box.
- Needs: create/rename/delete group; assign/remove a tag on a pal (quick, ideally right on
  the tile or a multi-select); a group picker that doubles as a filter chip.
- Design the empty state ("no groups yet — make one") and the multi-select tagging flow.

### 4e. Search / Filter / Sort
Same *capability* as the in-game Sort modal (`docs/reference/Filter Menu visual reference.png`)
— sort type, element, gender, work suitability, passive, plus our **groups** — but **far
cleaner**. In-game uses sectioned checkboxes with an amber "active" highlight; make ours nicer.

### 4f. Global chrome
App title bar / window controls (Tauri custom titlebar is fine), a subtle safety affordance
(we always back up before writing; a small "working on a copy / backed up" reassurance fits
the Palworld tone). A non-alarming warning style for risky actions.

## 5. Visual language & tokens (please return these as CSS custom properties)

Approximate values sampled from the reference shots — refine and formalize into a token set:
- **Surfaces:** panel base dark navy/charcoal, semi-transparent frosted (~`#121820`–`#1B2733`,
  ~85–92% opacity); header bars a touch lighter with a 1px light top border.
- **Global-box identity:** **purple/magenta** glow + border (`~#B060E0` over `~#2A1840`) — the
  "genetic data" theme. Use it to make the Global box feel special vs. neutral gray.
- **Accents:** amber/gold `#F5A623` = active/selected; cyan `#3FC7E0` = section accents/headers.
- **Text:** primary `~#F2F4F6`; secondary/labels `~#9AA6B2`.
- **Stat bars:** HP green `#5FD16A`; hunger/food orange `#E8963A`.
- **Elements (9)** — colored diamond per: Neutral, Fire (orange), Water (blue), Grass (green),
  Electric (yellow), Ice (cyan), Ground (tan), Dark (purple), Dragon (magenta).
- **Passive rating chips:** teal/green = strong positive; gold/yellow = positive; red/orange =
  negative. Left color border + rank chevrons.
- **Type:** condensed/geometric sans in the Palworld spirit for headers; a clean readable sans
  for data. Propose a free/bundleable pairing (we ship offline — no external font CDNs).
- **Shape/spacing:** propose radii, spacing scale, glow/shadow treatment, and the "frosted
  glass over game world" backdrop approach (we won't have the live game behind us — suggest a
  static atmospheric backdrop that evokes it).

## 6. Interaction & motion
- Drawer pop-out/retract: smooth, quick, with clearly-labeled **edge tabs** when retracted.
- Tile collapsed ⇄ matrix transition.
- Editing affordances: what "this is editable" looks like (inline fields, sliders, steppers,
  pickers) consistently across card + drawers.
- Filtering feels instant; selecting a group visibly narrows the box.
- Respect reduced-motion.

## 7. States to cover in mockups
Default · hover · selected · focused · disabled/greyed (e.g. N/A work suit) · empty (empty box
slot, no groups) · locked (Partner Skill needs saddle) · warning (risky action) · loading.

## 8. Official terminology (use verbatim)
Global Palbox · Pal Genetic Data · **Attack · Defense · Work Speed · HP · SAN · NEXT** ·
**Partner Skill · Passive Skills · Active Skills** (= moves) · **Current Task · Food · Trust ·
Level Sync · Favorite · Edit** · unused **Stat Points** · Major/Minor Injury ·
**Work Suitability** (Kindling, Watering, Planting, Generating Electricity, Handiwork,
Gathering, Lumbering, Mining, Medicine Production, Cooling, Transporting, Farming) ·
**Elements** (Neutral, Fire, Water, Grass, Electric, Ice, Ground, Dark, Dragon) ·
Sort types (Palpedia No., Level, Element, Alpha Pal, Work Suitability Level, Trust, Expedition
Firepower) · **Statue of Power** · **Pal Souls** (`+N`) · **Condensation** (0–4 stars).

## 9. Reference material (local, in `docs/reference/`)
- `Large Main Card Reference.webp` — **the center-card model.**
- `GlobalPalbox+Card Reference 1.webp`, `...2.webp` — the Global box + card; note the purple box
  theme, tile badges, condensation stars, souls `+N`.
- `Filter Menu visual reference.png` — the Sort/Filter modal to out-pretty.
- `docs/reference/README.md` — extracted palette, element/passive colors, card anatomy, terms.

## 10. What we need back (deliverables)
1. **Hi-fi mockups** of States A/B/C (§3), dark theme, at ~1440×900.
2. **The Pal Card** in detail, every editable field labeled and in its edit affordance.
3. **Tile design** — collapsed list + matrix, with badge system and selected/hover states.
4. **Advanced drawer** — IV/breeding + Statue of Power (with the statue image placement).
5. **Groups & tags** UI + the search/filter/sort panel.
6. **A token spec as CSS custom properties** (colors, spacing, radii, type, glow/shadow) —
   this is what we implement from, so please make it concrete.
7. **Component states** sheet (§7) and an **iconography** approach (elements, work suit, badges).
8. Notes on **motion** and **resize/responsive** behavior.
Format: whatever you like to work in (HTML/CSS mockup is ideal since we build in Svelte and can
lift styles directly), plus the token spec.

## 11. Open questions for design to explore
- Drawer behavior: overlay the card, or push/compress it? One drawer at a time, or allow both?
- Hero visual: static pal **icon/art** (what we have) vs. a faux-3D framed portrait — what
  reads best without the game's live render?
- Backdrop: how to evoke the "frosted glass over the game world" without the live game behind.
- Tile density: how much data on a collapsed tile before it gets noisy?
- How prominent should the safety/backup reassurance be without nagging?
