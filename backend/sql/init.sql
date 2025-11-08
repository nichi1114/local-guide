-- Table: users
-- Stores user account information for each registered user.
-- Each user has a unique UUID, email, display name, and optional avatar URL.
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,                        -- Unique identifier for the user
    email TEXT,                                 -- User's email address (may be null if not provided)
    name TEXT,                                  -- User's display name
    avatar_url TEXT,                            -- URL to the user's avatar image
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- Timestamp when the user was created
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- Timestamp when the user was last updated
);

-- Table: oauth_identities
-- Stores OAuth identity information for users authenticated via third-party providers.
-- Each record links a user account to an external OAuth provider (e.g., Google, GitHub).
-- The user_id field is a foreign key referencing users(id), establishing a one-to-many relationship:
--   - Each user can have multiple OAuth identities (for different providers).
--   - Each OAuth identity belongs to exactly one user.
CREATE TABLE IF NOT EXISTS oauth_identities (
    id UUID PRIMARY KEY,                        -- Unique identifier for the OAuth identity record
    provider TEXT NOT NULL,                     -- Name of the OAuth provider (e.g., 'google', 'github')
    provider_user_id TEXT NOT NULL,             -- Unique user ID assigned by the OAuth provider
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE, -- References the associated user
    linked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- Timestamp when the identity was linked
    UNIQUE (provider, provider_user_id)         -- Ensures a provider's user ID is only linked once
);

-- Index to optimize lookups of OAuth identities by user_id
CREATE INDEX IF NOT EXISTS oauth_identities_user_idx ON oauth_identities (user_id);
