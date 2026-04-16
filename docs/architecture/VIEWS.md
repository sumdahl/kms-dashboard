# The View Layer (Type-Safe Presentation)

The **View** layer defines how information is displayed to the user. It is built using **Askama** (templates) and **HTMX** (interactivity).

## 🎨 Askama: Type-Safe Templates
All HTML files are stored in `templates/`. These are Jinja2-style templates that are compiled directly into the Rust binary.

1.  **Template Structs:** Every template corresponds to a Rust struct.
    ```rust
    #[derive(askama::Template)]
    #[template(path = "dashboard/roles.html")]
    struct RolesTemplate {
        pub sidebar_pinned: bool,
        pub roles: Vec<RoleDisplay>,
        // ... all other data needed by roles.html
    }
    ```
2.  **Compile-Time Verification:** Askama verifies that every variable used in the HTML exists on the corresponding Rust struct at build-time. This eliminates runtime template errors.

## ⚡ HTMX: Seamless Interactivity
Instead of a heavy JS framework, we use HTMX to perform AJAX-driven partial page updates.

1.  **HTMX Attributes:** Attributes like `hx-get`, `hx-post`, and `hx-target` are used to define triggers and targets directly in the HTML.
2.  **OOB (Out-of-Band) Swaps:** Small UI components (like a success banner) can be "pushed" to different parts of the screen using the `hx-swap-oob` attribute.

## 🛠️ Components & Partials
*   `layout.html`: The base application shell (includes JS/CSS and common head tags).
*   `partials/`: Reusable HTML fragments (sidebar, header, search modals).
*   `*_partial.html`: Stripped-down versions of pages for HTMX requests to minimize data transfer and render times.
