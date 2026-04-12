# KMS Dashboard - Project Context

A high-performance web dashboard built with **Rust (Axum)**, **Tailwind CSS v4**, and **HTMX**. This project provides a modern, responsive interface for a Key Management System (KMS) or similar administrative tasks.

## 🏗️ Architectural Decision Record (ADR)

### 1. Strict MVC Pattern
- **Model:** Defined in `src/models/`, representing database entities and business logic.
- **View:** **Askama** templates in `templates/`. Templates are server-side rendered and include error state logic.
- **Controller:** Handlers in `src/handlers/`. Routes in `src/routes/` only map paths to handlers. Handlers are responsible for data fetching, validation, and returning either a full page template or a partial fragment.

### 2. HTMX & Error Handling
- **HTML-First Validation:** Form validation is performed on the backend. 
- **Validation Failure (422 Unprocessable Entity):** When a form submission fails validation, the server returns a `422` status code along with the HTML for the form (rendered with error messages). 
- **Global Errors:** Global/system errors use `HX-Trigger` or Out-of-Band (OOB) swaps to update a centralized notification area.

### 3. Redirection & Navigation
- **Post-Action Navigation:** Success states for form submissions (like Login/Signup) use the `HX-Redirect` header to trigger a full browser navigation.
- **In-App Navigation:** Use `hx-get` and `hx-push-url` for partial updates.

## ⚠️ Pitfalls & Solutions

1.  **Tailwind Purging:** Always use full class names in templates (e.g., `border-red-500`). Never construct them dynamically like `border-{{ color }}-500`.
2.  **HTMX Partials:** Every handler must detect the `HX-Request` header to decide between returning a partial fragment or a full page layout.
3.  **Form Resubmission:** Always follow the **Post/Redirect/Get** pattern. Use `HX-Redirect` on success.
4.  **Validation Status:** Return `422 Unprocessable Entity` for form errors so HTMX knows to swap the content.
5.  **Askama Macros (Critical):**
    *   **Syntax:** Always use `{% call ui::macro_name(...) %}`. Using `{{ ui::macro_name(...) }}` will cause "unresolved module ui" errors.
    *   **Keywords:** Never use Rust reserved keywords (like `type`, `match`, `let`) as macro parameter names.


## 🚀 Tech Stack
- **Backend:** [Rust](https://www.rust-lang.org/) with the [Axum](https://github.com/tokio-rs/axum) web framework.
- **Templating:** [Askama](https://github.com/djc/askama) for type-safe, compiled templates.
- **Frontend Interactivity:** [HTMX](https://htmx.org/) for AJAX-driven partial updates.
- **Styling:** [Tailwind CSS v4](https://tailwindcss.com/).
