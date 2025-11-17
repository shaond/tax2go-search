use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Index error: {0}")]
    Index(String),
}

/// Error response body
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match self {
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error".to_string(),
                "An internal error occurred".to_string(),
                Some(err.to_string()),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "bad_request".to_string(),
                msg,
                None,
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "not_found".to_string(),
                msg,
                None,
            ),
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error".to_string(),
                msg,
                None,
            ),
            AppError::Search(msg) => (
                StatusCode::BAD_REQUEST,
                "search_error".to_string(),
                msg,
                None,
            ),
            AppError::Index(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "index_error".to_string(),
                msg,
                None,
            ),
        };

        let body = ErrorResponse {
            error: error_type,
            message,
            details,
        };

        (status, Json(body)).into_response()
    }
}

/// Helper to convert anyhow::Error to AppError
impl From<tantivy::TantivyError> for AppError {
    fn from(err: tantivy::TantivyError) -> Self {
        AppError::Internal(err.into())
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;
