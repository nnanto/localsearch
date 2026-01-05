mod util;

use clap::{Parser, Subcommand};
use localsearch::{LocalEmbedder, LocalSearch, SearchType, SqliteLocalSearchEngine};
use util::{JsonFileIngestor, RawFileIngestor};

use crate::util::ingest::IngestionResult;

#[derive(Parser)]
#[command(name = "local-search")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand)]
enum Commands {
    /// Index documents from a directory or file
    Index {
        /// Path to directory or file to index
        path: String,
        /// Database file path (default: ./.localsearch.db)
        #[clap(
            long,
            default_value = "./.localsearch.db",
            help = "Path to the SQLite database file."
        )]
        db: String,
        /// File type filter: json, text
        #[clap(
            long,
            default_value = "json",
            help = "Type of files to ingest: 'json' for JSON files, 'text' for raw text files. json is expected to contain [{\"path\": \"unique_str\", \"content\": \"document content\", \"metadata\": {\"key\": \"value\"}}]."
        )]
        file_type: String,
    },
    /// Search indexed documents
    Search {
        /// Search query
        query: String,
        /// Database file path (default: ./.localsearch.db)
        #[clap(
            long,
            default_value = "./.localsearch.db",
            help = "Path to the SQLite database file that was indexed."
        )]
        db: String,
        /// Search type: fulltext, semantic, or hybrid
        #[clap(
            long,
            default_value = "hybrid",
            help = "Type of search to perform: 'fulltext' for traditional text search, 'semantic' for embedding-based search, or 'hybrid' for a combination of both."
        )]
        search_type: String,
        /// Maximum number of results to return
        #[clap(
            long,
            default_value = "10",
            help = "Maximum number of search results to return."
        )]
        limit: usize,
        /// Output results as pretty format instead of json text
        #[clap(
            long,
            help = "Output search results in pretty format instead of json text."
        )]
        pretty: bool,
    },
}

fn validate_db_presence(db_path: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(db_path).exists() {
        return Err(anyhow::anyhow!(
            "Database file '{}' does not exist. Please run the 'index' command first to create and populate the database.",
            db_path
        ));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command {
        Commands::Index {
            path,
            db,
            file_type,
        } => {
            println!(
                "Indexing documents from: {} and storing in database: {}",
                path, db
            );

            // Initialize the search engine
            let embedder = LocalEmbedder::new_with_default_model()?;
            let engine = SqliteLocalSearchEngine::new(&db, Some(embedder))?;
            engine.create_table()?;
            let boxed_engine = Box::new(engine);

            // Choose the appropriate ingestor based on file type
            let ingestion_result: IngestionResult = match file_type.as_str() {
                "json" => {
                    let ingestor = JsonFileIngestor::new(boxed_engine);
                    ingestor.ingest(&path)?
                }
                "text" => {
                    let ingestor = RawFileIngestor::new(boxed_engine);
                    ingestor.ingest(&path, |file_path| {
                        // Accept common text file extensions
                        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                            matches!(
                                ext,
                                "txt"
                                    | "md"
                                    | "py"
                                    | "rs"
                                    | "js"
                                    | "ts"
                                    | "html"
                                    | "css"
                                    | "json"
                                    | "xml"
                                    | "yaml"
                                    | "yml"
                            )
                        } else {
                            false
                        }
                    })?
                }
                _ => {
                    // Return error for unsupported file types
                    println!(
                        "Unsupported file type: {}. Use 'json' or 'text'.",
                        file_type
                    );
                    IngestionResult::new()
                }
            };

            if !ingestion_result.failed_files.is_empty() {
                println!("Failed files:");
                for file_path in &ingestion_result.failed_files {
                    println!(" - {}", file_path);
                }
            } else {
                println!(
                    "Indexing completed! \nSuccessfully indexed {} file(s). Total documents in the database: {}",
                    ingestion_result.indexed_count, ingestion_result.total_document_count
                );
            }
        }
        Commands::Search {
            query,
            db,
            search_type,
            limit,
            pretty,
        } => {
            if pretty {
                println!("Searching for: \"{}\"", query);
            }
            validate_db_presence(&db)?;
            // Initialize the search engine
            let embedder = LocalEmbedder::new_with_default_model()?;
            let engine = SqliteLocalSearchEngine::new(&db, Some(embedder))?;

            // Parse search type
            let search_type_enum = match search_type.as_str() {
                "fulltext" | "fts" => SearchType::FullText,
                "semantic" | "embedding" => SearchType::Semantic,
                _ => SearchType::Hybrid,
            };

            // Perform search
            let results = engine.search(&query, search_type_enum, Some(limit as i8))?;

            if !pretty {
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
                println!("No results found.");
                return Ok(());
            }

            println!("Found {} results:", results.len());
            println!();

            // Display results (limited by the limit parameter)
            for (i, result) in results.iter().take(limit).enumerate() {
                println!("Result {} - Score: {:.4}", i + 1, result.final_score);
                println!("   Path: {}", result.path);

                if let Some(fts_score) = result.fts_score {
                    println!("   FTS Score: {:.4}", fts_score);
                }

                if let Some(semantic_score) = result.semantic_score {
                    println!("   Semantic Score: {:.4}", semantic_score);
                }

                if let Some(ref metadata) = result.metadata
                    && !metadata.is_empty()
                {
                    println!("   Metadata: {:?}", metadata);
                }

                println!();
            }
        }
    }
    Ok(())
}
