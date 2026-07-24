-- OTP challenges and session storage for the optional, strictly opt-in
-- public account system (migrations/0007_accounts_and_drafts.sql). That
-- migration deliberately left OTP handling as "handled at the application
-- layer" rather than a permanent table — this migration is that
-- follow-up, adding two short-lived, narrowly-scoped tables rather than
-- persisting OTP codes in `user_accounts` itself.
--
-- Kept Postgres-backed (not an in-process cache) because `web-app` is
-- expected to run as more than one instance behind a load balancer in
-- production (docs/SECURITY-AND-SRE-OPERATIONS.md) — an in-memory OTP/
-- session store would fail unpredictably depending on which instance
-- served a given request.

CREATE TABLE account_otp_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Same hashing discipline as user_accounts.contact_identifier_hash:
    -- never store the plaintext email/phone here either.
    contact_identifier_hash TEXT NOT NULL,
    contact_channel TEXT NOT NULL CHECK (contact_channel IN ('email', 'phone')),
    -- The OTP itself is also hashed at rest (argon2, same as admin
    -- passwords) — a database read alone must never be sufficient to
    -- impersonate a pending login.
    otp_hash TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    consumed_at TIMESTAMPTZ
);

CREATE INDEX idx_account_otp_challenges_contact ON account_otp_challenges (contact_identifier_hash, contact_channel);
CREATE INDEX idx_account_otp_challenges_expires_at ON account_otp_challenges (expires_at);

COMMENT ON TABLE account_otp_challenges IS
    'Short-lived (10-minute expiry, enforced at the application layer and '
    'swept by a periodic purge job). attempt_count caps guesses at 5 '
    'before the challenge is invalidated outright, independent of expiry.';

CREATE TABLE account_sessions (
    id TEXT PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES user_accounts (id) ON DELETE CASCADE,
    expiry_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_account_sessions_expiry_date ON account_sessions (expiry_date);
CREATE INDEX idx_account_sessions_account_id ON account_sessions (account_id);

COMMENT ON TABLE account_sessions IS
    'Deliberately separate from admin_users'' `sessions` table '
    '(migrations/0006) — a citizen account and an admin/reviewer account '
    'are different trust domains with different tables and, in '
    'production, different least-privilege Postgres roles '
    '(docs/SECURITY-AND-SRE-OPERATIONS.md Section 3), so a compromised '
    'public-account credential can never be a path to admin access.';
