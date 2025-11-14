# local-guide

## Getting started

1. Create a `.env` file in the repository root using the template below, then fill in your real values. The same `.env` powers both the Rust backend (`cargo run --bin local-guide-backend`) and the Expo app (`npm run start`), so you only need to set things like `BACKEND_URL`, `DATABASE_URL`, and the Google OAuth credentials once.

   ```env
   BACKEND_URL=http://localhost:8080
   #BACKEND_BIND_ADDR=0.0.0.0:8080
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide
   #TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide_test
   JWT_SECRET=replace-with-a-random-secret
   JWT_TTL_SECONDS=3600
   GOOGLE_CLIENT_ID=<your-google-client-id>
   GOOGLE_CLIENT_SECRET=<your-google-client-secret>
   GOOGLE_REDIRECT_URI=http://localhost:8080/auth/google/callback
   GOOGLE_PROVIDER_NAME=google
   #GOOGLE_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
   #GOOGLE_TOKEN_URL=https://oauth2.googleapis.com/token
   #GOOGLE_USERINFO_URL=https://www.googleapis.com/oauth2/v3/userinfo
   ```
2. Start the backend:

   ```sh
   cargo run --bin local-guide-backend
   ```

3. In a separate terminal start Expo:

   ```sh
   npm run start
   ```

Adjust the commands if you prefer `yarn`/`pnpm` or release builds. Refer to `docs/backend/quick-start.md` for additional backend-specific details.
