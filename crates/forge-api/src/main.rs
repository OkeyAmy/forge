mod routes;

use axum::{Router, routing::{delete, get, post}};
use routes::watch::new_watch_state;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("forge_api=info".parse().expect("valid directive"))
                .add_directive("tower_http=debug".parse().expect("valid directive")),
        )
        .init();

    let watch_state = new_watch_state();

    let app = Router::new()
        // health
        .route("/health", get(routes::health::handler))
        // single run
        .route("/api/run", post(routes::run::run_handler))
        // batch run
        .route("/api/run/batch", post(routes::run::batch_handler))
        // github issues
        .route("/api/issues", get(routes::issues::handler))
        // trajectory files
        .route("/api/trajectories", get(routes::trajectories::list_handler))
        .route("/api/trajectories/{name}", get(routes::trajectories::get_handler))
        // trajectory stats
        .route("/api/stats", get(routes::stats::handler))
        // watch mode
        .route("/api/watch", post(routes::watch::start_handler))
        .route("/api/watch", get(routes::watch::status_handler))
        .route("/api/watch", delete(routes::watch::stop_handler))
        .with_state(watch_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("FORGE_API_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5000);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind {addr}: {e}"));

    tracing::info!("forge-api listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
