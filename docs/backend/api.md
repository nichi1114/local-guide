## Backend API Reference

Base URL (default): `http://localhost:8080`

All responses are JSON. Errors follow `{ "error": "<code>", "message": "<details>" }`.

---

### POST `/auth/{provider}/callback`

Completes an OAuth PKCE flow for the given provider (currently only `google`). Exchange the authorization code for profile data, create/update the user, and receive a session token.

**Request headers**
- `Content-Type: application/json`

**Request body**
```json
{
  "code": "string",          // authorization_code from the provider redirect
  "code_verifier": "string"  // PKCE verifier that matches the code_challenge sent earlier
}
```

**Successful response**
```json
{
  "user": {
    "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "email": "user@example.com",
    "name": "Test User",
    "avatar_url": "https://example.com/avatar.png"
  },
  "jwt_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Failure modes**
- `404 provider_not_configured` – provider key is unknown or missing in env config.
- `502 token_exchange_failed|userinfo_failed` – provider endpoints returned an error.
- `500 storage_error|jwt_error` – database or token generation failure.

---

### GET `/usr`

Returns the profile for the currently authenticated user. Requires a valid JWT issued by the backend.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)

**Successful response**
```json
{
  "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "email": "user@example.com",
  "name": "Test User",
  "avatar_url": "https://example.com/avatar.png"
}
```

**Failure modes**
- `401` – missing/invalid/expired token.
- `404 user_not_found` – token valid but corresponding database user no longer exists.
- `500 internal_error` – unexpected storage error.
