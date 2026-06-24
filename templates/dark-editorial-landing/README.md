# Dark Editorial Landing Page Template

Reusable single-page landing template — dark editorial SaaS aesthetic (from [Originate](https://originate.polsia.app/)).

## ContextLayer files

| File | Purpose |
|------|---------|
| **`contextlayer.html`** | Filled landing page for ContextLayer — open in browser or deploy as static site |
| `template.html` | Blank scaffold with `[BRACKET]` placeholders |
| `DESIGN-SPEC.md` | Colors, type, layout, components, copy formulas |
| `EXTENSIONS.md` | Pricing grid, CTA buttons, scroll animations |

## Quick start

1. Open **`contextlayer.html`** in a browser to preview
2. For another product, duplicate `template.html` and swap copy
3. Add pricing / animations from `EXTENSIONS.md` when ready

## Customization order

1. `--accent` in `:root` if not using teal
2. Nav logo + tagline
3. Hero → Manifesto → Pipeline → Features → Closing
4. Footer attribution
5. Pricing + extra animations (optional polish)

## Stack notes

- **Static HTML:** use as-is (current approach)
- **Next.js / Vercel:** split sections into components; see Tailwind map in `EXTENSIONS.md`
- **Cursor:** point agent at `DESIGN-SPEC.md` when extending the page
