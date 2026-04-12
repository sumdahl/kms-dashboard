# KMS Dashboard - Project Handover & Progress Report

This document summarizes the current status of the "Strict MVC" refactor. It is intended for the next agent to understand the architectural rules and the work remaining.

## 🏗️ Architectural Pattern: "Smart Backend, Dumb Frontend"
We are following a strict MVC pattern using **Axum (Rust)**, **Askama (Templates)**, and **HTMX**.

1.  **Backend-Driven State**: All UI state (e.g., Is the sidebar pinned? Is this form valid?) is stored on the server (Cookies/Session) and rendered into HTML.
2.  **HTML Over JSON**: We prefer `application/x-www-form-urlencoded` over `application/json` for form submissions.
3.  **HTMX Swapping**: 
    *   Success: Use `HX-Redirect` header for full navigation.
    *   Failure: Return `422 Unprocessable Entity` with the form partial containing error messages.
4.  **Macros as Components**: We use `templates/macros/forms.html` to create reusable UI components with "props."
5.  **Global Errors**: `src/error.rs` contains a `smart_response` method that sends a partial HTML banner (`templates/partials/error_banner.html`) for HTMX requests.

---

## ✅ Completed Work

### 1. Core Infrastructure
*   [x] **ADR Documented**: `GEMINI.md` contains the rules for status codes, redirects, and Tailwind classes.
*   [x] **Global Error System**: `AppError` is HTMX-aware and supports OOB (Out-of-Band) error banners.
*   [x] **Form Macros**: Created `input_field` macro for consistent styling and error handling.

### 2. Authentication Module (Fully Refactored)
*   [x] **Login Flow**: Standard Form POST -> Backend Validation -> HTMX Swap or Redirect.
*   [x] **Signup Flow**: Standard Form POST -> Backend Validation -> HTMX Swap or Redirect.
*   [x] **Zero JS**: Removed all manual `fetch()` and banner-toggling scripts from auth templates.

### 3. Sidebar (HTMX-Aware)
*   [x] **Persistent State**: Moved pinned state to a `sidebar_pinned` cookie.
*   [x] **HTMX Toggle**: Sidebar "Pin" button now uses `hx-post` and updates via server-rendered HTML.
*   [x] **Dumb Script**: `static/js/sidebar.js` only handles visual hover peeking.

---

## ⏳ Work Remaining

### 1. Dashboard Module Refactor
*   [ ] **Create Role Wizard**: Currently uses old logic. Needs to move to backend-driven steps (Step 1 -> Session -> Step 2).
*   [ ] **Assign Role Form**: Needs to use `forms::input_field` macro and backend validation.
*   [ ] **Password Reset**: `forgot_password` and `reset_password` handlers still return `Json`. Need to return HTML/Redirects.

### 2. UI & CSS Polish
*   [ ] **Alignment Fixes**: User noted that some CSS placements/alignments need adjustment after the macro migration.
*   [ ] **Component Library**: Create macros for **Buttons**, **Tables**, and **Badges** to match the `input_field` style.

### 3. Cleanup
*   [ ] **JS Deletion**: Delete `static/js/json-enc.js` once all forms are refactored to standard Form POSTs.
*   [ ] **Route Cleanup**: Ensure all routes in `src/routes/mod.rs` point only to controllers.

---

## 🚀 Instructions for the Next Agent
1.  **Read GEMINI.md first**: It contains the "Source of Truth" for how to write code in this repo.
2.  **Next Priority**: Start with the **Assign Role** or **Password Reset** refactor to finish the "HTML-First" transition.
3.  **Validation Rule**: Always return `422` when a form fails validation. Use the macros in `templates/macros/forms.html` for all inputs.
4.  **No Chitchat**: Follow the senior engineer tone established in the project.
