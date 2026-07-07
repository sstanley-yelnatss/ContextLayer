# Extensions — Pricing, CTA & Animations

Patterns that match the dark editorial system. Drop into `template.html` after the base sections.

---

## Primary CTA button

Minimal — no pill shape, no gradient. Matches the 2px radius language.

```css
.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 500;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--bg);
  background: var(--accent);
  border: none;
  padding: 14px 28px;
  border-radius: 2px;
  cursor: pointer;
  text-decoration: none;
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.btn-primary:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-ghost {
  font-family: var(--font-mono);
  font-size: 13px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--fg-2);
  background: transparent;
  border: 1px solid var(--border);
  padding: 14px 28px;
  border-radius: 2px;
  cursor: pointer;
  text-decoration: none;
  transition: color 0.2s ease, border-color 0.2s ease;
}

.btn-ghost:hover {
  color: var(--fg);
  border-color: var(--fg-3);
}
```

**Placement:** Hero below `.hero-sub` or in closing section below `.closing-sub`. Pair ghost + primary if you need two actions.

---

## Pricing section

Insert between **Why it works** and **Closing**. Same section padding and border rhythm.

### HTML structure

```html
<section class="pricing">
  <div class="pricing-inner">
    <div class="section-label">Pricing</div>
    <h2 class="section-headline">Simple tiers.<br>No surprises.</h2>

    <div class="pricing-grid">
      <div class="pricing-tier">
        <div class="tier-name">Starter</div>
        <div class="tier-price"><span class="tier-amount">€79</span><span class="tier-period">/mo</span></div>
        <p class="tier-desc">For teams getting started with [core use case].</p>
        <ul class="tier-features">
          <li>Feature one</li>
          <li>Feature two</li>
          <li>Feature three</li>
        </ul>
        <a href="#" class="btn-ghost tier-cta">Get started</a>
      </div>

      <div class="pricing-tier pricing-tier--featured">
        <div class="tier-badge">Most popular</div>
        <div class="tier-name">Pro</div>
        <div class="tier-price"><span class="tier-amount">€129</span><span class="tier-period">/mo</span></div>
        <p class="tier-desc">For daily power users who need [key capability].</p>
        <ul class="tier-features">
          <li>Everything in Starter</li>
          <li>Feature four</li>
          <li>Feature five</li>
        </ul>
        <a href="#" class="btn-primary tier-cta">Get started</a>
      </div>

      <div class="pricing-tier">
        <div class="tier-name">Enterprise</div>
        <div class="tier-price"><span class="tier-amount">Custom</span></div>
        <p class="tier-desc">For orgs with compliance, SSO, or dedicated infra.</p>
        <ul class="tier-features">
          <li>Everything in Pro</li>
          <li>Dedicated support</li>
          <li>Custom SLA</li>
        </ul>
        <a href="#" class="btn-ghost tier-cta">Contact us</a>
      </div>
    </div>
  </div>
</section>
```

### CSS

```css
.pricing {
  padding: 120px 48px;
  border-bottom: 1px solid var(--border);
}

.pricing-inner {
  max-width: 1200px;
  margin: 0 auto;
}

.pricing-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0;
  margin-top: 64px;
}

.pricing-tier {
  background: var(--bg-2);
  border: 1px solid var(--border);
  padding: 40px 36px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.pricing-tier:first-child { border-radius: 2px 0 0 2px; }
.pricing-tier:last-child  { border-radius: 0 2px 2px 0; }

.pricing-tier--featured {
  background: var(--bg-3);
  border-color: var(--accent);
  position: relative;
}

.tier-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--accent);
}

.tier-name {
  font-size: 20px;
  font-weight: 600;
  color: var(--fg);
}

.tier-price {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.tier-amount {
  font-family: var(--font-mono);
  font-size: 36px;
  font-weight: 500;
  color: var(--fg);
  letter-spacing: -0.02em;
}

.tier-period {
  font-size: 14px;
  color: var(--fg-3);
}

.tier-desc {
  font-size: 15px;
  color: var(--fg-2);
  line-height: 1.65;
}

.tier-features {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: auto;
}

.tier-features li {
  font-size: 14px;
  color: var(--fg-2);
  padding-left: 16px;
  position: relative;
}

.tier-features li::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
}

.tier-cta {
  margin-top: 8px;
  text-align: center;
  justify-content: center;
}

@media (max-width: 768px) {
  .pricing { padding-left: 24px; padding-right: 24px; }
  .pricing-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }
  .pricing-tier,
  .pricing-tier:first-child,
  .pricing-tier:last-child {
    border-radius: 2px;
  }
}
```

---

## Animations

Keep motion subtle — this design is editorial, not flashy. Prefer opacity + translateY, 0.5–0.7s, ease-out.

### CSS-only fade-up on scroll

Add to `<head>` after base styles:

```css
.reveal {
  opacity: 0;
  transform: translateY(24px);
  transition: opacity 0.6s ease, transform 0.6s ease;
}

.reveal.is-visible {
  opacity: 1;
  transform: translateY(0);
}

/* Stagger children in pipeline / pricing */
.reveal-stagger > * {
  opacity: 0;
  transform: translateY(20px);
  transition: opacity 0.5s ease, transform 0.5s ease;
}

.reveal-stagger.is-visible > *:nth-child(1) { transition-delay: 0s; }
.reveal-stagger.is-visible > *:nth-child(2) { transition-delay: 0.1s; }
.reveal-stagger.is-visible > *:nth-child(3) { transition-delay: 0.2s; }
.reveal-stagger.is-visible > *:nth-child(4) { transition-delay: 0.3s; }
.reveal-stagger.is-visible > *:nth-child(5) { transition-delay: 0.4s; }

.reveal-stagger.is-visible > * {
  opacity: 1;
  transform: translateY(0);
}
```

Add class `reveal` to section inners or headlines. Add `reveal-stagger` to `.pipeline` or `.pricing-grid`.

Script (before `</body>`):

```html
<script>
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.15, rootMargin: '0px 0px -40px 0px' }
  );

  document.querySelectorAll('.reveal, .reveal-stagger').forEach((el) => observer.observe(el));
</script>
```

### Hero entrance (on load)

```css
@keyframes hero-in {
  from { opacity: 0; transform: translateY(16px); }
  to   { opacity: 1; transform: translateY(0); }
}

.hero-label    { animation: hero-in 0.7s ease 0.1s both; }
.hero-headline { animation: hero-in 0.7s ease 0.2s both; }
.hero-sub      { animation: hero-in 0.7s ease 0.35s both; }
.hero-stats    { animation: hero-in 0.7s ease 0.5s both; }
```

### Stat counter (optional)

For hero stats like `0` → animate count on scroll. Use a tiny helper or library; keep numbers in mono. Don't animate if values aren't numeric.

### What to avoid

- Parallax, bounce, or scale-on-hover on cards
- Gradient text
- Cursor trails / particles (breaks the restrained tone)
- Animation duration > 1s on any element

---

## Next.js / Tailwind port (quick map)

If you move this to a React stack:

| CSS token        | Tailwind equivalent (extend theme)     |
|------------------|----------------------------------------|
| `--bg`           | `bg-[#050505]` or `colors.bg.DEFAULT`  |
| `--accent`       | `colors.accent.DEFAULT`                |
| `--fg-2`         | `colors.muted`                         |
| Space Grotesk    | `font-sans` via `next/font/google`     |
| JetBrains Mono   | `font-mono` via `next/font/google`     |
| Section padding  | `py-30 px-12 md:px-6`                  |
| Max width        | `max-w-[1200px] mx-auto`               |
| Border sections  | `border-b border-[#1e1e1c]`            |

Keep components as server components; add `reveal` observer in a small client wrapper.
