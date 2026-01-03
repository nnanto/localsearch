
pub mod traits;
pub use traits::{DocumentIndexer, LocalSearch, DocumentRequest, SearchResult, SearchType};


pub mod embed;
pub use embed::LocalEmbedder;

pub mod engines;
pub use engines::SqliteLocalSearchEngine;