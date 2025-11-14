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

### 2. Configure environment variables

Both the backend (Rust) service and the Expo frontend read configuration from a `.env` file at the repository root. Create that file manually (or via your preferred secrets manager) using the template below, then replace the placeholder values:

```env
BACKEND_URL=http://localhost:8080
#BACKEND_BIND_ADDR=0.0.0.0:8080
DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide
#TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide_test
JWT_SECRET=replace-with-a-random-secret
JWT_TTL_SECONDS=3600
GOOGLE_CLIENT_ID=<google-cloud-console-id>
GOOGLE_CLIENT_SECRET=<google-cloud-console-secret>
GOOGLE_REDIRECT_URI=https://your-website.com/auth/google/callback
GOOGLE_PROVIDER_NAME=google
#GOOGLE_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
#GOOGLE_TOKEN_URL=https://oauth2.googleapis.com/token
#GOOGLE_USERINFO_URL=https://www.googleapis.com/oauth2/v3/userinfo
```

`BACKEND_URL` should point to the public base URL (what the frontend uses), while `BACKEND_BIND_ADDR` controls which host/port the Axum server listens on. Leave `BACKEND_BIND_ADDR` unset to keep the default `0.0.0.0:8080`. Environment variables exported directly in your shell still override `.env`, which can be handy for short-lived overrides or CI.

### 3. Start the backend

From the repository root run:

```sh
cargo run --bin local-guide-backend
```

The server listens on `0.0.0.0:8080` by default.
