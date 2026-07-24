# Handoff: Palbox Studio — Global Palbox Editor (State A)

## Overview
Palbox Studio is a desktop app (Svelte + Tauri) for editing a Palworld 1.0 **Global Palbox**
save. One pal is selected at a time and shown on a large, editable **Pal Card** in the center;
a **left drawer** houses the box explorer and a **right drawer** houses advanced tuning
(IV / breeding + Statue of Power). This handoff covers **State A** — the full hero Pal Card
plus both slide-out drawers, at hi-fi.

## About the Design Files
The file in this bundle (`Palbox Studio.dc.html`) is a **design reference created in HTML** — a
working prototype showing the intended look and behavior. It is **not production code to copy
directly.** The task is to **recreate this design in the target codebase (Svelte + Tauri)** using
its established components, stores, and patterns. Styles are inline in the prototype specifically
so they're easy to lift; treat them as the source of truth for exact values.

> The HTML uses a small streaming-component runtime (`support.js`, `class Component extends
> DCLogic`). Ignore that runtime — it's just the prototype harness. Only the markup, inline
> styles, and the behavior described below matter for the Svelte rebuild.

## Fidelity
**High-fidelity (hifi).** Final colors, typography, spacing, glow/frost treatment, and
interactions are all intentional. Recreate the UI pixel-closely using the codebase's own
Svelte components. Where a value here conflicts with an existing house token, prefer this doc
for this feature (the visual identity is deliberate).

### Scope / what is NOT built
- Only **State A** is built (hero card + both drawers). States B/C from the brief are not
  separately mocked (the drawers here already cover most of B/C).
- Save-file loading/writing is **not** implemented — all data is placeholder (example pal:
  *Incineram*, Fire/Dark).
- These controls are **visual only** (not wired): Presets, Add/Clone/Delete, group create/rename,
  Sort menu, Filter "Apply/Clear", the IV sliders and Statue steppers, condensation stars,
  passive add. They show the intended affordance and states, not working logic.
- **Wired and interactive:** drawer open/close (edge tabs + transforms), favorite toggle,
  inline name edit, Level +/− and typable input, box view matrix/list toggle, filter modal
  open/close, Moves equip/unequip (click + drag), Work Suitability +/− steppers.
- A full **CSS custom-property token spec** (brief deliverable #6) and a **component-states
  sheet** (§7) were requested but are **not** in this bundle yet — see Design Tokens below for
  the concrete values to seed the token file.

## Layout (whole app)

Fixed full-viewport dark app with a custom title bar.

- **Title bar** — height 48px. Left: brand mark (17px conic-gradient rounded square) +
  "PALBOX STUDIO" (Rajdhani 700, 17px, letter-spacing .14em) + a purple "GLOBAL PALBOX" identity
  chip. Right: a green "Editing a copy · backed up" reassurance chip (shield check icon), then
  window controls (— ▢ ✕, each 34×30px). Background
  `linear-gradient(180deg,rgba(27,39,51,.92),rgba(18,24,32,.92))`, 1px top inner light border.
- **Stage** — below the title bar (`top:48px; inset 0`), padding `14px 50px` (the 50px sides
  leave room for the edge tabs). Holds the full-bleed card.
- **Backdrop** — layered radial gradients (purple top-left, cyan bottom-right, charcoal center)
  over `linear-gradient(160deg,#0b0f15,#0a0d12,#080a0e)`, plus ~5 slow-floating glowing particle
  dots (respect `prefers-reduced-motion`).

### Pal Card (center, full-bleed)
Fills the stage. 1px gradient border (`linear-gradient(150deg, rgba(176,96,224,.55),
rgba(63,199,224,.18) 40%, rgba(176,96,224,.30))`), radius 16px, outer glow
`0 0 60px rgba(176,96,224,.20), 0 24px 60px rgba(0,0,0,.55)`. Inner surface
`linear-gradient(155deg,rgba(27,39,51,.90),rgba(15,20,27,.93))` + `backdrop-filter:blur(18px)`,
flex column, `overflow:hidden`.

Structure top-to-bottom:
1. **Header** (padding 16px 26px) — pal **Name** as an inline `<input>` (Rajdhani 700, 34px,
   dashed underline), a 34px "rename" pencil button, gender circle (28px). Below: **element
   pills** (Fire, Dark), Palpedia No. On the right: **PRESETS** button + **Favorite** star
   button (42px; filled amber when favorited). 1px bottom border.
2. **NEXT exp bar** (padding 11px 26px) — "NEXT" label + 8px cyan progress track (62%) + numeric
   readout. 1px bottom border.
3. **3-column body** — CSS grid
   `grid-template-columns: minmax(220px,320px) minmax(340px,1fr) minmax(250px,350px)`, `flex:1`,
   each column `overflow:auto; min-width:0`.

**Left column** — *Partner Skill* card (fire-tinted, name + description, no lock) and *Passive
Skills* (up to 4). Each passive is a rating-colored left-border card showing rank chevrons, the
name, and **the stats it adjusts** (e.g. Legend → "+20% Attack · +20% Defense · +15% Move Speed";
Ferocious → "+20% Attack"; Burly Body → "+20% Defense"), plus a dashed "+ Add passive" slot.

**Center column** — framed **portrait slot** (`height:min(32vh,290px)`, striped placeholder,
ALPHA + LUCKY badges top corners, CONDENSATION stars + PAL SOULS +N readout in a bottom gradient
overlay); then a **big LEVEL editor** (46px Rajdhani number, typable `<input>`, 46px − / + buttons
either side); then **MOVES** — a boxed set of up to 3 **equipped** move rows (grip handle,
element diamond, name, PWR value) over a cyan-tinted zone, and below an **AVAILABLE MOVES** list.
Click a bench move to equip (swaps out the oldest when 3 are equipped), click an equipped move to
unequip; both lists are drag sources/targets (HTML5 drag: bench→equip zone equips, equipped→bench
unequips). Column scrolls.

**Right column** — **STATS** (HP green bar 3,940; Attack 452▲; Defense 318; Work Speed 100▲;
SAN cyan bar; Food orange bar; Trust purple bar Rank 8; "▲" = in-game boosted up-arrow). Then
**WORK SUITABILITY** — all 12 jobs as rows: a rounded icon chip (2-letter code placeholder —
**replace with real job icons**), job name, a big level number, and stacked ▲/▼ steppers (0–4).
Active jobs (level > 0) are amber-tinted; N/A jobs (level 0) are greyed.

### Left Drawer — Global Box (width 440px, max 90vw)
Overlays from the left (does not push the card). Purple "genetic-data" identity: 1px purple
gradient border, `linear-gradient(155deg,rgba(24,17,32,.96),rgba(15,13,22,.97))` surface,
blur(20px). Header: "GLOBAL BOX" title + pal count + close chevron; search field + Sort + Filter;
group filter chips (All / Combat Team / Base Crew / + Group) + matrix/list view toggle. Body:
tiles in **matrix** (3-col grid, round portrait, name, element diamond + Lv.N) or **list**
(row with 44px portrait, name, element + Lv + passive hint). Tiles carry a badge slot (alpha "A",
lucky "★"). Footer: + Add / ⧉ Clone / 🗑 Delete.

### Right Drawer — Advanced (width 420px, max 90vw)
Overlays from the right. Cyan-accented border/surface. Three sections:
- **IV / BREEDING TRAITS** — HP / Attack (labelled "single talent · 1.0") / Defense, each a
  labelled value + track with a draggable-looking knob (0–100).
- **STATUE OF POWER** — statue image placeholder (🗿, replace with real art) + **Pal Soul
  Enhancement**: three per-stat rank tracks (HP / ATK / DEF), each − / + with a 10-pip rank bar
  in the stat's color.
- **SOULS & CONDENSATION** — Pal Souls +N stepper card (max +4) and Condensation stars card (0–4).
- A non-alarming amber warning note about save rewrites + backup.

### Edge tabs
Vertical "BOX" (purple, left) and "IV / STATUE" (cyan/amber, right) tabs. When a drawer is
closed the tab sits at the window edge; when open, the **tab slides with the drawer to its inner
edge and stays the click target to collapse it** (its arrow flips direction). Uses the same
transform transition as the drawer.

## Interactions & Behavior
- **Drawers:** open/close via edge tab (and header close chevron). Overlay, never push the card;
  both may be open at once. Transform: `translateX` with
  `transition: transform .42s cubic-bezier(.22,.61,.36,1)`. When `prefers-reduced-motion`,
  transition is `none` (and a global rule zeroes animation/transition durations).
- **Name:** inline editable input, live.
- **Level:** − / + buttons (clamp 1–60) and a typable numeric input (parsed + clamped).
- **Favorite:** toggles star fill (amber ↔ hollow).
- **Box view:** matrix ⇄ list toggle.
- **Filter modal:** Filter button opens a centered modal (element/sort/gender/type/groups);
  backdrop or ✕/Apply/Clear closes it.
- **Moves:** click-to-equip / click-to-unequip with a max of 3 equipped (equipping a 4th drops
  the oldest); HTML5 drag between the equipped zone and the available list. Drag data is the move
  id via `dataTransfer`.
- **Work Suitability:** ▲/▼ per job, clamped 0–4; row + icon restyle to active/greyed at the
  0↔>0 boundary.

## State Management
Prototype state (map to a Svelte store per selected pal):
- `leftOpen`, `rightOpen` (booleans) — drawer visibility.
- `favorite` (boolean), `name` (string), `level` (int 1–60).
- `view` — `"matrix" | "list"`; `filterOpen` (boolean).
- `moves` — array of `{id, name, el: "Fire"|"Dark"|"Neutral"|…, power}`.
- `activeIds` — ordered array of equipped move ids (max length 3).
- `jobs` — array of `{code, name, level 0–4}` for the 12 work suitabilities.
Real app additionally needs: the loaded save model, selected-pal id, groups/tags, IV values,
soul-enhancement ranks, condensation, and a backup-before-write step.

## Design Tokens
Seed the CSS custom-property spec (brief deliverable #6) from these:

**Surfaces**
- Panel base gradient: `rgba(27,39,51,.90) → rgba(15,20,27,.93)` (card), frosted `blur(18px)`.
- Drawer (box): `rgba(24,17,32,.96) → rgba(15,13,22,.97)`, blur(20px).
- Drawer (advanced): `rgba(20,29,38,.96) → rgba(13,18,25,.97)`, blur(20px).
- App background base: `linear-gradient(160deg,#0b0f15,#0a0d12,#080a0e)`.
- Hairline border: `rgba(255,255,255,.05–.09)`.

**Identity / accents**
- Global-box purple: `#B060E0` (border/glow `rgba(176,96,224,*)`), soul purple `#9B5FE0`.
- Amber (active/selected): `#F5A623`; light amber text `#F5C97A`.
- Cyan (section accents/headers): `#3FC7E0`; light cyan text `#9FD8E6`.

**Text**
- Primary `#F2F4F6`; secondary/labels `#9AA6B2`; muted `#6E7A86`.

**Stat bars**
- HP green `#5FD16A`; food/hunger orange `#E8963A`; SAN/defense cyan `#3FC7E0`; trust purple `#B060E0`.

**Elements (diamond swatch per element)**
- Neutral `#C0C4CA` · Fire `#F0743A` · Water `#3F8FE0` · Grass `#5FBE4A` · Electric `#F0C93A` ·
  Ice `#5FD1E0` · Ground `#C9A05F` · Dark `#9B5FE0` · Dragon `#E05FC0`.

**Passive rating chips (left border color)**
- Strong positive teal `#35C9A5`; positive gold `#F5A623`; (negative would be red/orange
  `#E05A5A` / `#E8963A`). Rank chevrons "▲" repeated by strength.

**Type**
- Headers / numbers: **Rajdhani** (500/600/700). Section labels use letter-spacing .14–.24em.
- Body / data: **Barlow**; condensed labels: **Barlow Semi Condensed** (600).
- Monospace (placeholder captions): system `ui-monospace, Menlo`.
- The brief requires **self-hosted fonts (no CDN)** — bundle Rajdhani + Barlow (+ Barlow Semi
  Condensed) locally. The prototype loads them from Google Fonts for convenience only.

**Radii** — pills 16–20px; cards/controls 8–12px; tiles 8–12px; card 15–16px.
**Shadows/glow** — accent glows `0 0 8–22px rgba(<accent>,.18–.6)`; card lift
`0 24px 60px rgba(0,0,0,.55)`.
**Spacing** — section gaps 16–22px; control padding 6–14px; grid gaps 6–10px.

## Assets
All imagery is **placeholder** — replace with real assets in-app:
- Pal portrait (center) and box tile portraits — striped placeholder + user icon.
- Statue of Power — 🗿 emoji placeholder.
- Work-suitability icons — 2-letter code chips; needs the 12 real job icons.
- Icons drawn inline as simple SVG (search, shield-check, pencil, person, star) — swap for the
  house icon set. No external image files are used.

## Files
- `Palbox Studio.dc.html` — the complete State A prototype (card + both drawers + filter modal).
  Inline styles throughout; logic in the `class Component` block near the bottom.
