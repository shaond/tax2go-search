use tantivy::schema::{Schema, TextOptions, TextFieldIndexing, IndexRecordOption, Value, STORED, STRING, TEXT};
use tantivy::{TantivyError};
use tantivy::TantivyDocument;
use chrono::Utc;
use uuid::Uuid;

use super::models::IndexDocumentInput;

/// Field names used in the Tantivy schema
pub struct FieldNames;

impl FieldNames {
    pub const ID: &'static str = "id";
    pub const TITLE: &'static str = "title";
    pub const BODY: &'static str = "body";
    pub const CREATED_AT: &'static str = "created_at";
    pub const TAGS: &'static str = "tags";
    pub const SOURCE: &'static str = "source";
}

/// Build the Tantivy schema for document indexing
///
/// Fields:
/// - id: String field (stored, indexed) - unique document identifier
/// - title: Text field (stored, indexed) - document title
/// - body: Text field (stored, indexed) - document content
/// - created_at: Text field (stored) - ISO 8601 timestamp
/// - tags: Text field (indexed) - searchable tags
/// - source: Text field (stored, indexed) - optional source identifier
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // ID field - stored and indexed as a string
    schema_builder.add_text_field(FieldNames::ID, STRING | STORED);

    // Title - full-text searchable and stored
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("default")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    schema_builder.add_text_field(FieldNames::TITLE, text_options.clone());

    // Body - full-text searchable and stored
    schema_builder.add_text_field(FieldNames::BODY, text_options);

    // Created timestamp - stored as text (ISO 8601)
    schema_builder.add_text_field(FieldNames::CREATED_AT, STRING | STORED);

    // Tags - indexed for filtering
    schema_builder.add_text_field(FieldNames::TAGS, TEXT | STORED);

    // Source - stored and indexed as string
    schema_builder.add_text_field(FieldNames::SOURCE, STRING | STORED);

    schema_builder.build()
}

/// Convert an IndexDocumentInput into a Tantivy Document
pub fn doc_from_input(schema: &Schema, input: &IndexDocumentInput) -> Result<TantivyDocument, TantivyError> {
    let mut doc = TantivyDocument::default();

    // Get field handles - these should always exist in our schema
    let id_field = schema.get_field(FieldNames::ID)
        .expect("ID field must exist in schema");
    let title_field = schema.get_field(FieldNames::TITLE)
        .expect("Title field must exist in schema");
    let body_field = schema.get_field(FieldNames::BODY)
        .expect("Body field must exist in schema");
    let created_at_field = schema.get_field(FieldNames::CREATED_AT)
        .expect("Created_at field must exist in schema");
    let tags_field = schema.get_field(FieldNames::TAGS)
        .expect("Tags field must exist in schema");
    let source_field = schema.get_field(FieldNames::SOURCE)
        .expect("Source field must exist in schema");

    // ID - use provided ID or generate a new UUID
    let doc_id = input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    doc.add_text(id_field, &doc_id);

    // Title and body
    doc.add_text(title_field, &input.title);
    doc.add_text(body_field, &input.body);

    // Created timestamp
    let created_at = input.metadata.created_at
        .unwrap_or_else(Utc::now)
        .to_rfc3339();
    doc.add_text(created_at_field, &created_at);

    // Tags
    for tag in &input.metadata.tags {
        doc.add_text(tags_field, tag);
    }

    // Source
    if let Some(ref source) = input.metadata.source {
        doc.add_text(source_field, source);
    }

    Ok(doc)
}

/// Extract document ID from a Tantivy document
pub fn extract_doc_id(schema: &Schema, doc: &TantivyDocument) -> Option<String> {
    let id_field = schema.get_field(FieldNames::ID).ok()?;
    doc.get_first(id_field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::models::DocumentMetadata;

    #[test]
    fn test_schema_creation() {
        let schema = build_schema();

        assert!(schema.get_field(FieldNames::ID).is_ok());
        assert!(schema.get_field(FieldNames::TITLE).is_ok());
        assert!(schema.get_field(FieldNames::BODY).is_ok());
        assert!(schema.get_field(FieldNames::CREATED_AT).is_ok());
    }

    #[test]
    fn test_doc_from_input() {
        let schema = build_schema();
        let input = IndexDocumentInput {
            id: Some("test-123".to_string()),
            title: "Test Document".to_string(),
            body: "This is a test document body.".to_string(),
            metadata: DocumentMetadata {
                tags: vec!["test".to_string(), "demo".to_string()],
                source: Some("unit-test".to_string()),
                created_at: None,
                custom: Default::default(),
            },
        };

        let doc = doc_from_input(&schema, &input).unwrap();
        let extracted_id = extract_doc_id(&schema, &doc);

        assert_eq!(extracted_id, Some("test-123".to_string()));
    }
}
