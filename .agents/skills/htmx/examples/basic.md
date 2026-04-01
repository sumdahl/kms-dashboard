# htmx Examples

## Basic Button Click

```html
<button hx-post="/api/submit" hx-target="#result">
    Submit
</button>
<div id="result"></div>
```

## Search with Debounce

```html
<input type="text" 
       name="q" 
       hx-get="/api/search" 
       hx-trigger="keyup changed delay:300ms" 
       hx-target="#results"
       placeholder="Search...">
<div id="results"></div>
```

## Form Submission

```html
<form hx-post="/api/contact">
    <input type="text" name="name" required>
    <input type="email" name="email" required>
    <button type="submit">Send</button>
</form>
```

## Loading Indicator

```html
<button hx-get="/api/data" hx-indicator="#loading">
    Load Data
    <img class="htmx-indicator" src="/spinner.gif">
</button>
<span id="loading" class="htmx-indicator">Loading...</span>
```

## Delete with Confirmation

```html
<button hx-delete="/api/item/1" 
        hx-confirm="Are you sure?">
    Delete
</button>
```

## Boost Links and Forms

```html
<div hx-boost="true">
    <a href="/page">Link</a>
    <form action="/search">
        <input name="q">
    </form>
</div>
```

## Out of Band Updates

Server returns:
```html
<div id="sidebar" hx-swap-oob="true">Updated Sidebar</div>
<div>Main Content</div>
```

## Polling

```html
<div hx-get="/api/status" hx-trigger="every 5s">
    Status loading...
</div>
```

## History Push URL

```html
<a hx-get="/blog/post-1" hx-push-url="true">Post 1</a>
```

## Include Additional Elements

```html
<input hx-get="/api/search" hx-include="[name='filter']">
<select name="filter">
    <option value="all">All</option>
</select>
```

## File Upload

```html
<input type="file" 
       name="file" 
       hx-post="/api/upload" 
       hx-encoding="multipart/form-data">
```

## Custom Events

```javascript
// Trigger custom event from server
HX-Trigger: {"myEvent": {"message": "hello"}}

// Listen for it
<div hx-get="/api/data" hx-trigger="myEvent">
```

## Sync Requests (Cancel Previous)

```html
<input hx-get="/api/search" 
       hx-sync="this:abort">
```

## CSS Transitions

```css
.htmx-swapping {
    opacity: 0;
    transition: opacity 1s ease-out;
}
.htmx-settling {
    opacity: 1;
    transition: opacity 1s ease-in;
}
```
