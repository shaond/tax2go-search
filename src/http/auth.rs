use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

/// Represents an authenticated user
///
/// This extractor reads the X-User-Id header and validates it as a UUID.
/// In a production system, this would validate a JWT or session token.
#[derive(Debug, Clone, Copy)]
pub struct CurrentUser {
    pub user_id: Uuid,
}

impl CurrentUser {
    pub fn new(user_id: Uuid) -> Self {
        CurrentUser { user_id }
    }
}

/// Error response for authentication failures
#[derive(Debug, Serialize)]
pub struct AuthError {
    error: String,
    message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":"internal_error","message":"Failed to serialize error"}"#.to_string()
        });

        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract X-User-Id header
        let user_id_header = parts
            .headers
            .get("X-User-Id")
            .ok_or_else(|| AuthError {
                error: "missing_auth".to_string(),
                message: "X-User-Id header is required".to_string(),
            })?;

        // Convert header value to string
        let user_id_str = user_id_header.to_str().map_err(|_| AuthError {
            error: "invalid_auth".to_string(),
            message: "X-User-Id header contains invalid characters".to_string(),
        })?;

        // Parse as UUID
        let user_id = Uuid::parse_str(user_id_str).map_err(|_| AuthError {
            error: "invalid_auth".to_string(),
            message: "X-User-Id must be a valid UUID".to_string(),
        })?;

        Ok(CurrentUser { user_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::body::Body;

    #[tokio::test]
    async fn test_current_user_extractor() {
        let user_id = Uuid::new_v4();
        let mut req = Request::builder()
            .header("X-User-Id", user_id.to_string())
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = req.into_parts();

        let current_user = CurrentUser::from_request_parts(&mut parts, &())
            .await
            .unwrap();

        assert_eq!(current_user.user_id, user_id);
    }

    #[tokio::test]
    async fn test_current_user_missing_header() {
        let mut req = Request::builder()
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = req.into_parts();

        let result = CurrentUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_current_user_invalid_uuid() {
        let mut req = Request::builder()
            .header("X-User-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let (mut parts, _body) = req.into_parts();

        let result = CurrentUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
    }
}
