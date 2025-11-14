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

- Once the server is accepting connections, apply the schema by running `backend/sql/init.sql`:

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
# Google OAuth – iOS (required for iOS builds)
GOOGLE_IOS_CLIENT_ID=<ios-google-client-id>
GOOGLE_IOS_REDIRECT_URI=com.ece1778.localguide:/oauthredirect
#GOOGLE_IOS_PROVIDER_NAME=google-ios
# Google OAuth – Android (required for Android builds)
GOOGLE_ANDROID_CLIENT_ID=<android-google-client-id>
GOOGLE_ANDROID_REDIRECT_URI=com.ece1778.localguide:/oauthredirect
#GOOGLE_ANDROID_PROVIDER_NAME=google-android
# Optional provider override
#GOOGLE_PROVIDER_NAME=google
#GOOGLE_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
#GOOGLE_TOKEN_URL=https://oauth2.googleapis.com/token
#GOOGLE_USERINFO_URL=https://www.googleapis.com/oauth2/v3/userinfo
```

The Expo client automatically hits `/auth/<provider>/callback`, where `<provider>` becomes `google-ios` or `google-android` (and `google` only if you also configure the shared web client), so be sure the backend has matching values for every platform you plan to support.

`BACKEND_URL` should point to the public base URL (what the frontend uses), while `BACKEND_BIND_ADDR` controls which host/port the Axum server listens on. Leave `BACKEND_BIND_ADDR` unset to keep the default `0.0.0.0:8080`. Environment variables exported directly in your shell still override `.env`, which can be handy for short-lived overrides or CI.

### 3. Start the backend

From the repository root run:

```sh
cargo run --bin local-guide-backend
```

The server listens on `0.0.0.0:8080` by default.
