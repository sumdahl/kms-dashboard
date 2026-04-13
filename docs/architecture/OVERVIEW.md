# MVC Architecture Overview

The KMS Dashboard is built on a high-performance **MVC (Model-View-Controller)** architecture using **Rust (Axum)**, **Askama** (type-safe templates), and **HTMX** (for interactive frontend).

## 🎯 Architecture Pillars

1.  **Model (Data & Business Logic):**
    *   Defined in `src/models/`.
    *   Uses `sqlx` for asynchronous database interactions and `serde` for serialization.
    *   Provides strong typing and validation at the data layer.

2.  **View (Type-Safe Presentation):**
    *   Defined in `templates/` and corresponding structs in `src/routes/` or `src/handlers/`.
    *   Uses `Askama` to compile HTML templates into Rust code at build-time.
    *   Leverages `HTMX` for partial page updates (OOB swaps) and a fast, SPA-like user experience.

3.  **Controller (Routing & Logic):**
    *   Defined in `src/routes/` (URL definitions and layout composition) and `src/handlers/` (business logic).
    *   Powered by the `Axum` web framework.
    *   Middleware handles cross-cutting concerns like Auth, RBAC, and error management.

## 🚀 Key Advantages for the Team

*   **Type Safety:** If a template field is missing or typed incorrectly, the project will fail to compile. No runtime "undefined is not a function" errors.
*   **Performance:** Pre-compiled templates and the efficiency of Rust result in sub-millisecond response times.
*   **Simplicity:** HTMX reduces the need for a heavy frontend JavaScript framework (like React or Vue), keeping the logic and state centralized on the server.
*   **Developer Experience:** The "Dual-Template Strategy" allows us to return full pages for initial hits and small fragments for interactive updates using the same codebase.
