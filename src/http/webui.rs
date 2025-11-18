use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

/// Serve the web UI HTML page
///
/// GET /ui
pub async fn serve_ui() -> impl IntoResponse {
    (StatusCode::OK, Html(HTML_CONTENT))
}

const HTML_CONTENT: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tax2Go Search - Web UI</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
        }

        header {
            text-align: center;
            color: white;
            margin-bottom: 40px;
        }

        h1 {
            font-size: 2.5rem;
            margin-bottom: 10px;
        }

        .subtitle {
            opacity: 0.9;
            font-size: 1.1rem;
        }

        .panels {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 20px;
        }

        @media (max-width: 768px) {
            .panels {
                grid-template-columns: 1fr;
            }
        }

        .panel {
            background: white;
            border-radius: 12px;
            padding: 30px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
        }

        .panel h2 {
            color: #667eea;
            margin-bottom: 20px;
            font-size: 1.5rem;
        }

        .form-group {
            margin-bottom: 15px;
        }

        label {
            display: block;
            margin-bottom: 5px;
            color: #333;
            font-weight: 500;
        }

        input, textarea {
            width: 100%;
            padding: 12px;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 14px;
            transition: border-color 0.3s;
        }

        input:focus, textarea:focus {
            outline: none;
            border-color: #667eea;
        }

        textarea {
            resize: vertical;
            min-height: 100px;
            font-family: inherit;
        }

        button {
            width: 100%;
            padding: 14px;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.3s;
        }

        button:hover {
            background: #5568d3;
        }

        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }

        .delete-btn {
            background: #ef4444;
        }

        .delete-btn:hover {
            background: #dc2626;
        }

        .results {
            background: white;
            border-radius: 12px;
            padding: 30px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
        }

        .results h2 {
            color: #667eea;
            margin-bottom: 20px;
            font-size: 1.5rem;
        }

        .result-item {
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 15px;
            transition: border-color 0.3s;
        }

        .result-item:hover {
            border-color: #667eea;
        }

        .result-title {
            font-size: 1.2rem;
            font-weight: 600;
            color: #333;
            margin-bottom: 8px;
        }

        .result-body {
            color: #666;
            line-height: 1.6;
            margin-bottom: 10px;
        }

        .result-meta {
            display: flex;
            gap: 15px;
            font-size: 0.875rem;
            color: #999;
        }

        .result-score {
            background: #667eea;
            color: white;
            padding: 4px 10px;
            border-radius: 4px;
            font-weight: 600;
        }

        .message {
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            display: none;
        }

        .message.show {
            display: block;
        }

        .message.success {
            background: #d1fae5;
            color: #065f46;
            border: 2px solid #10b981;
        }

        .message.error {
            background: #fee2e2;
            color: #991b1b;
            border: 2px solid #ef4444;
        }

        .no-results {
            text-align: center;
            color: #999;
            padding: 40px;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Tax2Go Search</h1>
            <p class="subtitle">Full-Text Search Engine Web Interface</p>
        </header>

        <div id="message" class="message"></div>

        <div class="panels">
            <div class="panel">
                <h2>Add Document</h2>
                <form id="addForm">
                    <div class="form-group">
                        <label for="userId">User ID (UUID)</label>
                        <input type="text" id="userId" placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000" required>
                    </div>
                    <div class="form-group">
                        <label for="docId">Document ID (optional)</label>
                        <input type="text" id="docId" placeholder="Auto-generated if empty">
                    </div>
                    <div class="form-group">
                        <label for="title">Title</label>
                        <input type="text" id="title" placeholder="Enter document title" required>
                    </div>
                    <div class="form-group">
                        <label for="body">Content</label>
                        <textarea id="body" placeholder="Enter document content" required></textarea>
                    </div>
                    <div class="form-group">
                        <label for="tags">Tags (comma-separated)</label>
                        <input type="text" id="tags" placeholder="e.g., important, work, tax">
                    </div>
                    <button type="submit">Add Document</button>
                </form>
            </div>

            <div class="panel">
                <h2>Search Documents</h2>
                <form id="searchForm">
                    <div class="form-group">
                        <label for="searchUserId">User ID (UUID)</label>
                        <input type="text" id="searchUserId" placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000" required>
                    </div>
                    <div class="form-group">
                        <label for="query">Search Query</label>
                        <input type="text" id="query" placeholder="Enter search terms" required>
                    </div>
                    <div class="form-group">
                        <label for="limit">Result Limit</label>
                        <input type="number" id="limit" value="10" min="1" max="100">
                    </div>
                    <button type="submit">Search</button>
                </form>

                <h2 style="margin-top: 30px;">Browse Index</h2>
                <form id="browseForm">
                    <div class="form-group">
                        <label for="browseUserId">User ID (UUID)</label>
                        <input type="text" id="browseUserId" placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000" required>
                    </div>
                    <div class="form-group">
                        <label for="browseLimit">Documents to Show</label>
                        <input type="number" id="browseLimit" value="50" min="1" max="1000">
                    </div>
                    <button type="submit" style="background: #059669;">Browse All Documents</button>
                </form>

                <h2 style="margin-top: 30px;">Delete Document</h2>
                <form id="deleteForm">
                    <div class="form-group">
                        <label for="deleteUserId">User ID (UUID)</label>
                        <input type="text" id="deleteUserId" placeholder="e.g., 550e8400-e29b-41d4-a716-446655440000" required>
                    </div>
                    <div class="form-group">
                        <label for="deleteDocId">Document ID</label>
                        <input type="text" id="deleteDocId" placeholder="Enter document ID to delete" required>
                    </div>
                    <button type="submit" class="delete-btn">Delete Document</button>
                </form>
            </div>
        </div>

        <div class="results">
            <h2>Search Results</h2>
            <div id="results">
                <div class="no-results">No search results yet. Use the search form above to find documents.</div>
            </div>
        </div>
    </div>

    <script>
        const API_BASE = window.location.origin;

        function showMessage(text, type) {
            const messageEl = document.getElementById('message');
            messageEl.textContent = text;
            messageEl.className = `message show ${type}`;
            setTimeout(() => {
                messageEl.className = 'message';
            }, 5000);
        }

        // Add Document
        document.getElementById('addForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const userId = document.getElementById('userId').value.trim();
            const docId = document.getElementById('docId').value.trim();
            const title = document.getElementById('title').value.trim();
            const body = document.getElementById('body').value.trim();
            const tags = document.getElementById('tags').value.split(',').map(t => t.trim()).filter(t => t);

            try {
                const response = await fetch(`${API_BASE}/v1/documents`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-User-Id': userId
                    },
                    body: JSON.stringify({
                        id: docId || null,
                        title,
                        body,
                        metadata: {
                            tags,
                            source: 'web-ui',
                            created_at: null,
                            custom: {}
                        }
                    })
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.message || 'Failed to add document');
                }

                const result = await response.json();
                showMessage(`Document added successfully! ID: ${result.id}`, 'success');

                // Clear form
                document.getElementById('docId').value = '';
                document.getElementById('title').value = '';
                document.getElementById('body').value = '';
                document.getElementById('tags').value = '';
            } catch (error) {
                showMessage(`Error: ${error.message}`, 'error');
            }
        });

        // Search Documents
        document.getElementById('searchForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const userId = document.getElementById('searchUserId').value.trim();
            const query = document.getElementById('query').value.trim();
            const limit = parseInt(document.getElementById('limit').value);

            try {
                const response = await fetch(`${API_BASE}/v1/search`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-User-Id': userId
                    },
                    body: JSON.stringify({
                        query,
                        limit,
                        offset: 0
                    })
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.message || 'Search failed');
                }

                const result = await response.json();
                displaySearchResults(result);
            } catch (error) {
                showMessage(`Error: ${error.message}`, 'error');
            }
        });

        // Browse Documents
        document.getElementById('browseForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const userId = document.getElementById('browseUserId').value.trim();
            const limit = parseInt(document.getElementById('browseLimit').value);

            try {
                const response = await fetch(`${API_BASE}/v1/browse`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-User-Id': userId
                    },
                    body: JSON.stringify({
                        limit,
                        offset: 0
                    })
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.message || 'Browse failed');
                }

                const result = await response.json();
                displayBrowseResults(result);
                showMessage(`Loaded ${result.total} document(s) in ${result.took_ms}ms`, 'success');
            } catch (error) {
                showMessage(`Error: ${error.message}`, 'error');
            }
        });

        // Delete Document
        document.getElementById('deleteForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const userId = document.getElementById('deleteUserId').value.trim();
            const docId = document.getElementById('deleteDocId').value.trim();

            if (!confirm(`Are you sure you want to delete document ${docId}?`)) {
                return;
            }

            try {
                const response = await fetch(`${API_BASE}/v1/documents`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-User-Id': userId
                    },
                    body: JSON.stringify({ id: docId })
                });

                if (!response.ok) {
                    const error = await response.json();
                    throw new Error(error.message || 'Failed to delete document');
                }

                showMessage('Document deleted successfully!', 'success');
                document.getElementById('deleteDocId').value = '';
            } catch (error) {
                showMessage(`Error: ${error.message}`, 'error');
            }
        });

        function displaySearchResults(result) {
            const resultsEl = document.getElementById('results');

            if (!result.results || result.results.length === 0) {
                resultsEl.innerHTML = '<div class="no-results">No documents found matching your query.</div>';
                return;
            }

            resultsEl.innerHTML = result.results.map(doc => `
                <div class="result-item">
                    <div class="result-title">${escapeHtml(doc.title)}</div>
                    <div class="result-body" style="white-space: pre-wrap;">${escapeHtml(doc.body)}</div>
                    <div class="result-meta">
                        <span class="result-score">Score: ${doc.score.toFixed(2)}</span>
                        <span>ID: ${escapeHtml(doc.id)}</span>
                        ${doc.created_at ? `<span>Created: ${new Date(doc.created_at).toLocaleString()}</span>` : ''}
                    </div>
                </div>
            `).join('');
        }

        function displayBrowseResults(result) {
            const resultsEl = document.getElementById('results');

            if (!result.documents || result.documents.length === 0) {
                resultsEl.innerHTML = '<div class="no-results">No documents found in this index.</div>';
                return;
            }

            resultsEl.innerHTML = result.documents.map(doc => `
                <div class="result-item">
                    <div class="result-title">${escapeHtml(doc.title)}</div>
                    <div class="result-body" style="white-space: pre-wrap;">${escapeHtml(doc.body)}</div>
                    <div class="result-meta">
                        <span>ID: ${escapeHtml(doc.id)}</span>
                        ${doc.created_at ? `<span>Created: ${new Date(doc.created_at).toLocaleString()}</span>` : ''}
                        ${doc.tags && doc.tags.length > 0 ? `<span>Tags: ${doc.tags.map(t => escapeHtml(t)).join(', ')}</span>` : ''}
                    </div>
                </div>
            `).join('');
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Sync user ID fields
        document.getElementById('userId').addEventListener('input', (e) => {
            document.getElementById('searchUserId').value = e.target.value;
            document.getElementById('browseUserId').value = e.target.value;
            document.getElementById('deleteUserId').value = e.target.value;
        });
    </script>
</body>
</html>
"#;
