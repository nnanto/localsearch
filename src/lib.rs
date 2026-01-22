//! # Local Search Engine
//!
//! A fast, local search engine built in Rust with vector embeddings and SQLite storage.
//!
//! ## Features
//!
//! - Semantic search using vector embeddings
//! - Local file indexing and search
//! - SQLite-based storage
//! - Both library and CLI interfaces
//! - Configurable cache and database directories using system directories
//! - Support for custom local ONNX models and tokenizers
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use localsearch::{SqliteLocalSearchEngine, LocalEmbedder, DocumentIndexer, LocalSearch, SearchType, DocumentRequest, LocalSearchDirs};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Get default directories
//! let dirs = LocalSearchDirs::new();
//! let db_path = dirs.default_db_path();
//!
//! // Create embedder (uses default cache directory)
//! let embedder = LocalEmbedder::new_with_default_model()?;
//!
//! // Or create embedder with custom cache directory
//! // let cache_dir = dirs.ensure_cache_dir()?;
//! // let embedder = LocalEmbedder::new_with_cache_dir(cache_dir)?;
//!
//! // Or use your own local ONNX model
//! // let onnx_path = std::path::PathBuf::from("/path/to/model.onnx");
//! // let tokenizer_dir = std::path::PathBuf::from("/path/to/tokenizer");
//! // let embedder = LocalEmbedder::new_with_local_model(onnx_path, tokenizer_dir, Some(512))?;
//!
//! let mut engine = SqliteLocalSearchEngine::new(&db_path.to_string_lossy(), Some(embedder))?;
//!
//! // Index a document
//! engine.insert_document(DocumentRequest {
//!     path: "some/unique/path".to_string(),
//!     content: "This is example content".to_string(),
//!     metadata: None,
//! })?;
//!
//! // Search
//! let results = engine.search("example", SearchType::Hybrid, Some(10), None)?;
//!
//! // Search with path filters (multiple patterns supported)
//! let filters = vec!["src".to_string(), "test".to_string()];
//! let filtered_results = engine.search("example", SearchType::Hybrid, Some(10), Some(&filters))?;
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub use traits::{DocumentIndexer, DocumentRequest, LocalSearch, SearchResult, SearchType};

pub mod config;
pub use config::LocalSearchDirs;

pub mod embed;
pub use embed::LocalEmbedder;

pub mod engines;
pub use engines::SqliteLocalSearchEngine;
