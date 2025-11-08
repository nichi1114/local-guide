#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
START_SCRIPT="$SCRIPT_DIR/start-postgres.sh"

POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-local_guide_test}"
POSTGRES_PORT="${POSTGRES_PORT:-5433}"
PG_CONTAINER_NAME="${PG_TEST_CONTAINER_NAME:-local-guide-postgres-test}"

cleanup() {
  if docker ps --filter "name=^${PG_CONTAINER_NAME}$" --format '{{.Names}}' | grep -Fxq "$PG_CONTAINER_NAME"; then
    docker stop "$PG_CONTAINER_NAME" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT INT TERM

PG_CONTAINER_NAME="$PG_CONTAINER_NAME" \
POSTGRES_USER="$POSTGRES_USER" \
POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
POSTGRES_DB="$POSTGRES_DB" \
POSTGRES_PORT="$POSTGRES_PORT" \
"$START_SCRIPT" >/dev/null

DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}"
export DATABASE_URL
export TEST_DATABASE_URL="$DATABASE_URL"

echo "Running cargo test with DATABASE_URL=$DATABASE_URL"
cargo test -- --test-threads=1 "$@"
