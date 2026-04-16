# Request Lifecycle & HTMX Integration

The project uses a **Dual-Template Strategy** to provide a fast, single-page-application (SPA) experience using only standard HTML and Rust.

## 🔄 Request Flow

1.  **User Action:** A user clicks a link with `hx-get="/roles"`.
2.  **Request:** HTMX sends a standard HTTP request to the server, but adds an `hx-request: true` header.
3.  **Controller Logic:** The route handler (`src/routes/dashboard.rs`) receives the request.
4.  **Data Fetching:** The handler fetches all required data for the roles page.
5.  **View Selection:** The handler checks the headers using the `is_htmx_partial` function:
    ```rust
    fn is_htmx_partial(headers: &HeaderMap) -> bool {
        headers.get("hx-request").is_some() && !headers.contains_key("hx-history-restore-request")
    }
    ```
6.  **Rendering:**
    *   **If `true`:** The handler renders `RolesPartialTemplate` (a small HTML fragment).
    *   **If `false`:** The handler renders `RolesTemplate` (a full HTML page with layout).
7.  **HTMX Swap:**
    *   The fragment is sent back to the browser.
    *   HTMX identifies the target element (usually `#main-content`) and replaces its content with the new fragment.
    *   The URL in the browser is updated automatically.

## ⚡ Partial Rendering vs. Full Rendering

| Feature | Partial Rendering (HTMX) | Full Rendering (Direct Load) |
| :--- | :--- | :--- |
| **Request Header** | `hx-request: true` | (None) |
| **Response Body** | HTML fragment (no `<head>` or sidebar) | Complete HTML document |
| **Network Size** | Small (2-10KB) | Medium (10-50KB) |
| **Browser Behavior** | Fast DOM swap (no screen flicker) | Full page reload |
| **Browser History** | Managed by HTMX | Standard navigation |

## 🚀 Advanced HTMX Patterns

*   **`HX-Trigger`:** The server can include a header to trigger a specific JS event on the client after the swap.
*   **`HX-Replace-Url`:** The server can update the browser's URL to a "clean" version, even if the user arrived via a complex query.
*   **OOB Swaps:** Using `hx-swap-oob="true"`, we can update multiple parts of the page simultaneously (e.g., updating the content area AND showing a success message in the header).
