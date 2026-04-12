# KMS Dashboard - Project Context

A high-performance web dashboard built with **Rust (Axum)**, **Tailwind CSS v4**, and **HTMX**. This project provides a modern, responsive interface for a Key Management System (KMS) or similar administrative tasks.

## 🚀 Tech Stack

- **Backend:** [Rust](https://www.rust-lang.org/) with the [Axum](https://github.com/tokio-rs/axum) web framework.
- **Templating:** [Askama](https://github.com/djc/askama) for type-safe, compiled Jinja2-like templates.
- **Frontend Interactivity:** [HTMX](https://htmx.org/) for AJAX-driven partial page updates.
- **Styling:** [Tailwind CSS v4](https://tailwindcss.com/) via PostCSS.
- **Live Reload:** `tower-livereload` for seamless development.

## 🏗️ Architecture & Structure

- `src/main.rs`: The entry point, containing Axum routes, middleware, and request handlers.
- `templates/`: Contains all HTML templates.
    - `layout.html`: Base layout with the app shell.
    - `dashboard/`: Page-specific templates (e.g., `home.html`).
    - `partials/`: Reusable components (sidebar, header, banner).
- `static/`: Static assets served by the backend.
    - `css/output.css`: The compiled Tailwind CSS bundle.
    - `js/sidebar.js`: Vanilla JS logic for the dynamic sidebar.
- `src/styles/input.css`: The source CSS file where Tailwind v4 is initialized.
- `build.rs`: A custom Rust build script that automatically triggers the Tailwind CSS build (`npm run build:css`) during `cargo build`.

## 🛠️ Key Commands

### Development
```bash
# Full development environment with auto-reload (watches Rust, templates, and styles)
npm run dev

# Install all dependencies
cargo build
npm install
```

### Build & Utility
```bash
# Build Tailwind CSS
npm run build:css

# Watch for CSS changes
npm run watch:css

# Copy HTMX to static directory (one-time setup)
npm run copy-htmx
```

## 📋 Development Conventions

- **Type-Safe Templates:** Use Askama structs in `src/main.rs` to pass data to templates. Templates are verified at compile time.
- **HTMX for UI Updates:** Prefer HTMX attributes (`hx-delete`, `hx-post`, etc.) for small, dynamic UI updates (like dismissing a banner or updating a list) instead of full-page reloads or heavy JS frameworks.
- **Utility-First CSS:** Use Tailwind CSS v4 classes for styling. Custom styles should be added to `src/styles/input.css` using `@theme` or standard CSS.
- **Sidebar Logic:** The sidebar uses a "Three-Zone" model (Hover, Dead Zone, Toggle) managed by `static/js/sidebar.js`. It communicates pin state back to the server via `POST /ui/sidebar/pin`.
- **Static Assets:** New static files should be placed in `static/` and referenced via `/static/...` in templates.

## 🔗 Routes Overview
- `GET /`: Main dashboard home page.
- `POST /ui/sidebar/pin`: Persists the sidebar's pinned/unpinned state.
- `GET /ui/global-message?message=…&kind=…`: Returns OOB HTML to append a global message (`success` | `error` | `warning` | `info`).
