use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input for indexing a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentInput {
    /// Optional client-provided document ID. If None, a UUID will be generated.
    pub id: Option<String>,

    /// Document title
    pub title: String,

    /// Document body/content
    pub body: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optional source identifier
    pub source: Option<String>,

    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,

    /// Additional custom fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Response after indexing a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentResponse {
    /// Document ID
    pub id: String,

    /// Operation status
    pub status: String,

    /// Message
    pub message: String,
}

/// Input for deleting a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDocumentInput {
    /// Document ID to delete
    pub id: String,
}

/// Response after deleting a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDocumentResponse {
    /// Document ID
    pub id: String,

    /// Operation status
    pub status: String,

    /// Message
    pub message: String,
}

/// Search query input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query string
    pub query: String,

    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,

    /// Optional filters
    #[serde(default)]
    pub filters: SearchFilters,
}

fn default_limit() -> usize {
    10
}

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by tags (any match)
    #[serde(default)]
    pub tags: Vec<String>,

    /// Filter by source
    pub source: Option<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,

    /// Document title
    pub title: String,

    /// Document body (may be truncated)
    pub body: String,

    /// Search score
    pub score: f32,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Snippet/highlight (optional)
    pub snippet: Option<String>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,

    /// Total number of results found
    pub total: usize,

    /// Query that was executed
    pub query: String,

    /// Time taken in milliseconds
    pub took_ms: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Optional additional info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Browse/list documents request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseDocumentsQuery {
    /// Maximum number of results to return
    #[serde(default = "default_browse_limit")]
    pub limit: usize,

    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
}

fn default_browse_limit() -> usize {
    50
}

/// Document details for browse response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDetail {
    /// Document ID
    pub id: String,

    /// Document title
    pub title: String,

    /// Complete document body
    pub body: String,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Browse response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseDocumentsResponse {
    /// Documents
    pub documents: Vec<DocumentDetail>,

    /// Total number of documents returned
    pub total: usize,

    /// Time taken in milliseconds
    pub took_ms: u64,
}
