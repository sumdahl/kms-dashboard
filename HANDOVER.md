# KMS Dashboard - Project Handover & Progress Report

This document summarizes the current status of the "Strict MVC" refactor. It is intended for the next agent to understand the architectural rules and the work remaining.

## 🏗️ Architectural Pattern: "Smart Backend, Dumb Frontend"
We are following a strict MVC pattern using **Axum (Rust)**, **Askama (Templates)**, and **HTMX**.

1.  **Backend-Driven State**: All UI state (e.g., Is the sidebar pinned? Is this form valid?) is stored on the server (Cookies/Session) and rendered into HTML.
2.  **HTML Over JSON**: We prefer `application/x-www-form-urlencoded` over `application/json` for form submissions.
3.  **HTMX Swapping**: 
    *   Success: Use `HX-Redirect` header for full navigation.
    *   Failure: Return `422 Unprocessable Entity` with the form partial containing error messages.
4.  **Macros as Components**: We use `templates/macros/ui.html` to create reusable UI components with "props."
5.  **Global Errors**: `src/error.rs` contains a `smart_response` method that sends a partial HTML banner (`templates/partials/error_banner.html`) for HTMX requests.

---

## ✅ Completed Work

### 1. Core Infrastructure
*   [x] **ADR Documented**: `GEMINI.md` contains the rules for status codes, redirects, and Tailwind classes.
*   [x] **Global Error System**: `AppError` is HTMX-aware and supports OOB (Out-of-Band) error banners.
*   [x] **Form Macros**: Created standardized `input_field` and `select_field` macros.
*   [x] **Component Library**: Created `ui.html` with **Table Header** and **Badge** macros.

### 2. Authentication Module (Fully Refactored)
*   [x] **Login Flow**: Standard Form POST -> Backend Validation -> HTMX Swap or Redirect.
*   [x] **Signup Flow**: Standard Form POST -> Backend Validation -> HTMX Swap or Redirect.
*   [x] **Zero JS**: Removed all manual `fetch()` and banner-toggling scripts from auth templates.
*   [x] **Password Reset**: `forgot_password` and `reset_password` now return HTML/Redirects.

### 3. Dashboard Module (Fully Refactored)
*   [x] **Create Role Wizard**: Moved to backend-driven 3-step flow (Identity -> Permissions -> Review).
*   [x] **Assign Role Form**: Fully refactored to use `ui::input_field` and backend validation.
*   [x] **User List**: Refactored to server-side rendering with `ui::badge` and `ui::table_header`.
*   [x] **Role List**: Refactored to server-side rendering with live summary stats.

### 4. Sidebar & UI Fixes
*   [x] **Persistent State**: Moved pinned state to a `sidebar_pinned` cookie.
*   [x] **Sidebar Visibility**: Fixed bug where sidebar wouldn't stay expanded when pinned.
*   [x] **Logo Cleanup**: Fixed logo clipping and scaling issues.

---

## ⏳ Work Remaining

### 1. Final Component Polish
*   [ ] **Quick Create Role**: Refactor the single-page "Quick" role creation to match the Wizard's new standard.
*   [ ] **Role Detail Page**: Minor UI polish to match the new component styling.

### 2. Cleanup
*   [ ] **JS Deletion**: Delete `static/js/json-enc.js` once the Quick Create form is moved to standard Form POST.
*   [ ] **Route Cleanup**: Ensure all routes in `src/routes/mod.rs` point only to controllers.

---

## ⚠️ Lessons Learned (The "Askama Trap")
1.  **Macro Syntax**: Always use `{% call ui::macro(...) %}`. The `{{ ... }}` syntax will break compilation with "unresolved module" errors.
2.  **Keywords**: Never use Rust keywords like `type` as macro parameters.
3.  **Simplicity**: Prefer standard HTML for complex buttons/layouts. Use macros only for simple, repetitive UI bits like Badges and Inputs.

## 🚀 Instructions for the Next Agent
1.  **Read GEMINI.md first**: It contains the "Source of Truth" for how to write code in this repo.
2.  **Next Priority**: Finish the **Quick Create Role** refactor to achieve 100% "HTML-First" transition.
3.  **Validation Rule**: Always return `422` when a form fails validation.
4.  **No Chitchat**: Follow the senior engineer tone established in the project.
