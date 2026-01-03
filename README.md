# Local Search Engine

A fast, local search engine built in Rust with vector embeddings and SQLite storage.

## Features

- 🔍 Semantic search using vector embeddings
- 📁 Local file indexing and search
- 🗄️ SQLite-based storage
- 📚 Both library and CLI interfaces
- ⚡ Fast and efficient

## Installation

### As a CLI tool

```bash
cargo install local_search --features cli
```

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
local_search = "0.1.0"
```

## CLI Usage

```bash
# Index documents
local_search index /path/to/documents

# Search for content
local_search search "your query here"
```

## Library Usage

```rust
use local_search::{SqliteLocalSearchEngine, LocalEmbedder, DocumentIndexer, LocalSearch};

// Create embedder and search engine
let embedder = LocalEmbedder::new()?;
let mut engine = SqliteLocalSearchEngine::new("search.db", embedder)?;

// Index a document
engine.index_document("example.txt", "This is example content")?;

// Search
let results = engine.search("example", 5)?;
```

## Development

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/local_search.git
cd local_search

# Run tests
cargo test

# Run CLI with features
cargo run --features cli -- search "query"
```

## License

MIT License - see [LICENSE](LICENSE) file for details.