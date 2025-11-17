# Tax2Go Search - Full-Text Search Microservice

A production-grade HTTP microservice built with Rust, Axum, and Tantivy providing multi-tenant full-text search capabilities with strong isolation guarantees.

## Features

- **Multi-tenant architecture** - Each user gets their own isolated search index
- **Strong security** - User isolation enforced at the index level, not just filters
- **Full-text search** - Powered by Tantivy, a fast full-text search engine
- **RESTful API** - Clean HTTP API with JSON payloads
- **Production-ready** - Comprehensive error handling, logging, and testing

## Architecture

### Multi-Tenant Isolation

Security is the top priority. This service implements multi-tenant isolation through:

1. **Separate indexes per user** - Each user's documents are stored in a completely separate Tantivy index on disk (`${DATA_DIR}/{user_id}/index`)
2. **Authentication enforcement** - All operations require a valid `X-User-Id` header
3. **No cross-user access** - Even if a malicious user tampers with requests, they cannot access other users' data

### Technology Stack

- **Axum** - Modern async web framework
- **Tokio** - Async runtime
- **Tantivy** - Full-text search engine library
- **Tower** - Middleware for timeout, tracing, CORS
- **Tracing** - Structured logging

## Getting Started

### Prerequisites

- Rust 1.75 or newer
- Cargo

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd tax2go-search
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and set the required configuration:
```env
DATA_DIR=./data
BIND_ADDR=127.0.0.1:8080
LOG_LEVEL=info
```

4. Build the project:
```bash
cargo build --release
```

### Running the Service

```bash
cargo run --release
```

Or run the binary directly:
```bash
./target/release/tax2go-search
```

The service will start on the configured bind address (default: `127.0.0.1:8080`).

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_multi_tenant_isolation
```

## API Reference

### Authentication

All API endpoints (except `/health`) require authentication via the `X-User-Id` header:

```bash
X-User-Id: 550e8400-e29b-41d4-a716-446655440000
```

The value must be a valid UUID. In production, this would be replaced with JWT validation or session management.

### Endpoints

#### Health Check

```http
GET /health
```

Returns service status. No authentication required.

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

#### Index a Document

```http
PUT /v1/documents
X-User-Id: <uuid>
Content-Type: application/json

{
  "id": "optional-document-id",
  "title": "Document Title",
  "body": "Full text content of the document",
  "metadata": {
    "tags": ["optional", "tags"],
    "source": "optional-source",
    "created_at": "2025-01-01T12:00:00Z"
  }
}
```

If `id` is not provided, a UUID will be generated. If a document with the same ID exists, it will be replaced.

**Response:**
```json
{
  "id": "document-id",
  "status": "success",
  "message": "Document indexed successfully"
}
```

#### Delete a Document

```http
DELETE /v1/documents
X-User-Id: <uuid>
Content-Type: application/json

{
  "id": "document-id"
}
```

**Response:**
```json
{
  "id": "document-id",
  "status": "success",
  "message": "Document deleted successfully"
}
```

#### Search Documents

```http
POST /v1/search
X-User-Id: <uuid>
Content-Type: application/json

{
  "query": "search terms",
  "limit": 10,
  "offset": 0,
  "filters": {
    "tags": [],
    "source": null
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "document-id",
      "title": "Document Title",
      "body": "Document body (truncated to 500 chars)...",
      "score": 1.234,
      "created_at": "2025-01-01T12:00:00Z",
      "snippet": null
    }
  ],
  "total": 1,
  "query": "search terms",
  "took_ms": 15
}
```

#### Get Index Statistics

```http
GET /v1/stats
X-User-Id: <uuid>
```

Returns statistics about the current user's index.

**Response:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "num_documents": 42
}
```

## Example Usage

### Using cURL

```bash
# Set user ID
USER_ID="550e8400-e29b-41d4-a716-446655440000"

# Health check
curl http://localhost:8080/health

# Index a document
curl -X PUT http://localhost:8080/v1/documents \
  -H "X-User-Id: $USER_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Rust Programming Guide",
    "body": "Rust is a systems programming language focused on safety, speed, and concurrency.",
    "metadata": {
      "tags": ["rust", "programming"],
      "source": "tutorial"
    }
  }'

# Search documents
curl -X POST http://localhost:8080/v1/search \
  -H "X-User-Id: $USER_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "rust programming",
    "limit": 10
  }'

# Get statistics
curl http://localhost:8080/v1/stats \
  -H "X-User-Id: $USER_ID"

# Delete a document
curl -X DELETE http://localhost:8080/v1/documents \
  -H "X-User-Id: $USER_ID" \
  -H "Content-Type: application/json" \
  -d '{"id": "document-id"}'
```

### Using HTTPie

```bash
USER_ID="550e8400-e29b-41d4-a716-446655440000"

# Index a document
http PUT localhost:8080/v1/documents \
  X-User-Id:$USER_ID \
  title="Rust Programming" \
  body="Rust is awesome"

# Search
http POST localhost:8080/v1/search \
  X-User-Id:$USER_ID \
  query="rust"
```

## Project Structure

```
tax2go-search/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── http/
│   │   ├── mod.rs           # HTTP router setup
│   │   ├── routes.rs        # Request handlers
│   │   ├── error.rs         # Error types and handling
│   │   └── auth.rs          # Authentication middleware
│   └── search/
│       ├── mod.rs           # Search module exports
│       ├── index_manager.rs # Multi-tenant index management
│       ├── schema.rs        # Tantivy schema definition
│       └── models.rs        # Request/response models
├── tests/                   # Integration tests
├── Cargo.toml              # Dependencies and metadata
├── .env.example            # Example configuration
└── README.md               # This file
```

## Security Considerations

### Current Authentication

The current implementation uses a simple header-based authentication (`X-User-Id`) for demonstration purposes. **This is NOT suitable for production use.**

### Production Recommendations

For production deployment, replace the authentication system with:

1. **JWT validation** - Verify signed JSON Web Tokens
2. **OAuth 2.0** - Integrate with identity providers
3. **API Keys** - Use API key authentication with rate limiting
4. **mTLS** - Mutual TLS for service-to-service communication

### Multi-Tenant Isolation

The service implements defense-in-depth for multi-tenant isolation:

1. **Index-level isolation** - Each user gets a completely separate index directory
2. **Authentication at the middleware layer** - Invalid requests never reach handlers
3. **User ID from trusted source** - The `user_id` for operations comes from the authentication layer, never from request payloads

## Performance Considerations

### Index Writer Configuration

- Each user's index uses a 50MB heap for the writer
- Writers are shared across requests for the same user
- Commits are performed after each write operation

### Caching

- Index handles are cached in memory using `Arc<RwLock<HashMap>>`
- Once opened, an index remains in memory for subsequent operations
- Readers use Tantivy's `OnCommitWithDelay` reload policy for near-real-time search

### Scalability

- The service can handle multiple concurrent users efficiently
- Each user's index is independent, allowing horizontal scaling
- Consider using a distributed file system for `DATA_DIR` in clustered deployments

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check without building
cargo check
```

### Adding New Features

1. Update models in `src/search/models.rs`
2. Add handler in `src/http/routes.rs`
3. Register route in `src/http/mod.rs`
4. Add tests in the relevant module

## Troubleshooting

### Service won't start

- Check that `DATA_DIR` is writable
- Verify `BIND_ADDR` is not already in use
- Check logs for detailed error messages

### Search returns no results

- Ensure documents are indexed successfully
- Verify the `X-User-Id` header matches the user who indexed the documents
- Check that the query syntax is valid

### Performance issues

- Monitor the number of open indexes (one per active user)
- Consider implementing index cleanup for inactive users
- Review system resources (disk I/O, memory)

## License

[Specify your license here]

## Contributing

[Specify contribution guidelines here]
