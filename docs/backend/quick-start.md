## Running the backend locally

Follow the steps below to stand up the API with a local Postgres instance.

### 1. Prepare Postgres and schema

Choose one of the two supported flows:

**Option A – Start your own Postgres container**
- Launch Postgres yourself. Example:

```sh
docker run --rm -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=local_guide \
  postgres:15
```

- Once the server is accepting connections, apply the schema by running the `init_db.sql` file located at `backend/sql/init.sql`:

```sh
psql -U postgres -d local_guide -f backend/sql/init.sql
```

**Option B – Helper script**
- Run the helper:

```sh
./scripts/backend/start-postgres.sh
```

The script boots a disposable Docker container named `local-guide-postgres` and automatically feeds it the same `init_db.sql` schema.

### 2. Export required environment variables

The backend refuses to start unless these variables are set in the current shell:

```sh
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide
export JWT_SECRET=<random-jwt-secret>
# optional overrides
export JWT_TTL_SECONDS=3600
export GOOGLE_PROVIDER_NAME=google

# Google OAuth configuration (all required)
export GOOGLE_CLIENT_ID=<google-clould-console-id>
export GOOGLE_CLIENT_SECRET=<google-count-console-secret>
export GOOGLE_REDIRECT_URI=https://your-website.com/auth/google/callback
# Optional overrides if using a non-default Google workspace / proxy
export GOOGLE_AUTH_URL=...
export GOOGLE_TOKEN_URL=...
export GOOGLE_USERINFO_URL=...
```

### 3. Start the backend

From the repository root run:

```sh
cargo run --bin local-guide-backend
```

The server listens on `0.0.0.0:8080` by default.
