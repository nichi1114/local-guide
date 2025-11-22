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

---

### POST `/places`

Create a new place and upload all associated images in a single multipart request. The client must generate UUIDs for the place and each image; files are stored on disk and referenced in Postgres atomically so no dangling references remain.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)
- `Content-Type: multipart/form-data`

**Multipart fields**
- `id` (text, required) – UUID for the place.
- `name`, `category`, `location` (text, required)
- `note` (text, optional)
- `image_id` (text, required per image) – UUID string for the *next* `image` part.
- `image` (file, required) – binary image data; must follow an `image_id`.

**Successful response**
```json
{
  "id": "e3f82841-e0b6-4dda-8f3b-ea0f4ebda123",
  "user_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "name": "Blue Bottle Cafe",
  "category": "Coffee",
  "location": "300 Webster St, Oakland, CA",
  "note": "Try the oat latte",
  "images": [
    {
      "id": "a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
      "caption": null,
      "download_url": "/places/e3f82841-e0b6-4dda-8f3b-ea0f4ebda123/images/a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
      "created_at": "2024-08-22T18:25:43.511308Z"
    }
  ],
  "created_at": "2024-08-22T18:25:43.511308Z",
  "updated_at": "2024-08-22T18:25:43.511308Z"
}
```

**Failure modes**
- `400 invalid_request` – missing fields, malformed UUIDs, or unmatched `image_id`/`image` pairs.
- `401` – missing or invalid JWT.
- `500 image_io_error|internal_error` – failed to persist image file or DB transaction.

---

### GET `/places`

List all places owned by the authenticated user (most recent first).

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)

**Successful response**
```json
[
  {
    "id": "e3f82841-e0b6-4dda-8f3b-ea0f4ebda123",
    "name": "Blue Bottle Cafe",
    "category": "Coffee",
    "location": "300 Webster St, Oakland, CA",
    "note": "Try the oat latte",
    "images": [
      {
        "id": "a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
        "caption": null,
        "download_url": "/places/e3f82841-e0b6-4dda-8f3b-ea0f4ebda123/images/a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
        "created_at": "2024-08-22T18:25:43.511308Z"
      }
    ],
    "created_at": "2024-08-22T18:25:43.511308Z",
    "updated_at": "2024-08-22T18:25:43.511308Z"
  }
]
```

**Failure modes**
- `401` – missing/invalid JWT.
- `500 internal_error` – database failure.

---

### GET `/places/{id}`

Fetch a single place (and its images) for the authenticated user.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)

**Successful response**
- Same shape as `POST /places`.

**Failure modes**
- `401` – missing/invalid JWT.
- `404 not_found` – place does not belong to the user.
- `500 internal_error` – database error.

---

### PATCH `/places/{id}`

Update selected fields for a place and atomically add/remove images. All updates occur within a transaction; image files are deleted or cleaned up on failure.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)
- `Content-Type: multipart/form-data`

**Multipart fields**
- Any subset of `name`, `category`, `location`, `note` (text).
- `image_id` + `image` pairs for new images (same semantics as creation).
- `delete_image_ids` (text) – JSON array of UUID strings to remove (e.g., `["id1","id2"]`).

**Successful response**
- Same shape as `GET /places/{id}` with updated metadata and image set.

**Failure modes**
- `400 invalid_request` – malformed fields, mismatched `image_id` counts, or invalid JSON for deletions.
- `401` – missing/invalid JWT.
- `404 not_found` – place not owned by user.
- `500 image_io_error|internal_error` – failed to write/delete image files or DB issues.

---

### GET `/places/{id}/images`

List metadata for images attached to the place.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)

**Successful response**
```json
[
  {
    "id": "a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
    "caption": null,
    "download_url": "/places/e3f82841-e0b6-4dda-8f3b-ea0f4ebda123/images/a00e55ad-17c5-4a40-90c0-034b89cdb1c4",
    "created_at": "2024-08-22T18:25:43.511308Z"
  }
]
```

**Failure modes**
- `401` – missing/invalid JWT.
- `404 not_found` – place not owned by user.
- `500 internal_error` – database error.

---

### GET `/places/{place_id}/images/{image_id}`

Download a stored image file for the given place. Content-Type is inferred from the stored filename extension; data streams as binary.

**Request headers**
- `Authorization: Bearer <jwt_token>` (required)

**Successful response**
- Binary image data with `Content-Type` set (e.g., `image/jpeg`).

**Failure modes**
- `401` – missing/invalid JWT.
- `404 not_found` – place or image not owned by user, or image missing on disk.
- `500 image_io_error` – file read failure.
