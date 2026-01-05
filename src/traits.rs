use serde::{Deserialize, Serialize};

/// Search strategy for querying documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    FullText,
    Semantic,
    Hybrid,
}

/// Result from a search operation with scores and metadata.
#[derive(Debug)]
pub struct SearchResult {
    pub path: String,
    pub metadata: Option<std::collections::HashMap<String, String>>,
    pub created_at: f64,
    pub updated_at: f64,
    pub fts_score: Option<f64>,
    pub semantic_score: Option<f64>,
    pub final_score: f64,
}

/// Request to index a document with content and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub path: String,
    pub content: String,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Trait for managing documents in a search index.
pub trait DocumentIndexer {
    fn insert_document(&self, request: DocumentRequest) -> anyhow::Result<()>;
    fn upsert_document(&self, request: DocumentRequest) -> anyhow::Result<()>;
    fn delete_document(&self, path: &str) -> anyhow::Result<()>;
    fn stats(&self) -> anyhow::Result<i64>;
    fn refresh(&mut self) -> anyhow::Result<()>;
}

/// Trait for performing searches on indexed documents.
pub trait LocalSearch {
    fn search(
        &self,
        query: &str,
        search_type: SearchType,
        top: Option<i8>,
    ) -> anyhow::Result<Vec<SearchResult>>;
}
