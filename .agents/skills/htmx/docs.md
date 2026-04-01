# htmx Documentation

## htmx in a Nutshell

htmx is a library that allows you to access modern browser features directly from HTML, rather than using JavaScript.

To understand htmx, first let's take a look at an anchor tag:

```html
<a href="/blog">Blog</a>
```

This anchor tag tells a browser:

> "When a user clicks on this link, issue an HTTP GET request to '/blog' and load the response content into the browser window".

With that in mind, consider the following bit of HTML:

```html
<button hx-post="/clicked"
    hx-trigger="click"
    hx-target="#parent-div"
    hx-swap="outerHTML">
    Click Me!
</button>
```

This tells htmx:

> "When a user clicks on this button, issue an HTTP POST request to '/clicked' and use the content from the response to replace the element with the id parent-div in the DOM"

htmx extends and generalizes the core idea of HTML as a hypertext, opening up many more possibilities directly within the language:

- Now any element, not just anchors and forms, can issue an HTTP request
- Now any event, not just clicks or form submissions, can trigger requests
- Now any HTTP verb, not just GET and POST, can be used
- Now any element, not just the entire window, can be the target for update by the request

Note that when you are using htmx, on the server side you typically respond with HTML, not JSON. This keeps you firmly within the original web programming model, using Hypertext As The Engine Of Application State without even needing to really understand that concept.

It's worth mentioning that, if you prefer, you can use the data- prefix when using htmx:

```html
<a data-hx-post="/click">Click Me!</a>
```

If you understand the concepts around htmx and want to see the quirks of the library, please see our QUIRKS page.

---

## 1.x to 2.x Migration Guide

Version 1 of htmx is still supported and supports IE11, but the latest version of htmx is 2.x.

- If you are migrating to htmx 2.x from htmx 1.x, please see the htmx 1.x migration guide.
- If you are migrating to htmx from intercooler.js, please see the intercooler migration guide.

---

## Installing

Htmx is a dependency-free, browser-oriented JavaScript library. This means that using it is as simple as adding a `<script>` tag to your document head. There is no need for a build system to use it.

### Via A CDN (e.g. jsDelivr)

The fastest way to get going with htmx is to load it via a CDN. You can simply add this to your head tag and get going:

```html
<script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js" integrity="sha384-/TgkGk7p307TH7EXJDuUlgG3Ce1UVolAOFopFekQkkXihi5u/6OCvVKyz1W+idaz" crossorigin="anonymous"></script>
```

An unminified version is also available as well:

```html
<script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.js" integrity="sha384-ezjq8118wdwdRMj+nX4bevEi+cDLTbhLAeFF688VK8tPDGeLUe0WoY2MZtSla72F" crossorigin="anonymous"></script>
```

While the CDN approach is extremely simple, you may want to consider not using CDNs in production.

### Download a Copy

The next easiest way to install htmx is to simply copy it into your project.

Download htmx.min.js from jsDelivr and add it to the appropriate directory in your project and include it where necessary with a `<script>` tag:

```html
<script src="/path/to/htmx.min.js"></script>
```

### npm

For npm-style build systems, you can install htmx via npm:

```bash
npm install htmx.org@2.0.8
```

After installing, you'll need to use appropriate tooling to use `node_modules/htmx.org/dist/htmx.js` (or .min.js). For example, you might bundle htmx with some extensions and project-specific code.

### Webpack

If you are using webpack to manage your JavaScript:

1. Install htmx via your favourite package manager (like npm or yarn)
2. Add the import to your index.js

```javascript
import 'htmx.org';
```

If you want to use the global htmx variable (recommended), you need to inject it to the window scope:

1. Create a custom JS file
2. Import this file to your index.js (below the import from step 2)

```javascript
import 'path/to/my_custom.js';
```

Then add this code to the file:

```javascript
window.htmx = require('htmx.org');
```

3. Finally, rebuild your bundle

---

## AJAX

The core of htmx is a set of attributes that allow you to issue AJAX requests directly from HTML:

| Attribute  | Description                              |
|------------|------------------------------------------|
| hx-get     | Issues a GET request to the given URL    |
| hx-post    | Issues a POST request to the given URL   |
| hx-put     | Issues a PUT request to the given URL   |
| hx-patch   | Issues a PATCH request to the given URL |
| hx-delete  | Issues a DELETE request to the given URL |

Each of these attributes takes a URL to issue an AJAX request to. The element will issue a request of the specified type to the given URL when the element is triggered:

```html
<button hx-put="/messages">
    Put To Messages
</button>
```

This tells the browser:

> When a user clicks on this button, issue a PUT request to the URL /messages and load the response into the button

---

## Triggering Requests

By default, AJAX requests are triggered by the "natural" event of an element:

- `input`, `textarea` & `select` are triggered on the change event
- `form` is triggered on the submit event
- everything else is triggered by the click event

If you want different behavior you can use the `hx-trigger` attribute to specify which event will cause the request.

Here is a div that posts to `/mouse_entered` when a mouse enters it:

```html
<div hx-post="/mouse_entered" hx-trigger="mouseenter">
    [Here Mouse, Mouse!]
</div>
```

### Trigger Modifiers

A trigger can also have a few additional modifiers that change its behavior. For example, if you want a request to only happen once, you can use the `once` modifier for the trigger:

```html
<div hx-post="/mouse_entered" hx-trigger="mouseenter once">
    [Here Mouse, Mouse!]
</div>
```

Other modifiers you can use for triggers are:

- `changed` - only issue a request if the value of the element has changed
- `delay:<time interval>` - wait the given amount of time (e.g. 1s) before issuing the request. If the event triggers again, the countdown is reset.
- `throttle:<time interval>` - wait the given amount of time (e.g. 1s) before issuing the request. Unlike delay if a new event occurs before the time limit is hit the event will be discarded, so the request will trigger at the end of the time period.
- `from:<CSS Selector>` - listen for the event on a different element. This can be used for things like keyboard shortcuts. Note that this CSS selector is not re-evaluated if the page changes.

You can use these attributes to implement many common UX patterns, such as Active Search:

```html
<input type="text" name="q"
    hx-get="/trigger_delay"
    hx-trigger="keyup changed delay:500ms"
    hx-target="#search-results"
    placeholder="Search...">
<div id="search-results"></div>
```

This input will issue a request 500 milliseconds after a key up event if the input has been changed and inserts the results into the div with the id search-results.

Multiple triggers can be specified in the `hx-trigger` attribute, separated by commas.

### Trigger Filters

You may also apply trigger filters by using square brackets after the event name, enclosing a JavaScript expression that will be evaluated. If the expression evaluates to true the event will trigger, otherwise it will not.

Here is an example that triggers only on a Control-Click of the element:

```html
<div hx-get="/clicked" hx-trigger="click[ctrlKey]">
    Control Click Me
</div>
```

Properties like `ctrlKey` will be resolved against the triggering event first, then against the global scope. The `this` symbol will be set to the current element.

### Special Events

htmx provides a few special events for use in `hx-trigger`:

- `load` - fires once when the element is first loaded
- `revealed` - fires once when an element first scrolls into the viewport
- `intersect` - fires once when an element first intersects the viewport. This supports two additional options:
  - `root:<selector>` - a CSS selector of the root element for intersection
  - `threshold:<float>` - a floating point number between 0.0 and 1.0, indicating what amount of intersection to fire the event on

You can also use custom events to trigger requests if you have an advanced use case.

---

## Polling

If you want an element to poll the given URL rather than wait for an event, you can use the `every` syntax with the `hx-trigger` attribute:

```html
<div hx-get="/news" hx-trigger="every 2s"></div>
```

This tells htmx:

> Every 2 seconds, issue a GET to /news and load the response into the div

If you want to stop polling from a server response you can respond with the HTTP response code 286 and the element will cancel the polling.

### Load Polling

Another technique that can be used to achieve polling in htmx is "load polling", where an element specifies a load trigger along with a delay, and replaces itself with the response:

```html
<div hx-get="/messages"
    hx-trigger="load delay:1s"
    hx-swap="outerHTML">
</div>
```

If the `/messages` end point keeps returning a div set up this way, it will keep "polling" back to the URL every second.

Load polling can be useful in situations where a poll has an end point at which point the polling terminates, such as when you are showing the user a progress bar.

---

## Request Indicators

When an AJAX request is issued it is often good to let the user know that something is happening since the browser will not give them any feedback. You can accomplish this in htmx by using `htmx-indicator` class.

The htmx-indicator class is defined so that the opacity of any element with this class is 0 by default, making it invisible but present in the DOM.

When htmx issues a request, it will put a `htmx-request` class onto an element (either the requesting element or another element, if specified). The htmx-request class will cause a child element with the htmx-indicator class on it to transition to an opacity of 1, showing the indicator.

```html
<button hx-get="/click">
    Click Me!
    <img class="htmx-indicator" src="/spinner.gif" alt="Loading...">
</button>
```

Here we have a button. When it is clicked the htmx-request class will be added to it, which will reveal the spinner gif element.

While the htmx-indicator class uses opacity to hide and show the progress indicator, if you would prefer another mechanism you can create your own CSS transition like so:

```css
.htmx-indicator{
    display:none;
}
.htmx-request .htmx-indicator{
    display:inline;
}
.htmx-request.htmx-indicator{
    display:inline;
}
```

If you want the htmx-request class added to a different element, you can use the `hx-indicator` attribute with a CSS selector to do so:

```html
<div>
    <button hx-get="/click" hx-indicator="#indicator">
        Click Me!
    </button>
    <img id="indicator" class="htmx-indicator" src="/spinner.gif" alt="Loading..."/>
</div>
```

You can also add the `disabled` attribute to elements for the duration of a request by using the `hx-disabled-elt` attribute.

---

## Targets

If you want the response to be loaded into a different element other than the one that made the request, you can use the `hx-target` attribute, which takes a CSS selector. Looking back at our Live Search example:

```html
<input type="text" name="q"
    hx-get="/trigger_delay"
    hx-trigger="keyup delay:500ms changed"
    hx-target="#search-results"
    placeholder="Search...">
<div id="search-results"></div>
```

You can see that the results from the search are going to be loaded into div#search-results, rather than into the input tag.

### Extended CSS Selectors

`hx-target`, and most attributes that take a CSS selector, support an "extended" CSS syntax:

- You can use the `this` keyword, which indicates that the element that the hx-target attribute is on is the target
- The `closest <CSS selector>` syntax will find the closest ancestor element or itself, that matches the given CSS selector. (e.g. `closest tr` will target the closest table row to the element)
- The `next <CSS selector>` syntax will find the next element in the DOM matching the given CSS selector.
- The `previous <CSS selector>` syntax will find the previous element in the DOM matching the given CSS selector.
- `find <CSS selector>` which will find the first child descendant element that matches the given CSS selector. (e.g `find tr` would target the first child descendant row to the element)

In addition, a CSS selector may be wrapped in `<` and `/>` characters, mimicking the query literal syntax of hyperscript.

Relative targets like this can be useful for creating flexible user interfaces without peppering your DOM with lots of id attributes.

---

## Swapping

htmx offers a few different ways to swap the HTML returned into the DOM. By default, the content replaces the innerHTML of the target element. You can modify this by using the `hx-swap` attribute with any of the following values:

| Name        | Description                                                                      |
|-------------|----------------------------------------------------------------------------------|
| innerHTML   | the default, puts the content inside the target element                         |
| outerHTML   | replaces the entire target element with the returned content                    |
| afterbegin  | prepends the content before the first child inside the target                  |
| beforebegin | prepends the content before the target in the target's parent element           |
| beforeend   | appends the content after the last child inside the target                     |
| afterend    | appends the content after the target in the target's parent element            |
| delete      | deletes the target element regardless of the response                           |
| none        | does not append content from response (Out of Band Swaps and Response Headers will still be processed) |

### Morph Swaps

In addition to the standard swap mechanisms above, htmx also supports morphing swaps, via extensions. Morphing swaps attempt to merge new content into the existing DOM, rather than simply replacing it. They often do a better job preserving things like focus, video state, etc. by mutating existing nodes in-place during the swap operation, at the cost of more CPU.

The following extensions are available for morph-style swaps:

- **Idiomorph** - A morphing algorithm created by the htmx developers.
- **Morphdom Swap** - Based on the morphdom, the original DOM morphing library.
- **Alpine-morph** - Based on the alpine morph plugin, plays well with alpine.js

### View Transitions

The new, experimental View Transitions API gives developers a way to create an animated transition between different DOM states. It is still in active development and is not available in all browsers, but htmx provides a way to work with this new API that falls back to the non-transition mechanism if the API is not available in a given browser.

You can experiment with this new API using the following approaches:

- Set the `htmx.config.globalViewTransitions` config variable to true to use transitions for all swaps
- Use the `transition:true` option in the `hx-swap` attribute

If an element swap is going to be transitioned due to either of the above configurations, you may catch the `htmx:beforeTransition` event and call `preventDefault()` on it to cancel the transition.

View Transitions can be configured using CSS, as outlined in the Chrome documentation for the feature.

### Swap Options

The `hx-swap` attribute supports many options for tuning the swapping behavior of htmx. For example, by default htmx will swap in the title of a title tag found anywhere in the new content. You can turn this behavior off by setting the `ignoreTitle` modifier to true:

```html
<button hx-post="/like" hx-swap="outerHTML ignoreTitle:true">Like</button>
```

The modifiers available on `hx-swap` are:

| Option       | Description                                                                              |
|--------------|------------------------------------------------------------------------------------------|
| transition   | true or false, whether to use the view transition API for this swap                     |
| swap         | The swap delay to use (e.g. 100ms) between when old content is cleared and new content is inserted |
| settle       | The settle delay to use (e.g. 100ms) between when new content is inserted and when it is settled |
| ignoreTitle  | If set to true, any title found in the new content will be ignored and not update the document title |
| scroll       | top or bottom, will scroll the target element to its top or bottom                       |
| show         | top or bottom, will scroll the target element's top or bottom into view                  |

All swap modifiers appear after the swap style is specified, and are colon-separated.

---

## Synchronization

Often you want to coordinate the requests between two elements. For example, you may want a request from one element to supersede the request of another element, or to wait until the other element's request has finished.

htmx offers a `hx-sync` attribute to help you accomplish this.

Consider a race condition between a form submission and an individual input's validation request in this HTML:

```html
<form hx-post="/store">
    <input id="title" name="title" type="text"
        hx-post="/validate"
        hx-trigger="change">
    <button type="submit">Submit</button>
</form>
```

Without using `hx-sync`, filling out the input and immediately submitting the form triggers two parallel requests to /validate and /store.

Using `hx-sync="closest form:abort"` on the input will watch for requests on the form and abort the input's request if a form request is present or starts while the input request is in flight:

```html
<form hx-post="/store">
    <input id="title" name="title" type="text"
        hx-post="/validate"
        hx-trigger="change"
        hx-sync="closest form:abort">
    <button type="submit">Submit</button>
</form>
```

This resolves the synchronization between the two elements in a declarative way.

htmx also supports a programmatic way to cancel requests: you can send the `htmx:abort` event to an element to cancel any in-flight requests:

```html
<button id="request-button" hx-post="/example">
    Issue Request
</button>
<button onclick="htmx.trigger('#request-button', 'htmx:abort')">
    Cancel Request
</button>
```

---

## CSS Transitions

htmx makes it easy to use CSS Transitions without JavaScript. Consider this HTML content:

```html
<div id="div1">Original Content</div>
```

Imagine this content is replaced by htmx via an ajax request with this new content:

```html
<div id="div1" class="red">New Content</div>
```

Note two things:

- The div has the same id in the original and in the new content
- The red class has been added to the new content

Given this situation, we can write a CSS transition from the old state to the new state:

```css
.red {
    color: red;
    transition: all ease-in 1s ;
}
```

When htmx swaps in this new content, it will do so in such a way that the CSS transition will apply to the new content, giving you a nice, smooth transition to the new state.

So, in summary, all you need to do to use CSS transitions for an element is keep its id stable across requests!

### Details

To understand how CSS transitions actually work in htmx, you must understand the underlying swap & settle model that htmx uses.

When new content is received from a server, before the content is swapped in, the existing content of the page is examined for elements that match by the id attribute. If a match is found for an element in the new content, the attributes of the old content are copied onto the new element before the swap occurs. The new content is then swapped in, but with the old attribute values. Finally, the new attribute values are swapped in, after a "settle" delay (20ms by default). A little crazy, but this is what allows CSS transitions to work without any JavaScript by the developer.

---

## Out of Band Swaps

If you want to swap content from a response directly into the DOM by using the id attribute you can use the `hx-swap-oob` attribute in the response html:

```html
<div id="message" hx-swap-oob="true">Swap me directly!</div>
```

Additional Content

In this response, div#message would be swapped directly into the matching DOM element, while the additional content would be swapped into the target in the normal manner.

You can use this technique to "piggy-back" updates on other requests.

### Troublesome Tables

Table elements can be problematic when combined with out of band swaps, because, by the HTML spec, many can't stand on their own in the DOM (e.g. `<tr>` or `<td>`).

To avoid this issue you can use a template tag to encapsulate these elements:

```html
<template>
  <tr id="message" hx-swap-oob="true"><td>Joe</td><td>Smith</td></tr>
</template>
```

### Selecting Content To Swap

If you want to select a subset of the response HTML to swap into the target, you can use the `hx-select` attribute, which takes a CSS selector and selects the matching elements from the response.

You can also pick out pieces of content for an out-of-band swap by using the `hx-select-oob` attribute, which takes a list of element IDs to pick out and swap.

### Preserving Content During A Swap

If there is content that you wish to be preserved across swaps (e.g. a video player that you wish to remain playing even if a swap occurs) you can use the `hx-preserve` attribute on the elements you wish to be preserved.

---

## Parameters

By default, an element that causes a request will include its value if it has one. If the element is a form it will include the values of all inputs within it.

As with HTML forms, the name attribute of the input is used as the parameter name in the request that htmx sends.

Additionally, if the element causes a non-GET request, the values of all the inputs of the associated form will be included (typically this is the nearest enclosing form, but could be different if e.g. `<button form="associated-form">` is used).

If you wish to include the values of other elements, you can use the `hx-include` attribute with a CSS selector of all the elements whose values you want to include in the request.

If you wish to filter out some parameters you can use the `hx-params` attribute.

Finally, if you want to programmatically modify the parameters, you can use the `htmx:configRequest` event.

### File Upload

If you wish to upload files via an htmx request, you can set the `hx-encoding` attribute to `multipart/form-data`. This will use a FormData object to submit the request, which will properly include the file in the request.

Note that depending on your server-side technology, you may have to handle requests with this type of body content very differently.

Note that htmx fires a `htmx:xhr:progress` event periodically based on the standard progress event during upload, which you can hook into to show the progress of the upload.

### Extra Values

You can include extra values in a request using the `hx-vals` (name-expression pairs in JSON format) and `hx-vars` attributes (comma-separated name-expression pairs that are dynamically computed).

---

## Confirming Requests

Often you will want to confirm an action before issuing a request. htmx supports the `hx-confirm` attribute, which allows you to confirm an action using a simple JavaScript dialog:

```html
<button hx-delete="/account" hx-confirm="Are you sure you wish to delete your account?">
    Delete My Account
</button>
```

Using events you can implement more sophisticated confirmation dialogs.

### Confirming Requests Using Events

Another option to do confirmation with is via the `htmx:confirm` event. This event is fired on every trigger for a request (not just on elements that have a hx-confirm attribute) and can be used to implement asynchronous confirmation of the request.

Here is an example using sweet alert on any element with a `confirm-with-sweet-alert='true'` attribute on it:

```javascript
document.body.addEventListener('htmx:confirm', function(evt) {
  if (evt.target.matches("[confirm-with-sweet-alert='true']")) {
    evt.preventDefault();
    swal({
      title: "Are you sure?",
      text: "Are you sure you are sure?",
      icon: "warning",
      buttons: true,
      dangerMode: true,
    }).then((confirmed) => {
      if (confirmed) {
        evt.detail.issueRequest();
      }
    });
  }
});
```

---

## Attribute Inheritance

Most attributes in htmx are inherited: they apply to the element they are on as well as any children elements. This allows you to "hoist" attributes up the DOM to avoid code duplication. Consider the following htmx:

```html
<button hx-delete="/account" hx-confirm="Are you sure?">
    Delete My Account
</button>
<button hx-put="/account" hx-confirm="Are you sure?">
    Update My Account
</button>
```

Here we have a duplicate hx-confirm attribute. We can hoist this attribute to a parent element:

```html
<div hx-confirm="Are you sure?">
    <button hx-delete="/account">
        Delete My Account
    </button>
    <button hx-put="/account">
        Update My Account
    </button>
</div>
```

This hx-confirm attribute will now apply to all htmx-powered elements within it.

Sometimes you wish to undo this inheritance. Consider if we had a cancel button to this group, but didn't want it to be confirmed. We could add an unset directive on it like so:

```html
<div hx-confirm="Are you sure?">
    <button hx-delete="/account">
        Delete My Account
    </button>
    <button hx-put="/account">
        Update My Account
    </button>
    <button hx-confirm="unset" hx-get="/">
        Cancel
    </button>
</div>
```

The top two buttons would then show a confirm dialog, but the bottom cancel button would not.

Inheritance can be disabled on a per-element and per-attribute basis using the `hx-disinherit` attribute.

If you wish to disable attribute inheritance entirely, you can set the `htmx.config.disableInheritance` configuration variable to true. This will disable inheritance as a default, and allow you to specify inheritance explicitly with the `hx-inherit` attribute.

---

## Boosting

Htmx supports "boosting" regular HTML anchors and forms with the `hx-boost` attribute. This attribute will convert all anchor tags and forms into AJAX requests that, by default, target the body of the page.

Here is an example:

```html
<div hx-boost="true">
    <a href="/blog">Blog</a>
</div>
```

The anchor tag in this div will issue an AJAX GET request to /blog and swap the response into the body tag.

---

## Progressive Enhancement

A feature of `hx-boost` is that it degrades gracefully if JavaScript is not enabled: the links and forms continue to work, they simply don't use ajax requests. This is known as Progressive Enhancement, and it allows a wider audience to use your site's functionality.

Other htmx patterns can be adapted to achieve progressive enhancement as well, but they will require more thought.

Consider the active search example. As it is written, it will not degrade gracefully: someone who does not have JavaScript enabled will not be able to use this feature. This is done for simplicity's sake, to keep the example as brief as possible.

However, you could wrap the htmx-enhanced input in a form element:

```html
<form action="/search" method="POST">
    <input class="form-control" type="search"
        name="search" placeholder="Begin typing to search users..."
        hx-post="/search"
        hx-trigger="keyup changed delay:500ms, search"
        hx-target="#search-results"
        hx-indicator=".htmx-indicator">
</form>
```

With this in place, JavaScript-enabled clients would still get the nice active-search UX, but non-JavaScript enabled clients would be able to hit the enter key and still search. Even better, you could add a "Search" button as well. You would then need to update the form with an hx-post that mirrored the action attribute, or perhaps use hx-boost on it.

You would need to check on the server side for the HX-Request header to differentiate between an htmx-driven and a regular request, to determine exactly what to render to the client.

Other patterns can be adapted similarly to achieve the progressive enhancement needs of your application.

As you can see, this requires more thought and more work. It also rules some functionality entirely out of bounds. These tradeoffs must be made by you, the developer, with respect to your projects goals and audience.

### Accessibility

Accessibility is a concept closely related to progressive enhancement. Using progressive enhancement techniques such as hx-boost will make your htmx application more accessible to a wide array of users.

htmx-based applications are very similar to normal, non-AJAX driven web applications because htmx is HTML-oriented.

As such, the normal HTML accessibility recommendations apply. For example:

- Use semantic HTML as much as possible (i.e. the right tags for the right things)
- Ensure focus state is clearly visible
- Associate text labels with all form fields
- Maximize the readability of your application with appropriate fonts, contrast, etc.

---

## Web Sockets & SSE

Web Sockets and Server Sent Events (SSE) are supported via extensions. Please see the SSE extension and WebSocket extension pages to learn more.

---

## History Support

Htmx provides a simple mechanism for interacting with the browser history API:

If you want a given element to push its request URL into the browser navigation bar and add the current state of the page to the browser's history, include the `hx-push-url` attribute:

```html
<a hx-get="/blog" hx-push-url="true">Blog</a>
```

When a user clicks on this link, htmx will snapshot the current DOM and store it before it makes a request to /blog. It then does the swap and pushes a new location onto the history stack.

When a user hits the back button, htmx will retrieve the old content from storage and swap it back into the target, simulating "going back" to the previous state. If the location is not found in the cache, htmx will make an ajax request to the given URL, with the header HX-History-Restore-Request set to true, and expects back the HTML needed for the entire page. You should always set `htmx.config.historyRestoreAsHxRequest` to false to prevent the HX-Request header which can then be safely used to respond with partials. Alternatively, if the `htmx.config.refreshOnHistoryMiss` config variable is set to true, it will issue a hard browser refresh.

> NOTE: If you push a URL into the history, you must be able to navigate to that URL and get a full page back! A user could copy and paste the URL into an email, or new tab. Additionally, htmx will need the entire page when restoring history if the page is not in the history cache.

### Specifying History Snapshot Element

By default, htmx will use the body to take and restore the history snapshot from. This is usually the right thing, but if you want to use a narrower element for snapshotting you can use the `hx-history-elt` attribute to specify a different one.

> Careful: this element will need to be on all pages or restoring from history won't work reliably.

### Undoing DOM Mutations By 3rd Party Libraries

If you are using a 3rd party library and want to use the htmx history feature, you will need to clean up the DOM before a snapshot is taken. Let's consider the Tom Select library, which makes select elements a much richer user experience. Let's set up TomSelect to turn any input element with the `.tomselect` class into a rich select element.

First we need to initialize elements that have the class in new content:

```javascript
htmx.onLoad(function (target) {
    var editors = target.querySelectorAll(".tomselect")
            .forEach(elt => new TomSelect(elt))
});
```

This will create a rich selector for all input elements that have the `.tomselect` class on it. However, it mutates the DOM and we don't want that mutation saved to the history cache, since TomSelect will be reinitialized when the history content is loaded back into the screen.

To deal with this, we need to catch the `htmx:beforeHistorySave` event and clean out the TomSelect mutations by calling `destroy()` on them:

```javascript
htmx.on('htmx:beforeHistorySave', function() {
    document.querySelectorAll('.tomSelect')
            .forEach(elt => elt.tomselect.destroy())
})
```

This will revert the DOM to the original HTML, thus allowing for a clean snapshot.

### Disabling History Snapshots

History snapshotting can be disabled for a URL by setting the `hx-history` attribute to false on any element in the current document, or any html fragment loaded into the current document by htmx. This can be used to prevent sensitive data entering the localStorage cache, which can be important for shared-use / public computers. History navigation will work as expected, but on restoration the URL will be requested from the server instead of the local history cache.

---

## Requests & Responses

Htmx expects responses to the AJAX requests it makes to be HTML, typically HTML fragments (although a full HTML document, matched with a hx-select tag can be useful too). Htmx will then swap the returned HTML into the document at the target specified and with the swap strategy specified.

Sometimes you might want to do nothing in the swap, but still perhaps trigger a client side event (see below).

For this situation, by default, you can return a 204 - No Content response code, and htmx will ignore the content of the response.

In the event of an error response from the server (e.g. a 404 or a 501), htmx will trigger the `htmx:responseError` event, which you can handle.

In the event of a connection error, the `htmx:sendError` event will be triggered.

### Configuring Response Handling

You can configure the above behavior of htmx by mutating or replacing the `htmx.config.responseHandling` array. This object is a collection of JavaScript objects defined like so:

```javascript
responseHandling: [
    {code:"204", swap: false},   // 204 - No Content by default does nothing, but is not an error
    {code:"[23]..", swap: true}, // 200 & 300 responses are non-errors and are swapped
    {code:"[45]..", swap: false, error:true}, // 400 & 500 responses are not swapped and are errors
    {code:"...", swap: false}    // catch all for any other response code
]
```

When htmx receives a response it will iterate in order over the `htmx.config.responseHandling` array and test if the code property of a given object, when treated as a Regular Expression, matches the current response. If an entry does match the current response code, it will be used to determine if and how the response will be processed.

The fields available for response handling configuration on entries in this array are:

- `code` - a String representing a regular expression that will be tested against response codes.
- `swap` - true if the response should be swapped into the DOM, false otherwise
- `error` - true if htmx should treat this response as an error
- `ignoreTitle` - true if htmx should ignore title tags in the response
- `select` - A CSS selector to use to select content from the response
- `target` - A CSS selector specifying an alternative target for the response
- `swapOverride` - An alternative swap mechanism for the response

### Configuring Response Handling Examples

As an example of how to use this configuration, consider a situation when a server-side framework responds with a 422 - Unprocessable Entity response when validation errors occur. By default, htmx will ignore the response, since it matches the Regular Expression `[45]...`.

Using the meta config mechanism for configuring responseHandling, we could add the following config:

```html
<meta
    name="htmx-config"
    content='{
        "responseHandling":[
            {"code":"204", "swap": false},
            {"code":"[23]..", "swap": true},
            {"code":"422", "swap": true},
            {"code":"[45]..", "swap": false, "error":true},
            {"code":"...", "swap": true}
        ]
    }'
/>
```

If you wanted to swap everything, regardless of HTTP response code, you could use this configuration:

```html
<meta name="htmx-config" content='{"responseHandling": [{"code":".*", "swap": true}]}' />
```

### CORS

When using htmx in a cross origin context, remember to configure your web server to set Access-Control headers in order for htmx headers to be visible on the client side.

- `Access-Control-Allow-Headers` (for request headers)
- `Access-Control-Expose-Headers` (for response headers)

---

## Request Headers

htmx includes a number of useful headers in requests:

| Header                    | Description                                                                         |
|---------------------------|-------------------------------------------------------------------------------------|
| HX-Boosted                | indicates that the request is via an element using hx-boost                        |
| HX-Current-URL            | the current URL of the browser                                                      |
| HX-History-Restore-Request| "true" if the request is for history restoration after a miss in the local history cache |
| HX-Prompt                 | the user response to an hx-prompt                                                   |
| HX-Request                | always "true" except on history restore requests if `htmx.config.historyRestoreAsHxRequest' disabled |
| HX-Target                 | the id of the target element if it exists                                           |
| HX-Trigger-Name           | the name of the triggered element if it exists                                      |
| HX-Trigger                | the id of the triggered element if it exists                                        |

---

## Response Headers

htmx supports some htmx-specific response headers:

- **HX-Location** - allows you to do a client-side redirect that does not do a full page reload
- **HX-Push-Url** - pushes a new url into the history stack
- **HX-Redirect** - can be used to do a client-side redirect to a new location
- **HX-Refresh** - if set to "true" the client-side will do a full refresh of the page
- **HX-Replace-Url** - replaces the current URL in the location bar
- **HX-Reswap** - allows you to specify how the response will be swapped. See hx-swap for possible values
- **HX-Retarget** - a CSS selector that updates the target of the content update to a different element on the page
- **HX-Reselect** - a CSS selector that allows you to choose which part of the response is used to be swapped in. Overrides an existing hx-select on the triggering element
- **HX-Trigger** - allows you to trigger client-side events
- **HX-Trigger-After-Settle** - allows you to trigger client-side events after the settle step
- **HX-Trigger-After-Swap** - allows you to trigger client-side events after the swap step

Submitting a form via htmx has the benefit of no longer needing the Post/Redirect/Get Pattern. After successfully processing a POST request on the server, you don't need to return a HTTP 302 (Redirect). You can directly return the new HTML fragment.

Also the response headers above are not provided to htmx for processing with 3xx Redirect response codes like HTTP 302 (Redirect). Instead, the browser will intercept the redirection internally and return the headers and response from the redirected URL. Where possible use alternative response codes like 200 to allow returning of these response headers.

---

## Request Order of Operations

The order of operations in a htmx request are:

1. The element is triggered and begins a request
2. Values are gathered for the request
3. The htmx-request class is applied to the appropriate elements
4. The request is then issued asynchronously via AJAX
5. Upon getting a response the target element is marked with the htmx-swapping class
6. An optional swap delay is applied (see the hx-swap attribute)
7. The actual content swap is done
8. the htmx-swapping class is removed from the target
9. the htmx-added class is added to each new piece of content
10. the htmx-settling class is applied to the target
11. A settle delay is done (default: 20ms)
12. The DOM is settled
13. the htmx-settling class is removed from the target
14. the htmx-added class is removed from each new piece of content

You can use the htmx-swapping and htmx-settling classes to create CSS transitions between pages.

---

## Validation

Htmx integrates with the HTML5 Validation API and will not issue a request for a form if a validatable input is invalid. This is true for both AJAX requests as well as WebSocket sends.

Htmx fires events around validation that can be used to hook in custom validation and error handling:

- `htmx:validation:validate` - called before an element's `checkValidity()` method is called. May be used to add in custom validation logic
- `htmx:validation:failed` - called when `checkValidity()` returns false, indicating an invalid input
- `htmx:validation:halted` - called when a request is not issued due to validation errors. Specific errors may be found in the `event.detail.errors` object

Non-form elements do not validate before they make requests by default, but you can enable validation by setting the `hx-validate` attribute to "true".

Normal browser form submission alerts the user of any validation errors automatically and auto focuses on the first invalid input. For backwards compatibility reasons htmx does not report the validation to the users by default and you should always enable this option by setting `htmx.config.reportValidityOfForms` to true to restore the default browser behavior.

### Validation Example

Here is an example of an input that uses the hx-on attribute to catch the `htmx:validation:validate` event and require that the input have the value foo:

```html
<form id="example-form" hx-post="/test">
    <input name="example"
           onkeyup="this.setCustomValidity('')"
           hx-on:htmx:validation:validate="if(this.value != 'foo') {
                    this.setCustomValidity('Please enter the value foo')
                    htmx.find('#example-form').reportValidity()
                }">
</form>
```

Note that all client side validations must be re-done on the server side, as they can always be bypassed.

---

## Animations

Htmx allows you to use CSS transitions in many situations using only HTML and CSS.

Please see the Animation Guide for more details on the options available.

---

## Extensions

htmx provides an extensions mechanism that allows you to customize the libraries' behavior. Extensions are defined in JavaScript and then enabled via the `hx-ext` attribute.

### Core Extensions

htmx supports a few "core" extensions, which are supported by the htmx development team:

- **head-support** - support for merging head tag information (styles, etc.) in htmx requests
- **htmx-1-compat** - restores htmx 1 defaults & functionality
- **idiomorph** - supports the morph swap strategy using idiomorph
- **preload** - allows you to preload content for better performance
- **response-targets** - allows you to target elements based on HTTP response codes (e.g. 404)
- **sse** - support for Server Sent Events
- **ws** - support for Web Sockets

### Installing Extensions

The fastest way to install htmx extensions created by others is to load them via a CDN. Remember to always include the core htmx library before the extensions and enable the extension. For example, if you would like to use the response-targets extension, you can add this to your head tag:

```html
<head>
    <script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js" integrity="sha384-/TgkGk7p307TH7EXJDuUlgG3Ce1UVolAOFopFekQkkXihi5u/6OCvVKyz1W+idaz" crossorigin="anonymous"></script>
    <script src="https://cdn.jsdelivr.net/npm/htmx-ext-response-targets@2.0.4" integrity="sha384-T41oglUPvXLGBVyRdZsVRxNWnOOqCynaPubjUVjxhsjFTKrFJGEMm3/0KGmNQ+Pg" crossorigin="anonymous"></script>
</head>
<body hx-ext="extension-name">
    ...
</body>
```

While the CDN approach is simple, you may want to consider not using CDNs in production.

For npm-style build systems, you can install htmx extensions via npm:

```bash
npm install htmx-ext-extension-name
```

If you are using a bundler to manage your JavaScript (e.g. Webpack, Rollup):

1. Install htmx.org and htmx-ext-extension-name via npm
2. Import both packages to your index.js

```javascript
import 'htmx.org';
import 'htmx-ext-extension-name';
```

> Note: Idiomorph does not follow the naming convention of htmx extensions. Use idiomorph instead of htmx-ext-idiomorph.

### Enabling Extensions

To enable an extension, add a `hx-ext="extension-name"` attribute to `<body>` or another HTML element. The extension will be applied to all child elements.

```html
<body hx-ext="response-targets">
    ...
    <button hx-post="/register" hx-target="#response-div" hx-target-404="#not-found">
        Register!
    </button>
    <div id="response-div"></div>
    <div id="not-found"></div>
    ...
</body>
```

### Creating Extensions

If you are interested in adding your own extension to htmx, please see the extension docs.

---

## Events & Logging

Htmx has an extensive events mechanism, which doubles as the logging system.

If you want to register for a given htmx event you can use:

```javascript
document.body.addEventListener('htmx:load', function(evt) {
    myJavascriptLib.init(evt.detail.elt);
});
```

or, if you would prefer, you can use the following htmx helper:

```javascript
htmx.on("htmx:load", function(evt) {
    myJavascriptLib.init(evt.detail.elt);
});
```

The `htmx:load` event is fired every time an element is loaded into the DOM by htmx, and is effectively the equivalent to the normal load event.

### Common Uses for htmx Events

#### Initialize A 3rd Party Library With Events

Using the `htmx:load` event to initialize content is so common that htmx provides a helper function:

```javascript
htmx.onLoad(function(target) {
    myJavascriptLib.init(target);
});
```

#### Configure a Request With Events

You can handle the `htmx:configRequest` event in order to modify an AJAX request before it is issued:

```javascript
document.body.addEventListener('htmx:configRequest', function(evt) {
    evt.detail.parameters['auth_token'] = getAuthToken();
    evt.detail.headers['Authentication-Token'] = getAuthToken();
});
```

#### Modifying Swapping Behavior With Events

You can handle the `htmx:beforeSwap` event in order to modify the swap behavior of htmx:

```javascript
document.body.addEventListener('htmx:beforeSwap', function(evt) {
    if(evt.detail.xhr.status === 404){
        alert("Error: Could Not Find Resource");
    } else if(evt.detail.xhr.status === 422){
        evt.detail.shouldSwap = true;
        evt.detail.isError = false;
    } else if(evt.detail.xhr.status === 418){
        evt.detail.shouldSwap = true;
        evt.detail.target = htmx.find("#teapot");
    }
});
```

### Event Naming

Note that all events are fired with two different names:

- Camel Case
- Kebab Case

So, for example, you can listen for `htmx:afterSwap` or for `htmx:after-swap`. This facilitates interoperability with other libraries.

### Logging

If you set a logger at `htmx.logger`, every event will be logged:

```javascript
htmx.logger = function(elt, event, data) {
    if(console) {
        console.log(event, elt, data);
    }
}
```

---

## Debugging

Declarative and event driven programming with htmx can be a wonderful and highly productive activity, but one disadvantage when compared with imperative approaches is that it can be trickier to debug.

The first debugging tool you can use is the `htmx.logAll()` method:

```javascript
htmx.logAll();
```

You can also use the `monitorEvents()` method available in the browser console:

```javascript
monitorEvents(htmx.find("#theElement"));
```

> Note that this only works from the console, you cannot embed it in a script tag on your page.

---

## Creating Demos

To facilitate easy demo creation, htmx hosts a demo script site that will install:

- htmx
- hyperscript
- a request mocking library

Simply add the following script tag to your demo:

```html
<script src="https://demo.htmx.org"></script>
```

This helper allows you to add mock responses by adding template tags with a url attribute to indicate which URL. The response for that url will be the innerHTML of the template, making it easy to construct mock responses. You can add a delay to the response with a delay attribute.

You may embed simple expressions in the template with the `${}` syntax.

Example:

```html
<!-- load demo environment -->
<script src="https://demo.htmx.org"></script>

<!-- post to /foo -->
<button hx-post="/foo" hx-target="#result">
    Count Up
</button>
<output id="result"></output>

<!-- respond to /foo with some dynamic content in a template tag -->
<script>
    globalInt = 0;
</script>
<template url="/foo" delay="500">
    ${globalInt++}
</template>
```

---

## Scripting

While htmx encourages a hypermedia approach to building web applications, it offers many options for client scripting.

htmx recommends a hypermedia-friendly approach to scripting:

- Respect HATEOAS
- Use events to communicate between components
- Use islands to isolate non-hypermedia components from the rest of your application
- Consider inline scripting

The primary integration point between htmx and scripting solutions is the events that htmx sends and can respond to.

### Scripting solutions that pair well with htmx include:

- **VanillaJS** - Simply using the built-in abilities of JavaScript to hook in event handlers
- **AlpineJS** - A rich set of tools for creating sophisticated front end scripts
- **jQuery** - Pairs well with htmx, particularly in older code-bases
- **hyperscript** - An experimental front-end scripting language created by the same team as htmx

---

## The hx-on* Attributes

HTML allows the embedding of inline scripts via the onevent properties, such as onClick:

```html
<button onclick="alert('You clicked me!')">
    Click Me!
</button>
```

htmx offers `hx-on*` attributes that allow you to respond to any event:

```html
<button hx-on:click="alert('You clicked me!')">
    Click Me!
</button>
```

For events like `htmx:config-request`:

```html
<button hx-post="/example"
        hx-on:htmx:config-request="event.detail.parameters.example = 'Hello Scripting!'">
    Post Me!
</button>
```

> Note that HTML attributes are case insensitive. Events that rely on capitalization/camel casing cannot be responded to with hx-on*.

---

## 3rd Party JavaScript

Htmx integrates fairly well with third party libraries. If the library fires events on the DOM, you can use those events to trigger requests from htmx.

Using the `htmx.onLoad` function:

```javascript
htmx.onLoad(function(content) {
    var sortables = content.querySelectorAll(".sortable");
    for (var i = 0; i < sortables.length; i++) {
        var sortable = sortables[i];
        new Sortable(sortable, {
            animation: 150,
            ghostClass: 'blue-background-class'
        });
    }
})
```

If JavaScript adds content to the DOM that has htmx attributes on it, you need to make sure that this content is initialized with the `htmx.process()` function:

```javascript
let myDiv = document.getElementById('my-div')
fetch('http://example.com/movies.json')
    .then(response => response.text())
    .then(data => { myDiv.innerHTML = data; htmx.process(myDiv); } );
```

---

## Web Components

Please see the Web Components Examples page for examples on how to integrate htmx with web components.

---

## Caching

htmx works with standard HTTP caching mechanisms out of the box.

If your server adds the `Last-Modified` HTTP response header to the response for a given URL, the browser will automatically add the `If-Modified-Since` request HTTP header to the next requests to the same URL.

If you are unable to use the Vary header, you can alternatively set the configuration parameter `getCacheBusterParam` to true.

htmx also works with ETag as expected.

---

## Security

htmx allows you to define logic directly in your DOM. A concern with this approach is security: since htmx increases the expressiveness of HTML, if a malicious user is able to inject HTML into your application, they can leverage this expressiveness of htmx to malicious ends.

### Rule 1: Escape All User Content

The first rule of HTML-based web development has always been: do not trust input from the user. You should escape all 3rd party, untrusted content that is injected into your site. This is to prevent XSS attacks.

> Note: If you are injecting raw HTML and doing your own escaping, a best practice is to whitelist the attributes and tags you allow.

### htmx Security Tools

#### hx-disable

The `hx-disable` attribute will prevent processing of all htmx attributes on a given element, and on all elements within it:

```html
<div hx-disable>
    <%= raw(user_content) %>
</div>
```

#### hx-history

You may have pages that have sensitive data that you do not want stored in the users localStorage cache. You can omit a given page from the history cache by including the `hx-history` attribute anywhere on the page, and setting its value to false.

### Configuration Options

htmx provides configuration options related to security:

- `htmx.config.selfRequestsOnly` - if set to true, only requests to the same domain as the current document will be allowed
- `htmx.config.allowScriptTags` - htmx will process `<script>` tags found in new content it loads
- `htmx.config.historyCacheSize` - can be set to 0 to avoid storing any HTML in the localStorage cache
- `htmx.config.allowEval` - can be set to false to disable features that rely on eval

### Events

If you want to allow requests to some domains beyond the current host, you can use the `htmx:validateUrl` event:

```javascript
document.body.addEventListener('htmx:validateUrl', function (evt) {
  if (!evt.detail.sameHost && evt.detail.url.hostname !== "myserver.com") {
    evt.preventDefault();
  }
});
```

### CSP Options

Browsers provide tools for further securing your web application. The most powerful tool available is a Content Security Policy:

```html
<meta http-equiv="Content-Security-Policy" content="default-src 'self';">
```

### CSRF Prevention

htmx can support returning the CSRF token automatically with every request using the `hx-headers` attribute:

```html
<html lang="en" hx-headers='{"X-CSRF-TOKEN": "CSRF_TOKEN_INSERTED_HERE"}'>
    ...
</html>
<body hx-headers='{"X-CSRF-TOKEN": "CSRF_TOKEN_INSERTED_HERE"}'>
    ...
</body>
```

---

## Configuring htmx

Htmx has some configuration options that can be accessed either programmatically or declaratively:

| Config Variable                    | Default                                    | Description                                                                      |
|-------------------------------------|--------------------------------------------|----------------------------------------------------------------------------------|
| htmx.config.historyEnabled           | true                                       | really only useful for testing                                                   |
| htmx.config.historyCacheSize        | 10                                         |                                                                                 |
| htmx.config.refreshOnHistoryMiss    | false                                      | if true htmx will issue a full page refresh on history misses                   |
| htmx.config.defaultSwapStyle       | innerHTML                                  |                                                                                 |
| htmx.config.defaultSwapDelay        | 0                                          |                                                                                 |
| htmx.config.defaultSettleDelay     | 20                                         |                                                                                 |
| htmx.config.includeIndicatorStyles | true                                       | determines if the indicator styles are loaded                                    |
| htmx.config.indicatorClass         | htmx-indicator                             |                                                                                 |
| htmx.config.requestClass           | htmx-request                               |                                                                                 |
| htmx.config.addedClass             | htmx-added                                 |                                                                                 |
| htmx.config.settlingClass          | htmx-settling                              |                                                                                 |
| htmx.config.swappingClass          | htmx-swapping                              |                                                                                 |
| htmx.config.allowEval               | true                                       | can be used to disable htmx's use of eval                                        |
| htmx.config.allowScriptTags         | true                                       | determines if htmx will process script tags found in new content                |
| htmx.config.inlineScriptNonce      | ''                                         | meaning that no nonce will be added to inline scripts                            |
| htmx.config.attributesToSettle     | ["class", "style", "width", "height"]      | the attributes to settle during the settling phase                              |
| htmx.config.inlineStyleNonce       | ''                                         | meaning that no nonce will be added to inline styles                            |
| htmx.config.useTemplateFragments   | false                                      | HTML template tags for parsing content from the server                            |
| htmx.config.wsReconnectDelay       | full-jitter                                |                                                                                 |
| htmx.config.wsBinaryType            | blob                                       | the type of binary data being received over the WebSocket connection             |
| htmx.config.disableSelector        | [hx-disable], [data-hx-disable]            | htmx will not process elements with this attribute on it                         |
| htmx.config.withCredentials        | false                                      | allow cross-site Access-Control requests using credentials                       |
| htmx.config.timeout                | 0                                          | the number of milliseconds a request can take before automatically being terminated |
| htmx.config.scrollBehavior         | 'instant'                                  | the scroll behavior when using the show modifier with hx-swap                    |
| htmx.config.defaultFocusScroll     | false                                       | if the focused element should be scrolled into view                              |
| htmx.config.getCacheBusterParam    | false                                      | if true htmx will append the target element to the GET request                  |
| htmx.config.globalViewTransitions  | false                                       | if true, htmx will use the View Transition API when swapping                    |
| htmx.config.methodsThatUseUrlParams| ["get", "delete"]                          | htmx will format requests with these methods by encoding their parameters in the URL |
| htmx.config.selfRequestsOnly       | true                                       | whether to only allow AJAX requests to the same domain                            |
| htmx.config.ignoreTitle            | false                                      | if true htmx will not update the title of the document                           |
| htmx.config.disableInheritance      | false                                      | disables attribute inheritance in htmx                                           |
| htmx.config.scrollIntoViewOnBoost  | true                                       | whether or not the target of a boosted element is scrolled into the viewport    |
| htmx.config.triggerSpecsCache      | null                                        | the cache to store evaluated trigger specifications into                         |
| htmx.config.responseHandling        | (see docs)                                 | the default Response Handling behavior for response status codes                 |
| htmx.config.allowNestedOobSwaps    | true                                       | whether to process OOB swaps on elements nested within the main response        |
| htmx.config.historyRestoreAsHxRequest| true                                      | Whether to treat history cache miss requests as a "HX-Request"                 |

You can set them directly in JavaScript, or you can use a meta tag:

```html
<meta name="htmx-config" content='{"defaultSwapStyle":"outerHTML"}'>
```

---

## Conclusion

And that's it!

Have fun with htmx! You can accomplish quite a bit without writing a lot of code!
