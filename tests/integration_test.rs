use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

// Import from the main crate
use tax2go_search::http::{build_router, routes::AppState};
use tax2go_search::search::IndexManager;

/// Helper to create a test app with a temporary data directory
fn create_test_app() -> (axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let index_manager = Arc::new(IndexManager::new(temp_dir.path().to_path_buf()));
    let state = AppState { index_manager };
    let app = build_router(state);
    (app, temp_dir)
}

/// Helper to make a request and get the response body as JSON
async fn request_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    user_id: Option<Uuid>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request_builder = Request::builder().method(method).uri(uri);

    if let Some(uid) = user_id {
        request_builder = request_builder.header("X-User-Id", uid.to_string());
    }

    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();

    let request = request_builder
        .header("Content-Type", "application/json")
        .body(Body::from(body_bytes))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({}))
    };

    (status, json)
}

#[tokio::test]
async fn test_health_check() {
    let (app, _temp_dir) = create_test_app();

    let (status, body) = request_json(app, "GET", "/health", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_index_and_search_document() {
    let (app, _temp_dir) = create_test_app();
    let user_id = Uuid::new_v4();

    // Index a document
    let index_body = json!({
        "id": "doc1",
        "title": "Rust Programming Language",
        "body": "Rust is a systems programming language that runs blazingly fast",
        "metadata": {
            "tags": ["rust", "programming"],
            "source": "test"
        }
    });

    let (status, response) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user_id),
        Some(index_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], "doc1");
    assert_eq!(response["status"], "success");

    // Search for the document
    let search_body = json!({
        "query": "rust programming",
        "limit": 10
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/search",
        Some(user_id),
        Some(search_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);
    assert_eq!(response["results"][0]["id"], "doc1");
    assert_eq!(response["results"][0]["title"], "Rust Programming Language");
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let (app, _temp_dir) = create_test_app();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    // User 1 indexes a document
    let user1_doc = json!({
        "title": "User 1 Secret Document",
        "body": "This is private data for user 1"
    });

    let (status, _) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user1_id),
        Some(user1_doc),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // User 2 indexes a document
    let user2_doc = json!({
        "title": "User 2 Secret Document",
        "body": "This is private data for user 2"
    });

    let (status, _) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user2_id),
        Some(user2_doc),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // User 1 searches - should only see their own document
    let search_body = json!({
        "query": "Secret Document",
        "limit": 10
    });

    let (status, response) = request_json(
        app.clone(),
        "POST",
        "/v1/search",
        Some(user1_id),
        Some(search_body.clone()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);
    assert!(response["results"][0]["title"]
        .as_str()
        .unwrap()
        .contains("User 1"));

    // User 2 searches - should only see their own document
    let (status, response) = request_json(
        app,
        "POST",
        "/v1/search",
        Some(user2_id),
        Some(search_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);
    assert!(response["results"][0]["title"]
        .as_str()
        .unwrap()
        .contains("User 2"));
}

#[tokio::test]
async fn test_delete_document() {
    let (app, _temp_dir) = create_test_app();
    let user_id = Uuid::new_v4();

    // Index a document
    let index_body = json!({
        "id": "doc-to-delete",
        "title": "Temporary Document",
        "body": "This will be deleted"
    });

    let (status, _) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user_id),
        Some(index_body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify it exists
    let search_body = json!({
        "query": "Temporary",
        "limit": 10
    });

    let (status, response) = request_json(
        app.clone(),
        "POST",
        "/v1/search",
        Some(user_id),
        Some(search_body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);

    // Delete the document
    let delete_body = json!({
        "id": "doc-to-delete"
    });

    let (status, response) = request_json(
        app.clone(),
        "DELETE",
        "/v1/documents",
        Some(user_id),
        Some(delete_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], "doc-to-delete");
    assert_eq!(response["status"], "success");

    // Verify it's deleted
    let (status, response) = request_json(
        app,
        "POST",
        "/v1/search",
        Some(user_id),
        Some(search_body),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 0);
}

#[tokio::test]
async fn test_missing_authentication() {
    let (app, _temp_dir) = create_test_app();

    let search_body = json!({
        "query": "test",
        "limit": 10
    });

    let (status, response) = request_json(app, "POST", "/v1/search", None, Some(search_body)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response["error"], "missing_auth");
}

#[tokio::test]
async fn test_invalid_user_id() {
    let (app, _temp_dir) = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header("X-User-Id", "not-a-uuid")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"query":"test"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_validation_errors() {
    let (app, _temp_dir) = create_test_app();
    let user_id = Uuid::new_v4();

    // Empty title
    let invalid_doc = json!({
        "title": "",
        "body": "Some content"
    });

    let (status, response) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user_id),
        Some(invalid_doc),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["error"], "validation_error");

    // Empty query
    let invalid_search = json!({
        "query": "",
        "limit": 10
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/search",
        Some(user_id),
        Some(invalid_search),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response["error"], "validation_error");
}

#[tokio::test]
async fn test_get_stats() {
    let (app, _temp_dir) = create_test_app();
    let user_id = Uuid::new_v4();

    // Get initial stats (should be 0 documents)
    let (status, response) = request_json(app.clone(), "GET", "/v1/stats", Some(user_id), None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["num_documents"], 0);

    // Index two documents
    for i in 1..=2 {
        let doc = json!({
            "title": format!("Document {}", i),
            "body": format!("Content {}", i)
        });

        let (status, _) = request_json(
            app.clone(),
            "PUT",
            "/v1/documents",
            Some(user_id),
            Some(doc),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Get updated stats
    let (status, response) = request_json(app, "GET", "/v1/stats", Some(user_id), None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["num_documents"], 2);
}

#[tokio::test]
async fn test_document_update() {
    let (app, _temp_dir) = create_test_app();
    let user_id = Uuid::new_v4();

    // Index initial version
    let doc_v1 = json!({
        "id": "updatable-doc",
        "title": "Version 1",
        "body": "Initial content"
    });

    let (status, _) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user_id),
        Some(doc_v1),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Update the document
    let doc_v2 = json!({
        "id": "updatable-doc",
        "title": "Version 2",
        "body": "Updated content"
    });

    let (status, _) = request_json(
        app.clone(),
        "PUT",
        "/v1/documents",
        Some(user_id),
        Some(doc_v2),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Search should return the updated version
    let search_body = json!({
        "query": "Version",
        "limit": 10
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/search",
        Some(user_id),
        Some(search_body),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["total"], 1);
    assert_eq!(response["results"][0]["title"], "Version 2");
}
