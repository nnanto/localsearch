# Local Search Engine

A fast, local search engine built in Rust with vector embeddings and SQLite storage.

## Features

- 🔍 Full-Text + Semantic Search using embeddings generated and stored locally
- 📁 Local file indexing and search
- 🗄️ SQLite-based storage
- 📚 Both library and CLI interfaces

# Installation Guide

## Quick Install

### Linux/macOS
```bash
curl -sSL https://raw.githubusercontent.com/nnanto/localsearch/main/scripts/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/nnanto/localsearch/main/scripts/install.ps1 | iex
```

## Manual Installation

### Pre-built Binaries

Download the appropriate binary for your platform from the [latest release](https://github.com/nnanto/localsearch/releases/latest):

#### Linux (x86_64)
```bash
curl -L https://github.com/nnanto/localsearch/releases/latest/download/localsearch-linux-x86_64.tar.gz | tar xz
sudo mv localsearch /usr/local/bin/
```

#### macOS (Intel)
```bash
curl -L https://github.com/nnanto/localsearch/releases/latest/download/localsearch-macos-x86_64.tar.gz | tar xz
sudo mv localsearch /usr/local/bin/
```

#### macOS (Apple Silicon)
```bash
curl -L https://github.com/nnanto/localsearch/releases/latest/download/localsearch-macos-aarch64.tar.gz | tar xz
sudo mv localsearch /usr/local/bin/
```

#### Windows
1. Download [localsearch-windows-x86_64.zip](https://github.com/nnanto/localsearch/releases/latest/download/localsearch-windows-x86_64.zip)
2. Extract the ZIP file
3. Add the extracted directory to your PATH environment variable

### From Source

If you have Rust installed, you can build from source:

```bash
cargo install --git https://github.com/nnanto/localsearch --features cli
```

Or clone and build:

```bash
git clone https://github.com/nnanto/localsearch.git
cd localsearch
cargo build --release --features cli
sudo cp target/release/localsearch /usr/local/bin/
```

## Verify Installation

After installation, verify that the tool is working:

```bash
localsearch --help
```

You should see the help output for the localsearch CLI tool.

## Updating

To update to the latest version, simply re-run the installation command. The installer will replace the existing binary with the latest version.

## Uninstallation

### Linux/macOS
```bash
sudo rm /usr/local/bin/localsearch
```

### Windows
Remove the installation directory and update your PATH environment variable to remove the localsearch directory.

## Troubleshooting

### Permission Issues
If you get permission errors on Linux/macOS, make sure you're running the installation with appropriate permissions (using `sudo` when needed).

### Path Issues
If the `localsearch` command is not found after installation, make sure the installation directory is in your PATH:

- **Linux/macOS**: `/usr/local/bin` should be in your PATH
- **Windows**: The installation directory should be added to your PATH environment variable

### Antivirus False Positives
Some antivirus software may flag the binary as suspicious. This is a common issue with Rust binaries. You may need to add an exception for the localsearch binary.

# Usage

## CLI Usage

### Basic Commands

```bash
# Index documents (uses system default directories)
localsearch index /path/to/documents

# Search for content
localsearch search "your query here"
```

### Directory Configuration

By default, `localsearch` uses system-appropriate directories:
- **Cache**: Model files are stored in the system cache directory (e.g., `~/.cache` on Linux, `~/Library/Caches` on macOS)
- **Database**: SQLite database is stored in the application data directory (e.g., `~/.local/share` on Linux, `~/Library/Application Support` on macOS)

You can override these defaults:

```bash
# Use custom database location
localsearch index /path/to/documents --db /custom/path/to/database.db

# Use custom cache directory for embeddings
localsearch index /path/to/documents --cache-dir /custom/cache/path

# Use both custom paths
localsearch index /path/to/documents --db /custom/db.db --cache-dir /custom/cache

# Search with custom paths
localsearch search "query" --db /custom/db.db --cache-dir /custom/cache
```

### File Types

```bash
# Index JSON files (default)
localsearch index data.json --file-type json

# Index text files
localsearch index /path/to/text/files --file-type text
```

### Search Options

```bash
# Different search types
localsearch search "query" --search-type semantic
localsearch search "query" --search-type fulltext  
localsearch search "query" --search-type hybrid    # default

# Limit results
localsearch search "query" --limit 5

# Pretty output
localsearch search "query" --pretty
```

## Library Usage

```rust
use localsearch::{SqliteLocalSearchEngine, LocalEmbedder, DocumentIndexer, LocalSearch, SearchType, DocumentRequest, LocalSearchDirs};

fn main() -> anyhow::Result<()> {
    // Option 1: Use default system directories
    let dirs = LocalSearchDirs::new();
    let db_path = dirs.default_db_path();
    let embedder = LocalEmbedder::new_with_default_model()?;
    
    // Option 2: Use custom cache directory
    // let custom_cache = std::path::PathBuf::from("/custom/cache");
    // let embedder = LocalEmbedder::new_with_cache_dir(custom_cache)?;
    
    // Option 3: Use your own local ONNX model and tokenizer
    // let onnx_path = std::path::PathBuf::from("/path/to/your/model.onnx");
    // let tokenizer_dir = std::path::PathBuf::from("/path/to/tokenizer/files");
    // let embedder = LocalEmbedder::new_with_local_model(onnx_path, tokenizer_dir, Some(512))?;
    
    let mut engine = SqliteLocalSearchEngine::new(&db_path.to_string_lossy(), Some(embedder))?;

    // Index a document
    engine.insert_document(DocumentRequest {
        path: "some/unique/path".to_string(),
        content: "This is example content".to_string(),
        metadata: None,
    })?;

    // Search
    let results = engine.search("example", SearchType::Hybrid, Some(10))?;
    Ok(())
}
```

### Using Local ONNX Models

You can now use your own local ONNX embedding models instead of the default pre-built models:

```rust
use localsearch::LocalEmbedder;
use std::path::PathBuf;

// Method 1: Using a tokenizer directory
// Your tokenizer directory should contain:
// - tokenizer.json
// - config.json
// - special_tokens_map.json
// - tokenizer_config.json
let onnx_path = PathBuf::from("/path/to/your/model.onnx");
let tokenizer_dir = PathBuf::from("/path/to/tokenizer/directory");
let embedder = LocalEmbedder::new_with_local_model(onnx_path, tokenizer_dir, Some(512))?;

// Method 2: Using individual file paths
let embedder = LocalEmbedder::new_with_local_files(
    PathBuf::from("/path/to/model.onnx"),
    PathBuf::from("/path/to/tokenizer.json"),
    PathBuf::from("/path/to/config.json"),
    PathBuf::from("/path/to/special_tokens_map.json"),
    PathBuf::from("/path/to/tokenizer_config.json"),
    Some(512) // max_length
)?;
```

**Required Files for Local Models:**

1. **ONNX Model File**: Your embedding model in ONNX format (`.onnx`)
2. **Tokenizer Files**: Four JSON files typically found with transformer models:
   - `tokenizer.json` - Main tokenizer configuration
   - `config.json` - Model configuration
   - `special_tokens_map.json` - Special token mappings
   - `tokenizer_config.json` - Tokenizer-specific configuration

These files are commonly found in HuggingFace model repositories or can be exported when converting models to ONNX format.

## Development

```bash
# Clone the repository
git clone https://github.com/nnanto/localsearch.git
cd localsearch

# Run tests
cargo test

# Run CLI with features
cargo run --features cli -- search "query"
```

## License

MIT License - see [LICENSE](LICENSE) file for details.