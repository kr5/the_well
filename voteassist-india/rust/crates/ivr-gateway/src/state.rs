use std::sync::Arc;

use channel_core::SessionStore;
use core_domain::DecisionTree;

#[derive(Clone)]
pub struct AppState {
    pub tree: Arc<DecisionTree>,
    /// Keyed by Exotel's `CallSid` — the only call-scoped identifier this
    /// scaffold has to route a follow-up Passthru request back to the
    /// right in-progress `EngineState`. As with the WhatsApp `wa_id`
    /// index, this is purely internal routing; only `channel_core`'s
    /// randomly generated session data (never `CallSid` itself) is what
    /// would ever reach `analytics_events.session_id` if this channel is
    /// wired into analytics later.
    pub sessions: SessionStore,
    /// Present only if `WHATSAPP_*` env vars are configured for this
    /// deployment — enables E8.F3.T4's "spoken summary + SMS/WhatsApp
    /// follow-up with the deep link" terminal-outcome handoff (a phone
    /// call can't "click" a link). `None` means the handoff is skipped
    /// and logged, not silently pretended.
    pub whatsapp_handoff: Option<Arc<bot_whatsapp::client::WhatsAppClient>>,
    /// Optional shared secret embedded in the webhook URL configured in
    /// Exotel's dashboard (`?secret=...`). Exotel's Passthru applet has no
    /// built-in request-signing scheme analogous to Meta's
    /// `X-Hub-Signature-256`, so a URL-embedded secret is this scaffold's
    /// own, disclosed substitute — `None` means the endpoint is
    /// unauthenticated (logged loudly at boot; see `main.rs`).
    pub webhook_shared_secret: Option<Arc<str>>,
}
