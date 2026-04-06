include .env
export

DB_CONTAINER=kms-db
DB_USER=kms_user
DB_NAME=kms_db

# ── helpers ────────────────────────────────────────────────────────────────────
PSQL=docker exec -it $(DB_CONTAINER) psql -U $(DB_USER) -d $(DB_NAME)

# ── targets ────────────────────────────────────────────────────────────────────

## Wipe and recreate schema, then re-run all migrations on next cargo run
db/reset:
	@echo "⚠️  Resetting local database..."
	@$(PSQL) -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	@echo "✅ Schema wiped. Run 'cargo run' to re-apply all migrations."

## Show applied migrations
db/status:
	@$(PSQL) -c "SELECT version, installed_on, success FROM _sqlx_migrations ORDER BY version;"

## Fix a single migration mismatch without full reset (usage: make db/fix version=20260403085421)
db/fix:
	@test -n "$(version)" || (echo "Usage: make db/fix version=<version>" && exit 1)
	@echo "Removing migration record $(version)..."
	@$(PSQL) -c "DELETE FROM _sqlx_migrations WHERE version = $(version);"
	@echo "✅ Done. Now manually DROP the affected tables, then cargo run."

## Add a new migration (usage: make db/new name=add_sessions_table)
db/new:
	@test -n "$(name)" || (echo "Usage: make db/new name=your_migration_name" && exit 1)
	sqlx migrate add $(name)
	@echo "✅ Migration file created. Edit it, then cargo run."

## Prepare sqlx offline cache before committing
db/prepare:
	cargo sqlx prepare

.PHONY: db/reset db/status db/fix db/new db/prepare
