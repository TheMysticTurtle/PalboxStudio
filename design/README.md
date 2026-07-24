# Design

Design source-of-truth for the UI. The build ports these into the Svelte frontend (`ui/`).

- **`state-a-prototype/`** — the first hi-fi interactive prototype (State A: hero Pal Card +
  both drawers + filter modal). `Palbox Studio.dc.html` is the reference mockup;
  `PROTOTYPE-NOTES.md` documents its structure, interactions, state model, and the concrete
  **design tokens** (palette / type / radii / glow) to seed our CSS custom properties.
  - It's a *reference*, not production code — we recreate it in Svelte components.
  - The prototype loads fonts from Google's CDN and needs an external `dc` runtime to be
    interactive; the shipped app **self-hosts fonts** and reimplements behavior in Svelte.
- **Apply the corrected value ranges** from [`../docs/SPECS-1.0.md`](../docs/SPECS-1.0.md)
  when porting — the prototype's Level (1–60), Work Suitability (0–4), and stepper caps are
  placeholders. Correct: Level 1–80, Work Suitability 0–10, Pal Souls 0–10, Condensation 0–4.

See [`../docs/DESIGN-HANDOFF.md`](../docs/DESIGN-HANDOFF.md) for the full brief.
