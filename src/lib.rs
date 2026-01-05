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
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use localsearch::{SqliteLocalSearchEngine, LocalEmbedder, DocumentIndexer, LocalSearch, SearchType, DocumentRequest};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Create embedder and search engine
//! let embedder = LocalEmbedder::new_with_default_model()?;
//! let mut engine = SqliteLocalSearchEngine::new("search.db", Some(embedder))?;
//!
//! // Index a document
//! engine.insert_document(DocumentRequest {
//!     path: "some/unique/path".to_string(),
//!     content: "This is example content".to_string(),
//!     metadata: None,
//! })?;
//!
//! // Search
//! let results = engine.search("example", SearchType::Hybrid, Some(10))?;
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub use traits::{DocumentIndexer, DocumentRequest, LocalSearch, SearchResult, SearchType};

pub mod embed;
pub use embed::LocalEmbedder;

pub mod engines;
pub use engines::SqliteLocalSearchEngine;
