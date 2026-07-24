//! `voteassist-admin-app`: the standalone admin-dashboard SSR binary
//! (built by `cargo-leptos`), served as a genuinely separate deployment
//! from `voteassist-web-app` — different port, different binary, no
//! shared process — per this crate's blast-radius-containment rationale.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use admin_app::app::{shell, App};
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::postgres::PgPoolOptions;
    use tower_http::compression::CompressionLayer;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    let conf = get_configuration(None).expect("failed to read Leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
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
        .layer(CompressionLayer::new())
        .with_state(leptos_options);

    tracing::info!("voteassist-admin-app listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind admin-app listener");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // No client-side main function — hydration entry is `lib.rs`'s
    // `hydrate()`, invoked by the wasm-bindgen glue cargo-leptos generates.
}
