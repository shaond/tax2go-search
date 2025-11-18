use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::search::{
    DeleteDocumentInput, HealthResponse, IndexDocumentInput, IndexManager, SearchQuery,
    BrowseDocumentsQuery,
};

use super::auth::CurrentUser;
use super::error::{AppError, AppResult};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub index_manager: Arc<IndexManager>,
}

/// Health check endpoint
///
/// GET /health
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    })
}

/// Index or update a document
///
/// PUT /v1/documents
///
/// This endpoint allows users to add or update documents in their personal index.
/// If a document with the same ID already exists, it will be replaced.
pub async fn index_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<IndexDocumentInput>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %current_user.user_id,
        doc_id = ?input.id,
        "Indexing document"
    );

    // Validate input
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title cannot be empty".to_string()));
    }

    if input.body.trim().is_empty() {
        return Err(AppError::Validation("Body cannot be empty".to_string()));
    }

    // Index the document using the authenticated user's ID
    let response = state
        .index_manager
        .index_document(current_user.user_id, input)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to index document");
            AppError::Index(format!("Failed to index document: {}", e))
        })?;

    Ok((StatusCode::OK, Json(response)))
}

/// Delete a document
///
/// DELETE /v1/documents
///
/// This endpoint allows users to delete documents from their personal index.
pub async fn delete_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<DeleteDocumentInput>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %current_user.user_id,
        doc_id = %input.id,
        "Deleting document"
    );

    if input.id.trim().is_empty() {
        return Err(AppError::Validation("Document ID cannot be empty".to_string()));
    }

    let response = state
        .index_manager
        .delete_document(current_user.user_id, input.id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to delete document");
            AppError::Index(format!("Failed to delete document: {}", e))
        })?;

    Ok((StatusCode::OK, Json(response)))
}

/// Search documents
///
/// POST /v1/search
///
/// This endpoint allows users to search within their personal index.
/// Users can only search their own documents - multi-tenant isolation is enforced.
pub async fn search_documents(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(query): Json<SearchQuery>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %current_user.user_id,
        query = %query.query,
        "Searching documents"
    );

    if query.query.trim().is_empty() {
        return Err(AppError::Validation("Query cannot be empty".to_string()));
    }

    if query.limit == 0 {
        return Err(AppError::Validation("Limit must be greater than 0".to_string()));
    }

    if query.limit > 100 {
        return Err(AppError::Validation("Limit cannot exceed 100".to_string()));
    }

    let response = state
        .index_manager
        .search(current_user.user_id, query)
        .await
        .map_err(|e| {
            error!(error = %e, "Search failed");
            AppError::Search(format!("Search failed: {}", e))
        })?;

    Ok(Json(response))
}

/// Get user index statistics
///
/// GET /v1/stats
///
/// Returns statistics about the current user's index.
pub async fn get_stats(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %current_user.user_id,
        "Getting user stats"
    );

    let stats = state
        .index_manager
        .get_user_stats(current_user.user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get stats");
            AppError::Internal(e)
        })?;

    #[derive(serde::Serialize)]
    struct StatsResponse {
        user_id: String,
        num_documents: usize,
    }

    let response = StatsResponse {
        user_id: stats.user_id.to_string(),
        num_documents: stats.num_documents,
    };

    Ok(Json(response))
}

/// Browse/list all documents for a user
///
/// GET /v1/browse
///
/// Returns all documents in the user's index with optional pagination.
pub async fn browse_documents(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(query): Json<BrowseDocumentsQuery>,
) -> AppResult<impl IntoResponse> {
    info!(
        user_id = %current_user.user_id,
        limit = query.limit,
        offset = query.offset,
        "Browsing documents"
    );

    if query.limit == 0 {
        return Err(AppError::Validation("Limit must be greater than 0".to_string()));
    }

    if query.limit > 1000 {
        return Err(AppError::Validation("Limit cannot exceed 1000".to_string()));
    }

    let response = state
        .index_manager
        .browse_documents(current_user.user_id, query)
        .await
        .map_err(|e| {
            error!(error = %e, "Browse failed");
            AppError::Internal(e)
        })?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_clone() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index_manager = Arc::new(IndexManager::new(temp_dir.path().to_path_buf()));
        let state = AppState {
            index_manager: index_manager.clone(),
        };

        let cloned = state.clone();
        assert!(Arc::ptr_eq(&state.index_manager, &cloned.index_manager));
    }
}
