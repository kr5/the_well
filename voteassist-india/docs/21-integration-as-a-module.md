# Integration as a Module of Another System

VoteAssist India is built as a set of independently-deployable services
that happen to ship together in one repository — not as a monolith that
assumes it owns an entire domain, server, or Postgres instance. This
document is the concrete guide for the expected real-world deployment
shape: **VoteAssist India embedded as one module inside a larger parent
system** (a broader civic-tech platform, a party-neutral public-information
portal, an NGO's citizen-services suite, etc.), rather than run standalone
on its own domain.

Nothing described here requires code changes beyond what's already in this
repository (the `crates/api` CORS layer described in Section 6 below was
added specifically to support this scenario). Everything else is
configuration and deployment topology.

## 1. The seven services, and how they actually talk to each other

| Service | Crate | Talks to Postgres? | Talks to any other VoteAssist service over the network? |
|---|---|---|---|
| Public web app | `crates/web-app` | yes | no |
| Admin dashboard | `crates/admin-app` | yes | no |
| Stateless decision-engine API | `crates/api` | no (pure library logic, no DB) | no |
| Scheduled jobs (link-checker, analytics rollup, digest, etc.) | `crates/jobs` | yes | no |
| Telegram bot | `crates/bot-telegram` | yes | no |
| WhatsApp bot | `crates/bot-whatsapp` | yes | no |
| IVR gateway | `crates/ivr-gateway` | yes | **in-process only** — calls `bot_whatsapp::WhatsAppClient` as a Rust library for outbound WhatsApp fallback messages, never over HTTP |

The only shared dependency across all seven is Postgres. There is no
service mesh, no internal HTTP calls between VoteAssist services, and no
message queue. This matters for embedding: a parent system does not need
to stand up an internal network or service-discovery layer to run
VoteAssist alongside it — each binary is a standalone process that needs a
`DATABASE_URL` and, for the bot/IVR services, provider credentials.

A parent system can therefore adopt VoteAssist at any of three depths:

1. **Whole-service embedding** — run some or all of the seven binaries
   as-is, alongside the parent system's own services, sharing
   infrastructure (Postgres, reverse proxy, monitoring) per Sections 2-5.
2. **Selective feature embedding** — run only `crates/web-app` (the
   citizen-facing decision tool) and `crates/api`, skip the bot channels
   and admin dashboard if the parent system has its own moderation
   tooling.
3. **Library-level embedding** — the parent system is itself a Rust Axum
   application and mounts VoteAssist's router or calls its logic
   in-process, with no separate VoteAssist deployment at all. See Section
   6.

## 2. Path-prefix mounting (reverse proxy)

`crates/web-app` and `crates/admin-app` are Leptos SSR applications. Like
most SSR frameworks, they assume they own the root path space (`/`) —
there is no verified, tested "base path" configuration option for Leptos
0.8 in this codebase, and adding one without being able to compile and
manually exercise routing would be exactly the kind of unverified change
this project avoids (see `docs/16-security-threat-model.md`'s general
stance on unverified assumptions).

The practical, zero-code-change approach is **reverse-proxy path
stripping**: mount VoteAssist at a sub-path in the parent system's public
URL space, and have the reverse proxy rewrite/strip that prefix before
forwarding to VoteAssist's origin. For example, with nginx:

```nginx
location /vote-assist/ {
    proxy_pass http://127.0.0.1:8080/;   # trailing slash strips the /vote-assist/ prefix
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

Two things to check before relying on this:

- **Internal links.** VoteAssist's own internal `<a>`/router links are
  root-relative (e.g. `/kb/postal-ballot`), not prefixed. Path-stripping at
  the proxy means the browser always sees the `/vote-assist/...` URL and
  VoteAssist's server only ever sees the stripped, unprefixed path — so
  internal links resolve correctly through the proxy without VoteAssist
  needing to know its own mount point. This is the same reason the
  trailing-slash form of `proxy_pass` is required above, not the
  no-trailing-slash form (which would keep the prefix and break every
  internal route).
- **Absolute asset URLs.** If a future change introduces any
  absolute-path reference to a static asset or API base URL, it needs to
  go through the same prefix-stripping proxy rule, or be made
  root-relative. Nothing in the current codebase does this (all internal
  navigation is root-relative router links), but it's worth re-checking
  after any change to `crates/web-app/public/` or the asset pipeline.

If the parent system instead wants VoteAssist on its **own subdomain**
(`vote-assist.parent-system.example`) rather than a sub-path, none of the
above applies — that's a plain reverse-proxy virtual host, no path
rewriting needed, and is the simpler option when a subdomain is available.

## 3. Sharing Postgres

Every VoteAssist migration (`rust/migrations/0001` through `0012`) creates
plainly-named tables (`admin_users`, `sessions`, `account_sessions`,
`kb_entries`, `analytics_events`, etc.) in Postgres's default `public`
schema, with no project-specific namespacing. This is fine when VoteAssist
has its own database. It becomes a collision risk if a parent system's
own schema also happens to define a table named, say, `sessions` or
`feedback`, in the same database and schema.

Two supported options, in order of preference:

1. **Dedicated database** (recommended). Point `DATABASE_URL` at a
   database used only by VoteAssist, on the same Postgres cluster as the
   parent system if convenient, or a separate cluster if not. This is the
   simplest option and matches how every migration/backup script in this
   repository (`scripts/setup-db.sh`, `scripts/backup-db.sh`) already
   assumes VoteAssist owns its target database outright.
2. **Dedicated schema, shared database.** If operational constraints
   require one database, create a dedicated Postgres schema (e.g.
   `voteassist`) and a least-privilege role scoped to it
   (`GRANT ALL ON SCHEMA voteassist TO voteassist_app; ALTER ROLE
   voteassist_app SET search_path = voteassist, public;`), then run
   `sqlx migrate run` with that role's connection string. `search_path`
   set at the role level means the unmodified migration files (which use
   unqualified table names) create everything inside `voteassist` without
   any changes to the SQL itself. Confirm this ordering works in a
   throwaway database before running it against anything real — this
   repository's migrations have only ever been run/verified against a
   dedicated database with the default `search_path`.

Either way, VoteAssist's Postgres role should **not** be the parent
system's own superuser/owner role — grant it only what its own migrations
need (`CREATE`, `SELECT`/`INSERT`/`UPDATE`/`DELETE` on its own tables),
consistent with the least-privilege stance in
`docs/SECURITY-AND-SRE-OPERATIONS.md`.

## 4. Session and cookie scoping

VoteAssist sets two first-party session cookies, both `HttpOnly`,
`Secure`, `Path=/`, with no `Domain=` attribute set:

- `va_admin_session` (`admin-app`) — `SameSite=Strict`, 8-hour lifetime.
- `va_account_session` (`web-app`) — `SameSite=Lax`, 30-day lifetime.

No `Domain=` attribute means the browser scopes each cookie to the exact
host that set it (the default, most restrictive behavior) — it will
**not** leak to sibling subdomains or to the parent system's own origin.
For the "reverse-proxy path-mount" deployment in Section 2, this is
exactly correct with no changes needed: browser and proxy both see one
origin (the parent system's own domain), and the cookie is scoped to that
single origin automatically.

For the "own subdomain" deployment, the cookies are similarly scoped to
that subdomain only, which is also correct and requires no change —
VoteAssist's session state has no reason to be visible to the parent
system's own pages, and vice versa.

**Single sign-on is out of scope of the current codebase.** If the parent
system wants a citizen who is already logged into the parent system to
also be recognized by VoteAssist's optional account feature (or vice
versa) without a second login, that is a specific integration to design
and build (e.g. the parent system minting a signed assertion VoteAssist's
`crates/web-app` verifies, similar in shape to SAML/OIDC but much
smaller) — nothing in `crates/web-app/src/accounts/` currently reads any
identity from outside its own OTP-login flow. Treat SSO as a distinct,
explicitly-scoped follow-up task, not something Section 4's cookie
scoping alone solves.

## 5. Environment variable namespacing

VoteAssist's own environment variables are deliberately generic
(`DATABASE_URL`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`,
`SMTP_PASSWORD`, `SMTP_FROM_ADDRESS`, `ANTHROPIC_API_KEY`,
`ACCOUNT_CONTACT_HASH_PEPPER`, `JOBS_HEALTH_ADDR`,
`BOT_TELEGRAM_HEALTH_ADDR`, `API_CORS_ALLOWED_ORIGINS`, per-provider bot
tokens, and so on) — there is no `VOTEASSIST_` prefix on any of them.
This is a real collision risk **only if VoteAssist's processes run in the
same OS process or container as the parent system's own code and share
its environment.** It is not a risk at all if each VoteAssist binary runs
in its own process/container, which is already how this repository's
architecture works (seven independent binaries, no in-process merging of
VoteAssist's own services with each other, let alone with a parent
system).

Recommendation: run each VoteAssist service in its own container/process
with its own environment, exactly as `scripts/dev-up.sh` and the
per-service `README.md`s already assume for local development. Do not
attempt to run VoteAssist's binaries inside the parent system's own
process — that's also the reason Section 6's library-embedding option
mounts a *router*, not VoteAssist's env-var-reading `main()` functions.

## 6. Rust-library-level embedding

Several VoteAssist crates are plain, side-effect-free Rust libraries with
no `main()`, no env var reads, and no I/O beyond what's passed in — a
parent Rust/Axum system can depend on them directly, in-process, with no
separate VoteAssist deployment at all:

- **`core-domain`** — the decision-tree engine itself
  (`Engine::new(tree)`, `Engine::step(state, answer)`). Pure logic, no I/O.
- **`kb-content`** — knowledge-base loading and search. Reads its content
  from `include_str!`'d JSON at compile time, no filesystem/network access
  at runtime.
- **`jurisdiction`** — ECI-vs-SEC jurisdiction resolution logic.
- **`channel-core`** — shared bot-channel rendering (`RenderableNode` and
  friends), if a parent system wants to build its own bot channel against
  VoteAssist's decision engine without using `bot-telegram`/`bot-whatsapp`
  as-is.
- **`analytics`** — event-shape definitions and rollup logic, if a parent
  system wants to feed VoteAssist-shaped analytics events into its own
  pipeline rather than `crates/jobs`' rollup worker.

For the HTTP layer specifically, `crates/api` exposes:

```rust
pub fn build_router() -> axum::Router
```

which returns a complete, self-contained `Router` — routes, Prometheus
metrics middleware, tracing middleware, and (as of this document) a CORS
layer, all pre-wired. A parent Axum application can mount it directly at
a sub-path with `Router::nest`:

```rust
let parent_app = axum::Router::new()
    .nest("/vote-assist/api", api::build_router())
    .route("/", axum::routing::get(parent_home_handler));
    // ...the parent system's own routes
```

This is an alternative to Section 2's reverse-proxy approach specifically
for `crates/api` (the stateless decision-engine HTTP API has no Postgres
dependency and no cookies, so nesting it is straightforward). It does
**not** apply to `crates/web-app`/`crates/admin-app` — Leptos's SSR router
integration (`leptos_axum::generate_route_list`,
`leptos_routes_with_context`) assumes it configures the routes on a
`Router` it's given directly, and mounting a full Leptos app under
`.nest(...)` has known asset-path complications upstream that this
codebase has not attempted or verified. For the two Leptos apps, use
Section 2's reverse-proxy mounting instead.

## 7. CORS

`crates/api` did not, until this document's accompanying change, send any
`Access-Control-Allow-Origin` header — meaning a browser-based caller on a
different origin from the API (exactly the shape of "parent system's own
frontend calls VoteAssist's API directly from the browser, on the parent
system's own domain, while the API itself is deployed elsewhere") would
have every request silently blocked by the browser's same-origin policy,
even though `tower-http`'s `cors` Cargo feature had been enabled since
early in this project.

`crates/api::build_router()` now reads `API_CORS_ALLOWED_ORIGINS` — a
comma-separated list of exact origins (scheme + host \[+ port\], e.g.
`https://parent-system.example,https://staging.parent-system.example`) —
and only sends CORS headers back to origins on that list, for `GET` and
`POST` with a `Content-Type` header (the only methods/headers the API
actually uses). Leaving the variable unset is the correct default for
every deployment shape in Sections 1-6 above **except** the
cross-origin-browser-caller case this section describes: reverse-proxy
path-mounting (Section 2) and same-origin `Router::nest` embedding
(Section 6) both put the browser and the API on the same origin, where
CORS headers are irrelevant either way, and standalone deployments have no
cross-origin caller to permit. Set it only when a browser on a genuinely
different origin needs to call this API directly.

## 8. What is intentionally not solved here

- **Single sign-on** between VoteAssist's optional account system and a
  parent system's own login (Section 4) — a real integration to scope and
  build separately, not a configuration flag.
- **A shared design system / shared component library.** VoteAssist's
  design tokens (`docs/07-design-system.md`) are its own; a parent system
  embedding VoteAssist by reverse-proxy path-mount will see VoteAssist's
  own visual design at that path, not the parent system's. Making the two
  visually consistent is a design/frontend task, independent of anything
  in this document.
- **A shared analytics warehouse.** `crates/analytics` writes to
  VoteAssist's own `analytics_events`/`analytics_rollups_daily` tables
  (Section 3). Feeding those into a parent system's own analytics
  warehouse is an ETL task outside this repository's scope — the
  `analytics` crate's event shape (Section 6) is public and stable enough
  to build that against, but nothing here builds the pipe itself.
