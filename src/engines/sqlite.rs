use crate::traits::{DocumentIndexer, DocumentRequest, LocalSearch, SearchType};
use crate::{LocalEmbedder, traits::SearchResult};
use anyhow::anyhow;
use log::{debug, info};
use rusqlite::Connection;

pub struct SqliteLocalSearchEngine {
    db_path: String,
    conn: Connection,
    embedder: Option<LocalEmbedder>,
}

impl SqliteLocalSearchEngine {
    /// Creates a new SQLite-based search engine instance with the specified database path and embedder
    pub fn new(db_path: &str, embedder: Option<LocalEmbedder>) -> anyhow::Result<Self> {
        info!("Creating new SqliteLocalSearch for path: {}", db_path);
        let conn =
            Connection::open(db_path).map_err(|e| anyhow!("Failed to open database: {}", e))?;
        let lfts = SqliteLocalSearchEngine {
            db_path: db_path.to_string(),
            conn,
            embedder,
        };
        info!("SqliteLocalSearch initialization complete: {}", db_path);
        Ok(lfts)
    }

    /// Creates the required database tables for documents, FTS index, and embeddings.
    pub fn create_table(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                    path TEXT PRIMARY KEY,
                    content TEXT NOT NULL,
                    metadata TEXT NOT NULL,
                    createdAt REAL NOT NULL,
                    updatedAt REAL NOT NULL
                )",
            [],
        )?;
        debug!("Created documents table if it did not exist.");

        self.conn
            .execute("DROP TABLE IF EXISTS documents_fts", [])?;

        debug!("Dropped existing documents_fts table if it existed.");
        self.conn.execute(
            "CREATE VIRTUAL TABLE documents_fts USING fts5(
                path UNINDEXED,
                content
            )",
            [],
        )?;
        debug!("Created documents_fts FTS5 virtual table.");

        // Create embeddings table only if embedder is available
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS document_embeddings (
                path TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                FOREIGN KEY(path) REFERENCES documents(path)
            )",
            [],
        )?;
        debug!("Created document_embeddings table if it did not exist.");

        // let schema: String = self.conn.query_one("SELECT sql FROM sqlite_main WHERE type='table' AND name='documents'", [], |row| row.get(0))?;
        // debug!("Documents table schema: {}", schema);
        // // Check if FTS table was created
        // let fts_exists: i32 = self.conn.query_row(
        //     "SELECT COUNT(*) FROM sqlite_main WHERE type='table' AND name='documents_fts'",
        //     [],
        //     |row| row.get(0),
        // )?;
        // info!("FTS table exists: {}", fts_exists > 0);
        Ok(())
    }

    fn search_semantic_only(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let embedder = self
            .embedder
            .as_ref()
            .ok_or_else(|| anyhow!("Semantic search requires an embedder"))?;
        let query_embedding = embedder.embed_text(query)?;
        let semantic_results = self.search_by_embedding(&query_embedding)?;
        let results = semantic_results
            .into_iter()
            .map(|r| SearchResult {
                path: r.path,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
                fts_score: None,
                semantic_score: Some(r.semantic_score.unwrap_or(0.0)),
                final_score: r.final_score,
            })
            .collect();
        Ok(results)
    }

    fn search_hybrid(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        // If no embedder, fallback to FTS-only search
        if self.embedder.is_none() {
            debug!("No embedder available for hybrid search, falling back to FTS-only");
            return self.search_fulltext_only(query);
        }

        // Get FTS results
        let fts_results = self.search_fts(query).unwrap_or_default();

        // Get semantic results
        let query_embedding = self.embedder.as_ref().unwrap().embed_text(query)?;
        let semantic_results = self
            .search_by_embedding(&query_embedding)
            .unwrap_or_default();

        // Combine and normalize scores
        let mut combined_results = std::collections::HashMap::new();

        // Normalize FTS scores (convert to 0-1 range)
        let max_fts_score = fts_results
            .iter()
            .map(|r| r.fts_score.unwrap_or(0.0))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        for result in fts_results {
            let current_score = result.fts_score.unwrap_or(0.0);
            let normalized_score = current_score
                / (if max_fts_score.abs() < 1e-5 {
                    1.0
                } else {
                    max_fts_score
                });
            combined_results.insert(result.path.clone(), (result, Some(normalized_score), None));
        }

        // Semantic scores are already normalized (cosine similarity 0-1)
        for result in semantic_results {
            let result_score = result.semantic_score.unwrap_or(0.0); // Extract score before move
            match combined_results.get_mut(&result.path) {
                Some((_, _fts_score, semantic_score)) => {
                    *semantic_score = Some(result_score);
                }
                None => {
                    combined_results
                        .insert(result.path.clone(), (result, None, Some(result_score)));
                }
            }
        }

        // Calculate hybrid scores
        let mut final_results: Vec<SearchResult> = combined_results
            .into_iter()
            .map(|(_, (base_result, fts_score, semantic_score))| {
                let fts_component = fts_score.unwrap_or(0.0) * 0.6;
                let semantic_component = semantic_score.unwrap_or(0.0) * 0.4;
                let final_score = fts_component + semantic_component;

                SearchResult {
                    path: base_result.path,
                    metadata: base_result.metadata.clone(),
                    created_at: base_result.created_at,
                    updated_at: base_result.updated_at,
                    fts_score,
                    semantic_score,
                    final_score,
                }
            })
            .collect();

        // Sort by final score descending
        final_results.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!(
            "Hybrid search for query '{}' returned {} results.",
            query,
            final_results.len()
        );
        Ok(final_results)
    }

    fn search_by_embedding(&self, query_embedding: &[f32]) -> anyhow::Result<Vec<SearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT d.path, d.metadata, d.createdAt, d.updatedAt, e.embedding
             FROM documents d 
             JOIN document_embeddings e ON d.path = e.path",
            )
            .map_err(|e| anyhow!("Failed to prepare semantic search query: {}", e))?;

        let embedding_iter = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let metadata_str: String = row.get(1)?;
                let metadata: Option<std::collections::HashMap<String, String>> =
                    serde_json::from_str(&metadata_str).ok();
                let created_at: f64 = row.get(2)?;
                let updated_at: f64 = row.get(3)?;
                let embedding_bytes: Vec<u8> = row.get(4)?;
                Ok((path, metadata, created_at, updated_at, embedding_bytes))
            })
            .map_err(|e| anyhow!("Failed to query embeddings: {}", e))?;

        let mut results = Vec::new();
        for result in embedding_iter {
            let (path, metadata, created_at, updated_at, embedding_bytes) =
                result.map_err(|e| anyhow!("Failed to read embedding row: {}", e))?;

            // Convert bytes back to f32 vector
            let embedding: Vec<f32> = embedding_bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            // Calculate cosine similarity
            let similarity = self.cosine_similarity(query_embedding, &embedding);

            results.push(SearchResult {
                path,
                metadata,
                created_at,
                updated_at,
                fts_score: None,
                semantic_score: Some(similarity),
                final_score: similarity,
            });
        }

        // Sort by similarity score descending
        results.sort_by(|a, b| {
            b.semantic_score
                .unwrap_or(0.0)
                .partial_cmp(&a.semantic_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!("Semantic search returned {} results.", results.len());
        Ok(results)
    }

    fn search_fulltext_only(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let fts_results = self.search_fts(query)?;
        info!(
            "Full-text search for query '{}' returned {} results.",
            query,
            fts_results.len()
        );
        let results = fts_results
            .into_iter()
            .map(|r| SearchResult {
                path: r.path,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
                fts_score: Some(r.fts_score.unwrap_or(0.0)),
                semantic_score: None,
                final_score: r.final_score,
            })
            .collect();
        Ok(results)
    }

    fn search_fts(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.path, d.metadata, d.createdAt, d.updatedAt, bm25(documents_fts) * -1 as score
             FROM documents_fts 
             JOIN documents d ON documents_fts.path = d.path
             WHERE documents_fts MATCH ?1
             ORDER BY rank",
        )?;
        let search_iter = stmt.query_map(rusqlite::params![query], |row| {
            let score: f64 = row.get(4)?;
            Ok(SearchResult {
                path: row.get(0)?,
                metadata: serde_json::from_str(&row.get::<_, String>(1)?).ok(),
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                fts_score: Some(score),
                semantic_score: None,
                final_score: score,
            })
        })?;

        let mut results = Vec::new();
        for result in search_iter {
            results.push(result?);
        }
        debug!(
            "Search for query '{}' returned {} results.",
            query,
            results.len()
        );
        Ok(results)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        // For normalized embeddings, cosine similarity is just the dot product
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        dot_product as f64
    }
}

impl DocumentIndexer for SqliteLocalSearchEngine {
    /// Inserts a new document into the database with FTS and embedding support.
    fn insert_document(&self, request: DocumentRequest) -> anyhow::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let metadata_str = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
        let created_at = now;
        let updated_at = now;

        let rows_affected = self.conn.execute("INSERT INTO documents (path, content, metadata, createdAt, updatedAt) values (?1, ?2, ?3, ?4, ?5)", rusqlite::params![request.path, request.content, metadata_str, created_at, updated_at])
            .map_err(|e| anyhow!("Failed to insert document: {}", e))?;
        debug!(
            "Inserted document with path: {}. Number of rows affected: {}",
            request.path, rows_affected
        );

        // Generate and store embedding if embedder is available
        if let Some(ref embedder) = self.embedder {
            let embedding = embedder.embed_text(&request.content)?;
            let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
            self.conn
                .execute(
                    "INSERT INTO document_embeddings (path, embedding) VALUES (?1, ?2)",
                    rusqlite::params![request.path, embedding_bytes],
                )
                .map_err(|e| anyhow!("Failed to insert embedding: {}", e))?;
            debug!(
                "Inserted embedding for document with path: {}",
                request.path
            );
        }

        // Insert into FTS table for search
        self.conn
            .execute(
                "INSERT INTO documents_fts (path, content) VALUES (?1, ?2)",
                rusqlite::params![request.path, request.content],
            )
            .map_err(|e| anyhow!("Failed to insert into FTS: {}", e))?;
        debug!(
            "Inserted document into FTS table with path: {}",
            request.path
        );
        Ok(())
    }

    /// Updates an existing document or inserts a new one if it doesn't exist.
    fn upsert_document(&self, request: DocumentRequest) -> anyhow::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let metadata_str = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
        let updated_at = now;

        let rows_affected = self
            .conn
            .execute(
                "UPDATE documents SET content = ?1, metadata = ?2, updatedAt = ?3 WHERE path = ?4",
                rusqlite::params![request.content, metadata_str, updated_at, request.path],
            )
            .map_err(|e| anyhow!("Failed to update document: {}", e))?;

        if rows_affected == 0 {
            // Document does not exist, insert new
            debug!(
                "Document with path: {} did not exist. Inserting new document.",
                request.path
            );
            self.insert_document(request)?;
        } else {
            debug!(
                "Updated document with path: {}. Number of rows affected: {}",
                request.path, rows_affected
            );

            // Update embedding if embedder is available
            if let Some(ref embedder) = self.embedder {
                let embedding = embedder.embed_text(&request.content)?;
                let embedding_bytes: Vec<u8> =
                    embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
                self.conn
                    .execute(
                        "UPDATE document_embeddings SET embedding = ?1 WHERE path = ?2",
                        rusqlite::params![embedding_bytes, request.path],
                    )
                    .map_err(|e| anyhow!("Failed to update embedding: {}", e))?;
                debug!("Updated embedding for document with path: {}", request.path);
            }

            // Update FTS table
            self.conn
                .execute(
                    "UPDATE documents_fts SET content = ?1 WHERE path = ?2",
                    rusqlite::params![request.content, request.path],
                )
                .map_err(|e| anyhow!("Failed to update FTS: {}", e))?;
            debug!("Updated FTS entry for document with path: {}", request.path);
        }
        Ok(())
    }

    /// Removes a document and its associated embeddings and FTS entries by path.
    fn delete_document(&self, path: &str) -> anyhow::Result<()> {
        // Delete from child tables first to avoid foreign key constraint violations
        if self.embedder.is_some() {
            self.conn
                .execute(
                    "DELETE FROM document_embeddings WHERE path = ?1",
                    rusqlite::params![path],
                )
                .map_err(|e| anyhow!("Failed to delete embedding: {}", e))?;
            debug!("Deleted embedding for document with path: {}", path);
        }

        self.conn
            .execute(
                "DELETE FROM documents_fts WHERE path = ?1",
                rusqlite::params![path],
            )
            .map_err(|e| anyhow!("Failed to delete from FTS: {}", e))?;
        debug!("Deleted FTS entry for document with path: {}", path);

        let rows_affected = self
            .conn
            .execute(
                "DELETE FROM documents WHERE path = ?1",
                rusqlite::params![path],
            )
            .map_err(|e| anyhow!("Failed to delete document: {}", e))?;
        debug!(
            "Deleted document with path: {}. Number of rows affected: {}",
            path, rows_affected
        );

        Ok(())
    }

    /// Refreshes the database connection to pick up external changes.
    fn refresh(&mut self) -> anyhow::Result<()> {
        // Close and reopen the connection to refresh from underlying database changes
        let db_path = self.db_path.clone();
        let new_conn =
            Connection::open(&db_path).map_err(|e| anyhow!("Failed to reopen database: {}", e))?;
        let old_conn = std::mem::replace(&mut self.conn, new_conn);
        old_conn
            .close()
            .map_err(|e| anyhow!("Failed to close database connection: {}", e.1))?;
        info!("Database connection refreshed for path: {:?}", self.db_path);
        Ok(())
    }

    /// Returns the total number of documents currently indexed in the database.
    fn stats(&self) -> anyhow::Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        info!("Total documents indexed: {}", count);
        Ok(count)
    }
}

impl LocalSearch for SqliteLocalSearchEngine {
    /// Performs a search using the specified search type (FullText, Semantic, or Hybrid).
    fn search(
        &self,
        query: &str,
        search_type: SearchType,
        top: Option<i8>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let res = match search_type {
            SearchType::FullText => self.search_fulltext_only(query),
            SearchType::Semantic => {
                if self.embedder.is_none() {
                    return Err(anyhow!("Semantic search requires an embedder"));
                }
                self.search_semantic_only(query)
            }
            SearchType::Hybrid => self.search_hybrid(query),
        }?;
        let limit = std::cmp::min(top.unwrap_or(10) as usize, res.len());
        Ok(res.into_iter().take(limit).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_engine() -> (SqliteLocalSearchEngine, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let engine = SqliteLocalSearchEngine::new(db_path.to_str().unwrap(), None)
            .expect("Failed to create test engine");
        engine.create_table().expect("Failed to create tables");
        (engine, temp_dir)
    }

    fn create_test_engine_with_embedder() -> (SqliteLocalSearchEngine, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let embedder = LocalEmbedder::new_with_default_model().expect("Failed to create embedder");
        let engine = SqliteLocalSearchEngine::new(db_path.to_str().unwrap(), Some(embedder))
            .expect("Failed to create test engine");
        engine.create_table().expect("Failed to create tables");
        (engine, temp_dir)
    }

    fn create_test_document(path: &str, content: &str) -> DocumentRequest {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), format!("Test Document {}", path));
        metadata.insert("type".to_string(), "test".to_string());

        DocumentRequest {
            path: path.to_string(),
            content: content.to_string(),
            metadata: Some(metadata),
        }
    }

    #[test]

    fn test_engine_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let engine = SqliteLocalSearchEngine::new(db_path.to_str().unwrap(), None);
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        let result = engine.create_table();
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_insertion() {
        let (engine, _temp_dir) = create_test_engine();

        let doc = create_test_document(
            "test1.txt",
            "This is a test document about Rust programming.",
        );
        let result = engine.insert_document(doc);
        assert!(result.is_ok());

        let count = engine.stats().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_document_upsert() {
        let (engine, _temp_dir) = create_test_engine();

        // Insert initial document
        let doc1 = create_test_document("test1.txt", "Original content");
        engine.insert_document(doc1).unwrap();

        // Upsert with new content
        let doc2 = create_test_document("test1.txt", "Updated content about machine learning");
        let result = engine.upsert_document(doc2);
        assert!(result.is_ok());

        // Should still have only 1 document
        let count = engine.stats().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_document_deletion() {
        let (engine, _temp_dir) = create_test_engine();

        // Insert a document
        let doc = create_test_document("test1.txt", "This document will be deleted");
        engine.insert_document(doc).unwrap();
        assert_eq!(engine.stats().unwrap(), 1);

        // Delete the document
        let result = engine.delete_document("test1.txt");
        assert!(result.is_ok());

        // Should have 0 documents now
        let count = engine.stats().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fulltext_search() {
        let (engine, _temp_dir) = create_test_engine();

        // Insert test documents
        let docs = vec![
            create_test_document("rust1.txt", "Rust programming language is memory safe"),
            create_test_document("python1.txt", "Python is a high-level programming language"),
            create_test_document("js1.txt", "JavaScript is used for web development"),
        ];

        for doc in docs {
            engine.insert_document(doc).unwrap();
        }

        // Search for "programming"
        let results = engine
            .search("programming", SearchType::FullText, Some(10))
            .unwrap();
        assert_eq!(results.len(), 2); // Should match rust1.txt and python1.txt

        // All results should have FTS scores but no semantic scores
        for result in &results {
            assert!(result.fts_score.is_some());
            assert!(result.semantic_score.is_none());
        }
    }

    #[test]

    fn test_semantic_search() {
        let (engine, _temp_dir) = create_test_engine_with_embedder();

        // Insert test documents with different but semantically related content
        let docs = vec![
            create_test_document("car1.txt", "Automobiles are vehicles for transportation"),
            create_test_document("car2.txt", "Cars help people travel from place to place"),
            create_test_document("cooking1.txt", "Recipes help you prepare delicious meals"),
        ];

        for doc in docs {
            engine.insert_document(doc).unwrap();
        }

        // Search for "vehicle" (semantically related to car content)
        let results = engine
            .search("vehicle transportation", SearchType::Semantic, Some(10))
            .unwrap();
        assert!(!results.is_empty());

        // All results should have semantic scores but no FTS scores
        for result in &results {
            assert!(result.fts_score.is_none());
            assert!(result.semantic_score.is_some());
        }

        // First result should be most semantically similar
        assert!(results[0].semantic_score.unwrap() > 0.0);
    }

    #[test]

    fn test_hybrid_search() {
        let (engine, _temp_dir) = create_test_engine_with_embedder();

        // Insert test documents
        let docs = vec![
            create_test_document("tech1.txt", "Rust programming language memory safety"),
            create_test_document(
                "tech2.txt",
                "Programming languages help developers build software",
            ),
            create_test_document("other1.txt", "Cooking recipes for dinner tonight"),
        ];

        for doc in docs {
            engine.insert_document(doc).unwrap();
        }

        // Hybrid search combining keyword and semantic matching
        let results = engine
            .search("programming", SearchType::Hybrid, Some(10))
            .unwrap();
        assert!(!results.is_empty());
        println!("Hybrid search results:");
        for result in &results {
            println!(
                "Path: {}, Final Score: {}, FTS Score: {:?}, Semantic Score: {:?}",
                result.path, result.final_score, result.fts_score, result.semantic_score
            );
        }

        // Results should have both scores for documents that match both ways
        let mut found_both_scores = false;
        for result in &results {
            if result.fts_score.is_some() && result.semantic_score.is_some() {
                found_both_scores = true;
            }
            assert!(result.final_score > 0.0);
        }
        assert!(
            found_both_scores,
            "Should have at least one result with both FTS and semantic scores"
        );
    }

    #[test]
    fn test_cosine_similarity() {
        let (engine, _temp_dir) = create_test_engine();

        // Test identical vectors
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];
        let similarity = engine.cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);

        // Test orthogonal vectors
        let vec3 = vec![1.0, 0.0, 0.0];
        let vec4 = vec![0.0, 1.0, 0.0];
        let similarity = engine.cosine_similarity(&vec3, &vec4);
        assert!((similarity - 0.0).abs() < 0.001);

        // Test different length vectors
        let vec5 = vec![1.0, 0.0];
        let vec6 = vec![1.0, 0.0, 0.0];
        let similarity = engine.cosine_similarity(&vec5, &vec6);
        assert_eq!(similarity, 0.0);
    }

    #[test]

    fn test_refresh_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut engine = SqliteLocalSearchEngine::new(db_path.to_str().unwrap(), None).unwrap();
        // Create first database with one document
        {
            // let engine = SqliteLocalSearchEngine::new(db_path.to_str().unwrap()).unwrap();
            engine.create_table().unwrap();
            let doc = create_test_document("test1.txt", "Test content");
            engine.insert_document(doc).unwrap();
            assert_eq!(engine.stats().unwrap(), 1);
        } // engine goes out of scope, connection closed

        // Create new database file with different content
        {
            let temp_db_path = temp_dir.path().join("temp_test.db");
            let new_engine =
                SqliteLocalSearchEngine::new(temp_db_path.to_str().unwrap(), None).unwrap();
            new_engine.create_table().unwrap();
            let doc1 = create_test_document("test2.txt", "Different content");
            let doc2 = create_test_document("test3.txt", "More different content");
            new_engine.insert_document(doc1).unwrap();
            new_engine.insert_document(doc2).unwrap();
            assert_eq!(new_engine.stats().unwrap(), 2);
            // Move new database file to original path
            std::fs::rename(temp_db_path, db_path).unwrap();
        } // new_engine goes out of scope

        let count_before = engine.stats().unwrap();
        assert_eq!(count_before, 1); // Should see the 2 documents from new database

        // Refresh connection
        let result = engine.refresh();
        assert!(result.is_ok());

        // Should still see the same data after refresh
        let count_after = engine.stats().unwrap();
        assert_eq!(count_after, 2);

        // Verify specific documents exist
        let results = engine
            .search("Different", SearchType::FullText, Some(10))
            .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_stats_empty_database() {
        let (engine, _temp_dir) = create_test_engine();

        let count = engine.stats().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_search_no_results() {
        let (engine, _temp_dir) = create_test_engine();

        // Search empty database - FTS should work without embedder
        let results = engine
            .search("nonexistent query", SearchType::FullText, Some(10))
            .unwrap();
        assert!(results.is_empty());

        // Semantic search should fail without embedder
        let semantic_result = engine.search("nonexistent query", SearchType::Semantic, Some(10));
        assert!(semantic_result.is_err());

        // Hybrid should fallback to FTS without embedder
        let results = engine
            .search("nonexistent query", SearchType::Hybrid, Some(10))
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]

    fn test_no_search_result_embedder() {
        // Test with embedder engine
        let (engine_with_embedder, _temp_dir2) = create_test_engine_with_embedder();

        let results = engine_with_embedder
            .search("nonexistent query", SearchType::Semantic, Some(10))
            .unwrap();
        assert!(results.is_empty());

        let results = engine_with_embedder
            .search("nonexistent query", SearchType::Hybrid, Some(10))
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_duplicate_insertion_fails() {
        let (engine, _temp_dir) = create_test_engine();

        let doc1 = create_test_document("test1.txt", "First content");
        let doc2 = create_test_document("test1.txt", "Second content");

        // First insertion should succeed
        let result1 = engine.insert_document(doc1);
        assert!(result1.is_ok());

        // Second insertion with same path should fail
        let result2 = engine.insert_document(doc2);
        assert!(result2.is_err());
    }

    #[test]
    fn test_delete_nonexistent_document() {
        let (engine, _temp_dir) = create_test_engine();

        // Deleting non-existent document should not error
        let result = engine.delete_document("nonexistent.txt");
        assert!(result.is_ok());

        let count = engine.stats().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_upsert_new_document() {
        let (engine, _temp_dir) = create_test_engine();

        // Upsert on empty database should insert new document
        let doc = create_test_document("new.txt", "New document content");
        let result = engine.upsert_document(doc);
        assert!(result.is_ok());

        let count = engine.stats().unwrap();
        assert_eq!(count, 1);
    }
}
