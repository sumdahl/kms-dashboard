# HTMX Skill

## When to use
Use this skill whenever working with HTMX attributes, requests, responses, or patterns in kms-dashboard (Axum + Askama + HTMX stack).

## Key files
- `quick-reference.md` — attributes cheatsheet
- `docs.md` — full documentation
- `examples/basic.md` — usage patterns

## Quick rules
- Always return partial HTML fragments from Axum handlers for HTMX targets
- Use `HX-Trigger` response header for out-of-band events
- Prefer `hx-boost` on anchor tags before reaching for `hx-get`
- Swap strategy default is `innerHTML` — be explicit when using `outerSwap` or `beforeend`
