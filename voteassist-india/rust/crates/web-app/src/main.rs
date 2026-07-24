//! `voteassist-web-app`: the public-site SSR binary (built by `cargo-leptos`).
//! See `src/app.rs` for the route table and `src/server_fns.rs` for the
//! server functions this binary exposes to the hydrated islands.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::routing::get;
    use axum::Router;
    use axum_prometheus::PrometheusMetricLayer;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::postgres::PgPoolOptions;
    use tower_http::compression::CompressionLayer;
    use web_app::app::{shell, App};

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Only used by `submit_feedback` (see `server_fns.rs`) — every other
    // server function on this site is deliberately database-free (public
    // decision-engine "sessions" are never persisted, per Section 8.7).
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (used only for feedback submission)");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    let conf = get_configuration(None).expect("failed to read Leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || leptos::prelude::provide_context(pool.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        // Layers apply to every route added *before* this point in the
        // chain, so both go last: 2G/low-bandwidth is a stated hard
        // requirement (Section 18) — gzip every response — and
        // docs/SECURITY-AND-SRE-OPERATIONS.md's Prometheus scraping needs
        // request metrics for the real page/server-fn traffic, not just
        // the /healthz and /metrics endpoints themselves.
        .layer(CompressionLayer::new())
        .layer(prometheus_layer)
        .with_state(leptos_options);

    tracing::info!("voteassist-web-app listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind web-app listener");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // No client-side main function — hydration entry is `lib.rs`'s
    // `hydrate()`, invoked by the wasm-bindgen glue cargo-leptos generates.
}
