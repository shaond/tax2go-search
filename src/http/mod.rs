pub mod auth;
pub mod error;
pub mod routes;
pub mod webui;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use routes::AppState;

/// Build the Axum router with all routes and middleware
pub fn build_router(state: AppState, web_ui_enabled: bool) -> Router {
    // API v1 routes - all require authentication
    let api_v1 = Router::new()
        .route("/documents", put(routes::index_document))
        .route("/documents", delete(routes::delete_document))
        .route("/search", post(routes::search_documents))
        .route("/stats", get(routes::get_stats));

    // Main router with health check and API routes
    let mut router = Router::new()
        .route("/health", get(routes::health_check))
        .nest("/v1", api_v1);

    // Conditionally add web UI route
    if web_ui_enabled {
        router = router.route("/ui", get(webui::serve_ui));
    }

    router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::search::IndexManager;

    #[tokio::test]
    async fn test_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let index_manager = Arc::new(IndexManager::new(temp_dir.path().to_path_buf()));
        let state = AppState { index_manager };
        let app = build_router(state, false);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_auth() {
        let temp_dir = TempDir::new().unwrap();
        let index_manager = Arc::new(IndexManager::new(temp_dir.path().to_path_buf()));
        let state = AppState { index_manager };
        let app = build_router(state, false);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/search")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"query":"test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
