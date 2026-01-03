
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
//! use local_search::{SqliteLocalSearchEngine, LocalEmbedder, DocumentIndexer, LocalSearch};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Create embedder and search engine
//! let embedder = LocalEmbedder::new()?;
//! let mut engine = SqliteLocalSearchEngine::new("search.db", embedder)?;
//! 
//! // Index a document
//! engine.index_document("example.txt", "This is example content")?;
//! 
//! // Search
//! let results = engine.search("example", 5)?;
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub use traits::{DocumentIndexer, LocalSearch, DocumentRequest, SearchResult, SearchType};


pub mod embed;
pub use embed::LocalEmbedder;

pub mod engines;
pub use engines::SqliteLocalSearchEngine;