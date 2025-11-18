use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value};
use tantivy::{Index, IndexReader, IndexWriter, Term, TantivyDocument};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::models::{
    DeleteDocumentResponse, IndexDocumentInput, IndexDocumentResponse, SearchQuery, SearchResponse,
    SearchResult, BrowseDocumentsQuery, BrowseDocumentsResponse, DocumentDetail,
};
use super::schema::{build_schema, doc_from_input, FieldNames};

/// Handle to a user's index with reader and writer
struct IndexHandle {
    index: Index,
    writer: Arc<tokio::sync::Mutex<IndexWriter>>,
    reader: IndexReader,
    schema: Schema,
}

impl IndexHandle {
    /// Create a new index handle for a user
    fn new(index_path: PathBuf) -> Result<Self> {
        let schema = build_schema();

        // Create or open the index
        let index = if index_path.exists() {
            Index::open_in_dir(&index_path)
                .with_context(|| format!("Failed to open index at {:?}", index_path))?
        } else {
            std::fs::create_dir_all(&index_path)
                .with_context(|| format!("Failed to create index directory: {:?}", index_path))?;
            Index::create_in_dir(&index_path, schema.clone())
                .with_context(|| format!("Failed to create index at {:?}", index_path))?
        };

        // Create writer with 50MB heap
        let writer = index
            .writer(50_000_000)
            .context("Failed to create index writer")?;

        // Create reader - will reload automatically or manually as needed
        let reader = index
            .reader()
            .context("Failed to create index reader")?;

        Ok(IndexHandle {
            index,
            writer: Arc::new(tokio::sync::Mutex::new(writer)),
            reader,
            schema,
        })
    }
}

/// Manages per-user Tantivy indexes with strong isolation
///
/// Each user gets their own independent index stored in a separate directory.
/// This ensures that users cannot access other users' documents even if they
/// tamper with request payloads.
pub struct IndexManager {
    /// Base directory for all indexes
    base_dir: PathBuf,

    /// Cache of opened indexes, keyed by user ID
    indexes: Arc<RwLock<HashMap<Uuid, Arc<IndexHandle>>>>,
}

impl IndexManager {
    /// Create a new IndexManager
    pub fn new(base_dir: PathBuf) -> Self {
        IndexManager {
            base_dir,
            indexes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create an index handle for a user
    async fn get_or_create_index(&self, user_id: Uuid) -> Result<Arc<IndexHandle>> {
        // Fast path: check if index is already loaded
        {
            let indexes = self.indexes.read().await;
            if let Some(handle) = indexes.get(&user_id) {
                return Ok(Arc::clone(handle));
            }
        }

        // Slow path: create new index
        let mut indexes = self.indexes.write().await;

        // Double-check in case another task created it
        if let Some(handle) = indexes.get(&user_id) {
            return Ok(Arc::clone(handle));
        }

        // Create index directory path: base_dir/{user_id}/index
        let user_dir = self.base_dir.join(user_id.to_string());
        let index_path = user_dir.join("index");

        info!(
            user_id = %user_id,
            path = ?index_path,
            "Creating new index for user"
        );

        let handle = Arc::new(IndexHandle::new(index_path)?);
        indexes.insert(user_id, Arc::clone(&handle));

        Ok(handle)
    }

    /// Index or update a document for a user
    ///
    /// If a document with the same ID exists, it will be deleted and re-added.
    pub async fn index_document(
        &self,
        user_id: Uuid,
        input: IndexDocumentInput,
    ) -> Result<IndexDocumentResponse> {
        let handle = self.get_or_create_index(user_id).await?;

        let doc = doc_from_input(&handle.schema, &input)
            .context("Failed to create document from input")?;

        let doc_id = input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

        let id_field = handle.schema
            .get_field(FieldNames::ID)
            .context("ID field not found in schema")?;

        let mut writer = handle.writer.lock().await;

        // Delete existing document with the same ID (if any)
        let term = Term::from_field_text(id_field, &doc_id);
        writer.delete_term(term);

        // Add the new document
        writer.add_document(doc)?;

        // Commit changes
        writer.commit()?;

        debug!(
            user_id = %user_id,
            doc_id = %doc_id,
            "Document indexed successfully"
        );

        Ok(IndexDocumentResponse {
            id: doc_id,
            status: "success".to_string(),
            message: "Document indexed successfully".to_string(),
        })
    }

    /// Delete a document by ID for a user
    pub async fn delete_document(
        &self,
        user_id: Uuid,
        document_id: String,
    ) -> Result<DeleteDocumentResponse> {
        let handle = self.get_or_create_index(user_id).await?;

        let id_field = handle.schema
            .get_field(FieldNames::ID)
            .context("ID field not found in schema")?;

        let mut writer = handle.writer.lock().await;

        let term = Term::from_field_text(id_field, &document_id);
        writer.delete_term(term);
        writer.commit()?;

        debug!(
            user_id = %user_id,
            doc_id = %document_id,
            "Document deleted successfully"
        );

        Ok(DeleteDocumentResponse {
            id: document_id,
            status: "success".to_string(),
            message: "Document deleted successfully".to_string(),
        })
    }

    /// Search documents for a user
    ///
    /// This method ensures that only the user's own documents are searched.
    pub async fn search(
        &self,
        user_id: Uuid,
        query: SearchQuery,
    ) -> Result<SearchResponse> {
        let start = Instant::now();

        let handle = self.get_or_create_index(user_id).await?;

        // Reload the reader to see latest commits
        handle.reader.reload()?;
        let searcher = handle.reader.searcher();

        // Build query parser for title and body fields
        let title_field = handle.schema
            .get_field(FieldNames::TITLE)
            .context("Title field not found")?;
        let body_field = handle.schema
            .get_field(FieldNames::BODY)
            .context("Body field not found")?;

        let query_parser = QueryParser::for_index(&handle.index, vec![title_field, body_field]);

        // Parse the query
        let parsed_query = query_parser
            .parse_query(&query.query)
            .context("Failed to parse search query")?;

        // Execute search
        let limit = query.limit.min(100); // Cap at 100 results
        let offset = query.offset;
        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit + offset))?;

        // Convert results
        let mut results = Vec::new();
        let id_field = handle.schema.get_field(FieldNames::ID).context("ID field not found")?;
        let created_at_field = handle.schema.get_field(FieldNames::CREATED_AT).ok();

        for (_score, doc_address) in top_docs.into_iter().skip(offset).take(limit) {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let title = retrieved_doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let body = retrieved_doc
                .get_first(body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let created_at = created_at_field
                .and_then(|f| retrieved_doc.get_first(f))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            results.push(SearchResult {
                id,
                title,
                body, // Complete body, not truncated
                score: _score,
                created_at,
                snippet: None, // TODO: Implement snippet generation
            });
        }

        let took_ms = start.elapsed().as_millis() as u64;
        let total = results.len();

        debug!(
            user_id = %user_id,
            query = %query.query,
            results = total,
            took_ms = took_ms,
            "Search completed"
        );

        Ok(SearchResponse {
            results,
            total,
            query: query.query,
            took_ms,
        })
    }

    /// Get statistics about a user's index
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserIndexStats> {
        let handle = self.get_or_create_index(user_id).await?;

        // Reload the reader to see latest commits
        handle.reader.reload()?;
        let searcher = handle.reader.searcher();

        let num_docs = searcher.num_docs() as usize;

        Ok(UserIndexStats {
            user_id,
            num_documents: num_docs,
        })
    }

    /// Browse/list all documents for a user
    ///
    /// Returns complete documents without requiring a search query.
    pub async fn browse_documents(
        &self,
        user_id: Uuid,
        query: BrowseDocumentsQuery,
    ) -> Result<BrowseDocumentsResponse> {
        let start = Instant::now();

        let handle = self.get_or_create_index(user_id).await?;

        // Reload the reader to see latest commits
        handle.reader.reload()?;
        let searcher = handle.reader.searcher();

        // Get field handles
        let id_field = handle.schema.get_field(FieldNames::ID).context("ID field not found")?;
        let title_field = handle.schema.get_field(FieldNames::TITLE).context("Title field not found")?;
        let body_field = handle.schema.get_field(FieldNames::BODY).context("Body field not found")?;
        let created_at_field = handle.schema.get_field(FieldNames::CREATED_AT).ok();
        let tags_field = handle.schema.get_field(FieldNames::TAGS).ok();

        // Use a match-all query to get all documents
        use tantivy::query::AllQuery;
        let all_query = AllQuery;

        // Get all documents, limited by the query parameters
        let limit = query.limit.min(1000); // Cap at 1000 documents
        let offset = query.offset;
        let top_docs = searcher.search(&all_query, &TopDocs::with_limit(limit + offset))?;

        // Convert results
        let mut documents = Vec::new();

        for (_score, doc_address) in top_docs.into_iter().skip(offset).take(limit) {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let title = retrieved_doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let body = retrieved_doc
                .get_first(body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let created_at = created_at_field
                .and_then(|f| retrieved_doc.get_first(f))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Extract tags from the document
            let tags = if let Some(tags_f) = tags_field {
                retrieved_doc
                    .get_all(tags_f)
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                Vec::new()
            };

            documents.push(DocumentDetail {
                id,
                title,
                body, // Full body, not truncated
                created_at,
                tags,
            });
        }

        let took_ms = start.elapsed().as_millis() as u64;
        let total = documents.len();

        debug!(
            user_id = %user_id,
            documents = total,
            took_ms = took_ms,
            "Browse completed"
        );

        Ok(BrowseDocumentsResponse {
            documents,
            total,
            took_ms,
        })
    }
}

/// Statistics about a user's index
#[derive(Debug, Clone)]
pub struct UserIndexStats {
    pub user_id: Uuid,
    pub num_documents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::models::DocumentMetadata;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let manager = IndexManager::new(temp_dir.path().to_path_buf());

        let user_id = Uuid::new_v4();

        // Index a document
        let input = IndexDocumentInput {
            id: Some("doc1".to_string()),
            title: "Rust Programming".to_string(),
            body: "Rust is a systems programming language".to_string(),
            metadata: DocumentMetadata::default(),
        };

        let response = manager.index_document(user_id, input).await.unwrap();
        assert_eq!(response.id, "doc1");

        // Search for the document
        let query = SearchQuery {
            query: "Rust".to_string(),
            limit: 10,
            offset: 0,
            filters: Default::default(),
        };

        let search_response = manager.search(user_id, query).await.unwrap();
        assert_eq!(search_response.results.len(), 1);
        assert_eq!(search_response.results[0].title, "Rust Programming");
    }

    #[tokio::test]
    async fn test_multi_tenant_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = IndexManager::new(temp_dir.path().to_path_buf());

        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();

        // User 1 indexes a document
        let input1 = IndexDocumentInput {
            id: Some("doc1".to_string()),
            title: "User 1 Document".to_string(),
            body: "This belongs to user 1".to_string(),
            metadata: DocumentMetadata::default(),
        };
        manager.index_document(user1_id, input1).await.unwrap();

        // User 2 indexes a document
        let input2 = IndexDocumentInput {
            id: Some("doc2".to_string()),
            title: "User 2 Document".to_string(),
            body: "This belongs to user 2".to_string(),
            metadata: DocumentMetadata::default(),
        };
        manager.index_document(user2_id, input2).await.unwrap();

        // User 1 searches - should only see their document
        let query = SearchQuery {
            query: "Document".to_string(),
            limit: 10,
            offset: 0,
            filters: Default::default(),
        };

        let user1_results = manager.search(user1_id, query.clone()).await.unwrap();
        assert_eq!(user1_results.results.len(), 1);
        assert!(user1_results.results[0].title.contains("User 1"));

        // User 2 searches - should only see their document
        let user2_results = manager.search(user2_id, query).await.unwrap();
        assert_eq!(user2_results.results.len(), 1);
        assert!(user2_results.results[0].title.contains("User 2"));
    }
}
