#[cfg(feature = "cli")]
mod util;
#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use colored::*;
#[cfg(feature = "cli")]
use local_search::{SqliteLocalSearchEngine, LocalEmbedder, LocalSearch, SearchType};
#[cfg(feature = "cli")]
use util::{JsonFileIngestor, RawFileIngestor};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "local-search")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Index documents from a directory or file
    Index { 
        /// Path to directory or file to index
        path: String,
        /// Database file path (default: ./.local_search.db)
        #[clap(long, default_value = "./.local_search.db", help = "Path to the SQLite database file.")]
        db: String,
        /// File type filter: json, text
        #[clap(long, default_value = "json", help = "Type of files to ingest: 'json' for JSON files, 'text' for raw text files. json is expected to contain [{\"path\": \"unique_str\", \"content\": \"document content\", \"metadata\": {\"key\": \"value\"}}].")]
        file_type: String,
    },
    /// Search indexed documents
    Search { 
        /// Search query
        query: String,
        /// Database file path (default: ./.local_search.db)
        #[clap(long, default_value = "./.local_search.db", help = "Path to the SQLite database file that was indexed.")]
        db: String,
        /// Search type: fulltext, semantic, or hybrid
        #[clap(long, default_value = "hybrid", help = "Type of search to perform: 'fulltext' for traditional text search, 'semantic' for embedding-based search, or 'hybrid' for a combination of both.")]
        search_type: String,
        /// Maximum number of results to return
        #[clap(long, default_value = "10", help = "Maximum number of search results to return.")]
        limit: usize,
        /// Output results as JSON instead of formatted text
        #[clap(long, help = "Output search results in JSON format.")]
        json: bool,
    },
}

fn validate_db_presence(db_path: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(db_path).exists() {
        return Err(anyhow::anyhow!("Database file '{}' does not exist. Please run the 'index' command first to create and populate the database.", db_path));
    }
    Ok(())
}

#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Index { path, db, file_type } => {
            println!("{} Indexing documents from: {}", "📚".green(), path);
            
            // Initialize the search engine
            let embedder = LocalEmbedder::default()?;
            let engine = SqliteLocalSearchEngine::new(&db, Some(embedder))?;
            engine.create_table()?;
            
            // Choose the appropriate ingestor based on file type
            match file_type.as_str() {
                "json" => {
                    let ingestor = JsonFileIngestor::new(Box::new(engine));
                    ingestor.ingest(&path)?;
                }
                "text" => {
                    let engine_boxed = Box::new(engine);
                    let ingestor = RawFileIngestor::new(engine_boxed);
                    ingestor.ingest(&path, |file_path| {
                        // Accept common text file extensions
                        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                            matches!(ext, "txt" | "md" | "py" | "rs" | "js" | "ts" | "html" | "css" | "json" | "xml" | "yaml" | "yml")
                        } else {
                            false
                        }
                    })?;
                }
                _ => {
                    // Return error for unsupported file types
                    return Err(anyhow::anyhow!("Unsupported file type: {}. Use 'json' or 'text'.", file_type));
                }
            }
            
            println!("{} Indexing completed!", "✅".green());
        }
        Commands::Search { query, db, search_type, limit, json } => {
            if !json {
                println!("{} Searching for: \"{}\"", "🔍".blue(), query);
            }
            validate_db_presence(&db)?;
            // Initialize the search engine
            let embedder = LocalEmbedder::default()?;
            let engine = SqliteLocalSearchEngine::new(&db, Some(embedder))?;
            
            // Parse search type
            let search_type_enum = match search_type.as_str() {
                "fulltext" | "fts" => SearchType::FullText,
                "semantic" | "embedding" => SearchType::Semantic,
                "hybrid" | _ => SearchType::Hybrid,
            };
            
            // Perform search
            let results = engine.search(&query, search_type_enum, Some(limit as i8))?;
            
            if json {
                // Output as JSON
                let json_output = serde_json::json!({
                    "query": query,
                    "search_type": search_type,
                    "results_count": results.len(),
                    "results": results.iter().take(limit).map(|result| {
                        serde_json::json!({
                            "path": result.path,
                            "final_score": result.final_score,
                            "fts_score": result.fts_score,
                            "semantic_score": result.semantic_score,
                            "metadata": result.metadata
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
                return Ok(());
            }
            
            if results.is_empty() {
                println!("{} No results found.", "📭".yellow());
                return Ok(());
            }
            
            println!("{} Found {} results:", "📋".blue(), results.len());
            println!();
            
            // Display results (limited by the limit parameter)
            for (i, result) in results.iter().take(limit).enumerate() {
                println!("{} Result {} - Score: {:.4}", 
                    "🔸".cyan(), 
                    i + 1, 
                    result.final_score);
                println!("   📄 Path: {}", result.path.green());
                
                if let Some(fts_score) = result.fts_score {
                    println!("   📊 FTS Score: {:.4}", fts_score);
                }
                
                if let Some(semantic_score) = result.semantic_score {
                    println!("   🧠 Semantic Score: {:.4}", semantic_score);
                }
                
                if let Some(ref metadata) = result.metadata {
                    if !metadata.is_empty() {
                        println!("   🏷️  Metadata: {:?}", metadata);
                    }
                }
                
                println!();
            }
        }
    }
    Ok(())
}