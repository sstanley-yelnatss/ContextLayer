# Dark Editorial Landing Page — Design Spec

Portable design language extracted from [Originate (Polsia)](https://originate.polsia.app/). Use for any B2B / SaaS / studio-style product landing page.

**Files in this template:**
- `template.html` — open in browser, duplicate, swap copy
- `EXTENSIONS.md` — pricing section + animation patterns that match this system
- This file — tokens, components, content patterns

---

## Vibe

**Dark editorial SaaS** — near-black canvas, warm off-white text, single teal accent, monospace metadata, oversized headlines, hairline dividers. Reads like a venture studio / infra tool, not a consumer app.

No images, no gradients on cards, no drop shadows. Restraint is the aesthetic.

---

## Color tokens

```css
:root {
  --bg:         #050505;   /* page background */
  --bg-2:       #0d0d0d;   /* cards, pipeline steps, pricing tiers */
  --bg-3:       #141414;   /* optional elevated surface */
  --fg:         #f0ede8;   /* primary text — warm off-white */
  --fg-2:       #8a8680;   /* body / secondary text */
  --fg-3:       #555250;   /* muted labels, tags, footer */
  --accent:     #00d4aa;   /* labels, step numbers, dots, highlights */
  --accent-dim: rgba(0, 212, 170, 0.12);
  --border:     #1e1e1c;   /* section dividers, card borders */
}
```

**Accent variants** (swap `--accent` + radial glow color only):

| Hue    | Hex       | Feel              |
|--------|-----------|-------------------|
| Teal   | `#00d4aa` | Default / orig    |
| Blue   | `#6c8cff` | Devtools / API    |
| Amber  | `#e8a838` | Finance / ops     |
| Violet | `#c084fc` | AI / productivity |

**Ambient glow** (fixed pseudo-element on `body`):

```css
body::before {
  content: '';
  position: fixed;
  inset: 0;
  background: radial-gradient(
    ellipse 80% 60% at 50% -10%,
    rgba(0, 212, 170, 0.04) 0%,
    transparent 60%
  );
  pointer-events: none;
  z-index: 0;
}
```

---

## Typography

**Fonts (Google Fonts):**

```
Space Grotesk — 300, 400, 500, 600, 700  → headlines + body
JetBrains Mono — 400, 500                → labels, logo, stats, tags
```

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

| Role              | Font           | Weight | Size                          | Extra                          |
|-------------------|----------------|--------|-------------------------------|--------------------------------|
| Body              | Space Grotesk  | 400    | 16px base                     | `line-height: 1.6`             |
| Hero H1           | Space Grotesk  | 700    | `clamp(52px, 8vw, 96px)`      | `letter-spacing: -0.03em`      |
| Section H2        | Space Grotesk  | 600    | `clamp(28px, 4vw, 44px)`      | `letter-spacing: -0.02em`      |
| Closing H2        | Space Grotesk  | 700    | `clamp(32px, 5vw, 56px)`      | `letter-spacing: -0.03em`      |
| Step title        | Space Grotesk  | 600    | 20px                          |                                |
| Body paragraphs   | Space Grotesk  | 400    | 15–17px                       | `color: var(--fg-2)`           |
| Eyebrow / label   | JetBrains Mono | 400    | 11–12px                       | uppercase, `letter-spacing: 0.1–0.12em`, `color: var(--accent)` |
| Logo / footer     | JetBrains Mono | 500    | 12–14px                       | `letter-spacing: 0.12em`       |
| Stat value        | JetBrains Mono | 500    | 28px                          |                                |
| Tags              | JetBrains Mono | 400    | 11px                          | `letter-spacing: 0.04em`       |

**Pairing rule:** Sans = narrative. Mono = metadata (never long paragraphs in mono).

---

## Layout

| Token                 | Value                          |
|-----------------------|--------------------------------|
| Max content width     | `1200px`                       |
| Horizontal padding    | `48px` desktop / `24px` mobile |
| Section padding Y     | `120px` (closing: `140px`)     |
| Hero padding          | `120px 48px 100px`             |
| Nav padding           | `28px 48px`                    |
| 2-col grid gap        | `80px` desktop / `40px` mobile |
| Manifesto label col   | `280px` fixed left column      |

**Section rhythm:** Full-width blocks separated by `border-bottom: 1px solid var(--border)`. Inner content always `max-width: 1200px; margin: 0 auto`.

**Breakpoint:** Single breakpoint at `768px` — grids stack, pipeline goes vertical, stat dividers hide.

---

## Page structure

```
NAV          → mono logo + muted one-line tagline
HERO         → accent eyebrow + giant H1 + sub (max ~560px) + 3 stats
MANIFESTO    → left label column + problem headline + 2 body paragraphs
HOW IT WORKS → eyebrow + H2 + horizontal pipeline (3 steps + arrows)
WHY IT WORKS → 2-col: headline + intro | feature bullets with accent dots
[PRICING]    → optional — see EXTENSIONS.md
CLOSING      → urgency H2 + one reassurance paragraph
FOOTER       → brand left, credit right
```

---

## Components

### Nav
- Bottom border only (no background change)
- Logo + tagline on same baseline row, `gap: 24px`

### Hero eyebrow
- Mono, accent, uppercase — category claim (“X run better with AI”)

### Stats row
- 3 items, mono values, uppercase muted labels
- `1px × 40px` vertical dividers between stats

### Section label
- Mono `11px`, accent, uppercase — sits above or in left grid column

### Manifesto grid
- `grid-template-columns: 280px 1fr`
- Headline allows `<br>` for dramatic line breaks
- Body: stacked paragraphs, `gap: 20px`, `line-height: 1.75`

### Pipeline cards
- `--bg-2` fill, `1px` border, `padding: 40px 36px`
- `border-radius: 2px` on outer corners only (first/last child)
- Step number top → title → description → tags (`margin-top: auto`)
- Chevron SVG arrows between cards (`--fg-3` stroke)
- Tags: mono bordered pills, `padding: 5px 10px`, `border-radius: 2px`

### Feature bullets
- `8px` accent circle dot (not icon font)
- Pattern: **Bold lead** — em dash — explanation in `--fg-2`

### Footer
- `justify-content: space-between`
- Left: logo + `13px` description
- Right: mono `11px` attribution

---

## Copy patterns

| Section    | Formula |
|------------|---------|
| Eyebrow    | `[Audience] run better with [category]` |
| Hero H1    | Short punchy claim, use `<br>` for rhythm |
| Hero sub   | One sentence, concrete outcome |
| Stats      | 3 numbers — specific, not vague (“24/7”, “3 agents”, “0 manual”) |
| Problem H2 | Provocative contrast, line break mid-thought |
| Pipeline   | `01` / `02` / `03` + agent/step name + paragraph + 4 capability tags |
| Features   | 4 items, each opens with bold differentiator |
| Closing    | Urgency line + calm reassurance (no hard sell) |

Original page had **no CTA button** — copy-only close. Add CTA/pricing when ready (see `EXTENSIONS.md`).

---

## Do / Don't

**Do**
- Keep borders hairline (`1px`, `--border`)
- Use `border-radius: 2px` max — barely rounded
- Let whitespace breathe (`120px` section padding)
- Break headlines across lines intentionally
- Use em dashes in feature copy

**Don't**
- Add card shadows or heavy gradients
- Use more than one accent color
- Put body copy in mono
- Over-iconify (dots > icons)
- Crowd sections — this template is sparse by design

---

## Quick customization checklist

- [ ] Swap `--accent` for brand color (+ update glow `rgba`)
- [ ] Replace logo text + tagline
- [ ] Rewrite 6 content sections (hero → closing)
- [ ] Adjust pipeline to 2–4 steps if needed
- [ ] Add pricing block (stub in `template.html`, styles in `EXTENSIONS.md`)
- [ ] Add fade-in animations (optional, `EXTENSIONS.md`)
- [ ] Add primary CTA button when ready
