# htmx Quick Reference

## Core Attributes

| Attribute | Description |
|-----------|-------------|
| `hx-get`, `hx-post`, `hx-put`, `hx-patch`, `hx-delete` | HTTP methods |
| `hx-target` | CSS selector for response target |
| `hx-trigger` | Event that triggers the request |
| `hx-swap` | How to swap content (innerHTML, outerHTML, etc.) |
| `hx-indicator` | Show loading indicator |
| `hx-boost` | Convert links/forms to AJAX |
| `hx-push-url` | Push URL to history |
| `hx-confirm` | Confirm dialog before request |
| `hx-include` | Include values from other elements |
| `hx-params` | Filter which parameters to send |

## Common Triggers

- `click` (default for most elements)
- `change` (default for input, select, textarea)
- `submit` (default for form)
- `mouseenter`, `keyup`, `focus`
- `load` (on page load)
- `revealed` (when scrolled into view)
- `intersect` (intersection observer)
- `every 2s` (polling)

## Trigger Modifiers

- `changed` - only if value changed
- `delay:500ms` - debounce
- `throttle:500ms` - rate limit
- `once` - only trigger once
- `from:#id` - listen on different element

## Swap Options

- `innerHTML` - default, content inside target
- `outerHTML` - replace entire target
- `afterbegin`, `beforeend` - prepend/append
- `delete` - remove target
- `none` - don't swap content

## Extended CSS Selectors

- `this` - target the element itself
- `closest .class` - find closest ancestor
- `next .class` - find next sibling
- `previous .class` - find previous sibling
- `find .class` - find child element

## Response Headers (Server → Client)

- `HX-Location` - redirect
- `HX-Redirect` - redirect
- `HX-Refresh` - full page refresh
- `HX-Trigger` - trigger client events

## Request Headers (Client → Server)

- `HX-Request: true`
- `HX-Target` - target element ID
- `HX-Trigger` - triggering element ID
- `HX-Boosted` - if boosted

## Installation

```html
<script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js"></script>
```

npm: `npm install htmx.org@2.0.8`

## Validation Events

- `htmx:validation:validate` - before validation
- `htmx:validation:failed` - validation failed
- `htmx:validation:halted` - request not sent due to validation
