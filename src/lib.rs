pub mod search;
pub use search::{SqliteLocalSearch, SearchResult, SearchType};
pub mod embed;
pub use embed::LocalEmbedder;