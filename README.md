# KMS Dashboard

A web dashboard built with Rust (Axum) and Tailwind CSS.

## Tech Stack

- **Backend**: Rust with [Axum](https://github.com/tokio-rs/axum) web framework
- **Database**: PostgreSQL with SQLx
- **Templating**: [Askama](https://github.com/djc/askama) (Jinja2-like templates)
- **Frontend**: Tailwind CSS v4, HTMX
- **Live Reload**: tower-livereload
- **Logging**: tracing, tracing-subscriber

## Getting Started

```bash
# Install Rust dependencies
cargo build

# Install Node dependencies
npm install

# Copy HTMX to static files
npm run copy-htmx

# Build CSS
npm run build:css

# Run development server (watches Rust, templates, and CSS)
npm run dev
```

The server runs at `http://localhost:3000`.

## Project Structure

```
├── src/
│   ├── main.rs          # Application entrypoint
│   ├── startup.rs       # Application initialization
│   ├── db/              # Database and migrations
│   └── ...              # Routes, models, auth, etc.
├── templates/
│   ├── layout.html      # Base layout
│   ├── dashboard/
│   │   └── home.html    # Dashboard page
│   └── partials/        # Reusable components
├── static/
│   ├── css/output.css   # Compiled Tailwind
│   └── js/              # JavaScript files
├── src/styles/input.css # Tailwind source
└── Makefile             # Database management targets
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm run copy-htmx` | Copy HTMX to static |
| `npm run build:css` | Build Tailwind CSS |
| `npm run watch:css` | Watch and rebuild CSS |
| `npm run dev` | Run with auto-reload |

## Database Commands

| Command | Description |
|---------|-------------|
| `make db/reset` | Wipe and recreate database schema |
| `make db/status` | Show applied migrations status |
| `make db/fix version=20260403085421` | Fix failed migration |
| `make db/new name=add_table` | Create new SQLx migration |
| `make db/prepare` | Prepare offline SQLx cache |

## License

MIT
