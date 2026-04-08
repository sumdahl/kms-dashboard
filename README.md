# KMS Dashboard

A web-based key management and access control dashboard built with Rust and HTMX. Provides user authentication, role-based access control (RBAC), admin panel, and inventory management through a fast, server-rendered interface.

## Table of Contents

- [Features](#features)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Database Management](#database-management)
- [Project Structure](#project-structure)
- [Troubleshooting](#troubleshooting)
- [License](#license)

---

## Features

- **Authentication** — Login, signup, password reset via email (Resend API), JWT session tokens with Argon2 password hashing
- **Role-Based Access Control** — Granular permission system with resources (`orders`, `customers`, `reports`, `inventory`, `admin_panel`) and access levels (`read`, `write`, `admin`)
- **Role Management** — Create roles with custom permissions, assign roles to users with optional expiry dates, view role details and assignment history
- **Admin Panel** — List and disable user accounts, manage role assignments across the organization
- **Dashboard** — Home page with inventory status, personal roles view, and global search
- **Dynamic UI** — HTMX-powered interactions without full page reloads, Lucide icons, collapsible sidebar
- **Live Reload** — Automatic browser refresh during development via `tower-livereload`

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, [Axum](https://github.com/tokio-rs/axum) 0.7, Tokio |
| Database | PostgreSQL 16, [SQLx](https://github.com/launchbadge/sqlx) 0.8 |
| Templating | [Askama](https://github.com/djc/askama) 0.12 (Jinja2-like) |
| Frontend | [HTMX](https://htmx.org/), Tailwind CSS v4 |
| Auth | JWT (`jsonwebtoken`), Argon2 password hashing |
| Email | [Resend](https://resend.com/) via `resend-rs` |
| Dev tooling | `cargo-watch`, `tower-livereload`, PostCSS |

---

## Prerequisites

- **Rust** (stable) — [rustup.rs](https://rustup.rs/)
- **Node.js** 18+ and npm
- **Docker** and Docker Compose (for the database)
- **cargo-watch** — `cargo install cargo-watch`

---

## Quick Start

```bash
# 1. Clone and enter the repo
git clone <repo-url> && cd kms-dashboard

# 2. Start the database
docker compose up -d

# 3. Copy the example env and fill in secrets
cp .env.example .env   # then edit .env

# 4. Install Node dependencies and build CSS
npm install && npm run copy-htmx && npm run build:css

# 5. Run migrations
cargo sqlx migrate run

# 6. Start the dev server
npm run dev
```

The server starts at `http://localhost:3000`.

---

## Installation

### 1. Start the database

```bash
docker compose up -d
```

This creates a PostgreSQL 16 container named `kms-db` on port `5432` with:
- User: `kms_user`
- Password: `admin@123`
- Database: `kms_db`

### 2. Configure environment

```bash
cp .env.example .env
```

Edit `.env` — see [Configuration](#configuration) for required variables.

### 3. Install frontend dependencies

```bash
npm install
npm run copy-htmx   # copies HTMX to static/js/
npm run build:css   # compiles Tailwind CSS
```

### 4. Run database migrations

```bash
cargo sqlx migrate run
```

### 5. Build and run

```bash
# Development (with auto-reload)
npm run dev

# Production build
cargo build --release
./target/release/kms-dashboard
```

---

## Configuration

Environment variables (set in `.env` or the shell):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | Yes | — | Secret key for signing JWT tokens |
| `RESEND_API_KEY` | Yes | — | API key for sending password-reset emails |
| `PORT` | No | `3000` | HTTP port the server listens on |

Example `.env`:

```dotenv
DATABASE_URL=postgres://kms_user:admin@123@localhost:5432/kms_db
JWT_SECRET=change-me-to-a-long-random-string
RESEND_API_KEY=re_xxxxxxxxxxxxxxxxxxxx
PORT=3000
```

---

## Usage

### Pages

| Path | Description | Auth required |
|------|-------------|---------------|
| `/` | Dashboard home | Yes |
| `/login` | Sign in | No |
| `/signup` | Create account | No |
| `/forgot-password` | Request password reset | No |
| `/reset-password` | Set new password (via email link) | No |
| `/roles` | Browse all roles | Yes |
| `/roles/new` | Role creation wizard | Yes |
| `/roles/:name` | Role detail and assignments | Yes |
| `/users` | User list | Yes |
| `/assign` | Assign a role to a user | Yes |
| `/admin/*` | Admin management pages | Yes (admin) |

### Role Permissions

Roles are composed of one or more permission entries. Each entry combines a **resource** and an **access level**:

| Resource | Description |
|----------|-------------|
| `orders` | Order management |
| `customers` | Customer records |
| `reports` | Reporting and analytics |
| `inventory` | Inventory management |
| `admin_panel` | Administrative panel access |

| Access Level | Description |
|--------------|-------------|
| `read` | View only |
| `write` | Create and modify |
| `admin` | Full control including deletion |

---

## API Reference

All API endpoints are under `/api/` and require a valid JWT cookie.

| Method | Path | Permission | Description |
|--------|------|------------|-------------|
| `GET` | `/api/inventory` | `inventory` read+ | Inventory status summary |
| `GET` | `/api/me/roles` | Authenticated | Current user's active roles |
| `GET` | `/api/search?q=<query>` | Authenticated | Global search |

### Auth endpoints (under `/auth/`)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/auth/login` | Authenticate and receive JWT cookie |
| `POST` | `/auth/signup` | Register a new account |
| `POST` | `/auth/logout` | Clear the session cookie |
| `POST` | `/auth/forgot-password` | Send password-reset email |
| `POST` | `/auth/reset-password` | Reset password with token from email |

---

## Database Management

Database commands are managed via `make` and assume the `kms-db` Docker container is running.

| Command | Description |
|---------|-------------|
| `make db/reset` | Wipe and recreate the database schema |
| `make db/status` | Show applied migration status |
| `make db/new name=<migration_name>` | Create a new SQLx migration file |
| `make db/fix version=<timestamp>` | Mark a failed migration as fixed |
| `make db/prepare` | Regenerate the offline SQLx query cache |

---

## Project Structure

```
kms-dashboard/
├── src/
│   ├── main.rs              # Entrypoint
│   ├── startup.rs           # App initialization and router setup
│   ├── config.rs            # Environment variable loading
│   ├── app_state.rs         # Shared application state
│   ├── db/                  # Database pool and migrations
│   ├── models/              # Domain types (User, Role, Claims, etc.)
│   │   └── types.rs         # Resource and AccessLevel enums
│   ├── handlers/            # Request handlers
│   │   ├── auth.rs          # Login, signup, logout
│   │   ├── password_reset.rs
│   │   ├── dashboard.rs     # Home, inventory, roles API
│   │   ├── admin.rs         # Admin user/role management
│   │   └── api.rs           # Global search
│   ├── routes/              # Router definitions
│   │   ├── mod.rs           # Main page routes
│   │   ├── auth.rs          # /auth/* routes
│   │   ├── admin.rs         # /admin/* routes
│   │   └── api.rs           # /api/* routes
│   ├── middleware/
│   │   └── auth.rs          # JWT extraction, admin guard
│   └── styles/
│       └── input.css        # Tailwind CSS source
├── templates/
│   ├── layout.html          # Base layout with sidebar
│   ├── login.html
│   ├── signup.html
│   ├── forgot_password.html
│   ├── reset_password.html
│   ├── error.html
│   ├── dashboard/           # Authenticated page templates
│   │   ├── home.html
│   │   ├── users.html
│   │   ├── roles.html
│   │   ├── role_detail.html
│   │   ├── create_role_wizard.html
│   │   ├── assign.html
│   │   └── onboarding.html
│   ├── partials/            # HTMX partial templates
│   └── email/               # Email HTML templates
├── static/
│   ├── css/output.css       # Compiled Tailwind (generated)
│   └── js/                  # HTMX, Lucide, and other JS
├── migrations/              # SQLx migration files
├── docker-compose.yml
├── Cargo.toml
├── package.json
└── Makefile
```

---

## Troubleshooting

**`DATABASE_URL must be set` on startup**
Make sure your `.env` file exists and contains `DATABASE_URL`. Run `docker compose up -d` first.

**CSS not updating in browser**
Run `npm run build:css` (or use `npm run watch:css` in a separate terminal). The compiled file is `static/css/output.css`.

**`cargo sqlx` offline errors**
Run `make db/prepare` to regenerate the SQLx query cache after schema changes.

**Port already in use**
Set a different port with `PORT=3001` in your `.env`.

---

## License

MIT
