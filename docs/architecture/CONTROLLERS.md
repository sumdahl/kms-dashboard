# The Controller Layer (Routing & Logic)

The **Controller** layer handles incoming requests, interacts with the **Model**, and selects the appropriate **View** for the response.

## 🌉 Core Components

1.  **Axum Routes:**
    *   Defined in `src/routes/`.
    *   Routes define the API structure and determine which handler to call for each URL.
    *   Routes also act as the "View-Model" layer, gathering all the data required to populate an `Askama` template struct.

2.  **Axum Handlers:**
    *   Defined in `src/handlers/`.
    *   Handlers are where the "real work" happens (database interaction, permission checks).
    *   Handlers extract data from the request (URL params, JSON body, query strings) and return responses (JSON or HTML).

3.  **Middleware:**
    *   Defined in `src/middleware/`.
    *   Authentication and Role-Based Access Control (RBAC) are implemented as middleware.
    *   This ensures security is enforced globally and not duplicated across every handler.

## ⚙️ Response Strategy

Handlers can return multiple types of responses:
*   `Html<String>`: Rendered HTML for full or partial page requests.
*   `Json<T>`: Standard JSON for API clients.
*   `Response`: A complex object with custom headers (like `HX-Trigger` or `HX-Replace-Url` for HTMX).
*   `AppResult<T>`: A custom error-handling wrapper to ensure clean and consistent error responses.

## 📦 Route Composition
The router is composed of nested modules for better organization:
*   `dashboard::router()`: Main application interface.
*   `admin::router()`: Privileged operations with mandatory RBAC middleware.
*   `api::router()`: Publicly accessible JSON endpoints.
*   `auth::router()`: Login, signup, and session management.
