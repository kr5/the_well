-- Multi-channel bot configuration and status.
-- docs/PRD-V2-RUST-PLATFORM.md Section 11 admin page 9.

CREATE TABLE bot_channel_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel client_platform NOT NULL UNIQUE CHECK (channel IN ('telegram', 'whatsapp', 'ivr')),
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    -- Secrets (bot tokens, API keys) are NEVER stored in this table or any
    -- database row — they live in the SOPS+age-encrypted secret files
    -- described in docs/SECURITY-AND-SRE-OPERATIONS.md Section 2 and are
    -- injected as environment variables at deploy time. This table only
    -- tracks non-secret operational config and status.
    display_name TEXT NOT NULL,
    webhook_configured BOOLEAN NOT NULL DEFAULT false,
    last_health_check_at TIMESTAMPTZ,
    last_health_check_ok BOOLEAN,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_bot_channel_config_updated_at
    BEFORE UPDATE ON bot_channel_config
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO bot_channel_config (channel, display_name, is_enabled) VALUES
    ('telegram', 'Telegram', false),
    ('whatsapp', 'WhatsApp (Meta Cloud API)', false),
    ('ivr', 'Exotel IVR (scaffolded, not yet implemented)', false);

-- WhatsApp template messages require Meta pre-approval for outbound
-- messages sent outside a user-initiated 24-hour window — this table
-- tracks that review workflow (PRD v2 Section 11 admin page 9).
CREATE TABLE whatsapp_message_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_name TEXT NOT NULL UNIQUE,
    template_body TEXT NOT NULL,
    locale TEXT NOT NULL,
    meta_approval_status TEXT NOT NULL DEFAULT 'pending' CHECK (meta_approval_status IN ('pending', 'approved', 'rejected')),
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_whatsapp_message_templates_status ON whatsapp_message_templates (meta_approval_status);
