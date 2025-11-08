#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SQL_FILE="$REPO_ROOT/backend/sql/init.sql"

CONTAINER_NAME="${PG_CONTAINER_NAME:-local-guide-postgres}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-local_guide}"
HOST_PORT="${POSTGRES_PORT:-5432}"
IMAGE="${POSTGRES_IMAGE:-postgres:15}"

if [[ ! -f "$SQL_FILE" ]]; then
  echo "❌ Initialization script not found at $SQL_FILE" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "❌ Docker is required but was not found in PATH." >&2
  exit 1
fi

running_container() {
  docker ps --filter "name=^${CONTAINER_NAME}$" --format '{{.Names}}'
}

remove_stopped_container() {
  if docker ps -a --filter "name=^${CONTAINER_NAME}$" --format '{{.Names}}' | grep -Fxq "$CONTAINER_NAME"; then
    docker rm -f "$CONTAINER_NAME" >/dev/null
  fi
}

start_container() {
  echo "🚀 Launching Postgres container $CONTAINER_NAME..."
  docker run --rm -d \
    --name "$CONTAINER_NAME" \
    -e POSTGRES_USER="$POSTGRES_USER" \
    -e POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
    -e POSTGRES_DB="$POSTGRES_DB" \
    -p "${HOST_PORT}:5432" \
    "$IMAGE" >/dev/null
}

ensure_running() {
  if [[ -n "$(running_container)" ]]; then
    echo "ℹ️  Container $CONTAINER_NAME already running; reusing it."
    return
  fi

  remove_stopped_container
  start_container
}

wait_for_postgres() {
  echo "⏳ Waiting for Postgres to accept connections..."
  until docker exec "$CONTAINER_NAME" pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" >/dev/null 2>&1; do
    sleep 1
  done
}

apply_sql() {
  echo "📄 Applying schema from $SQL_FILE..."
  docker exec -i "$CONTAINER_NAME" psql \
    -v ON_ERROR_STOP=1 \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    <"$SQL_FILE"
}

ensure_running
wait_for_postgres
apply_sql

cat <<EOF

✅ Postgres ready in container: $CONTAINER_NAME
    Image: $IMAGE
    Port : $HOST_PORT
    DB   : $POSTGRES_DB

Stop with: docker stop $CONTAINER_NAME
(container will be removed automatically because it was started with --rm)

EOF
