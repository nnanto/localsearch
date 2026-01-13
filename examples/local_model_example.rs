//! Example demonstrating how to use LocalEmbedder with local ONNX models
//! 
//! This example shows how to initialize the LocalEmbedder with your own
//! local ONNX embedding model and tokenizer files instead of using the
//! pre-built FastEmbed models.

use localsearch::{DocumentIndexer, DocumentRequest, LocalEmbedder, SqliteLocalSearchEngine, LocalSearch, SearchType};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Example 1: Using a tokenizer directory
    // This assumes you have:
    // - /path/to/your/model.onnx
    // - /path/to/tokenizer/directory/tokenizer.json
    // - /path/to/tokenizer/directory/config.json
    // - /path/to/tokenizer/directory/special_tokens_map.json
    // - /path/to/tokenizer/directory/tokenizer_config.json
    
    println!("Example 1: Using local model with tokenizer directory");
    
    // Uncomment and modify these paths to point to your actual model files
    /*
    let onnx_path = PathBuf::from("/path/to/your/model.onnx");
    let tokenizer_dir = PathBuf::from("/path/to/tokenizer/directory");
    
    // Optional: specify max sequence length (default is used if None)
    let max_length = Some(512);
    
    match LocalEmbedder::new_with_local_model(onnx_path, tokenizer_dir, max_length) {
        Ok(embedder) => {
            println!("✅ Successfully initialized local model embedder!");
            
            // Test embedding
            let text = "This is a test sentence.";
            match embedder.embed_text(text) {
                Ok(embedding) => {
                    println!("✅ Successfully created embedding with {} dimensions", embedding.len());
                }
                Err(e) => {
                    eprintln!("❌ Failed to create embedding: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize local model embedder: {}", e);
        }
    }
    */

    // Example 2: Using individual file paths
    println!("\nExample 2: Using individual file paths");
    
    /*
    let embedder = LocalEmbedder::new_with_local_files(
        PathBuf::from("/path/to/model.onnx"),
        PathBuf::from("/path/to/tokenizer.json"),
        PathBuf::from("/path/to/config.json"),
        PathBuf::from("/path/to/special_tokens_map.json"),
        PathBuf::from("/path/to/tokenizer_config.json"),
        Some(512), // max_length
    )?;
    
    // Use the embedder with a search engine
    let db_path = "/tmp/local_model_example.db";
    let mut engine = SqliteLocalSearchEngine::new(db_path, Some(embedder))?;
    engine.create_table()?;

    // Index a document
    engine.insert_document(DocumentRequest {
        path: "example/doc1".to_string(),
        content: "This is an example document with custom embeddings.".to_string(),
        metadata: None,
    })?;

    // Search using the custom model
    let results = engine.search("example custom embeddings", SearchType::Semantic, Some(5))?;
    println!("Found {} results", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("Result {}: {} (score: {:.4})", 
            i + 1, 
            result.path, 
            result.semantic_score.unwrap_or(0.0)
        );
    }
    */

    // Example 3: Fallback to default model if local model fails
    println!("\nExample 3: Using default model (working example)");
    
    // This will work with the default FastEmbed model
    let embedder = LocalEmbedder::new_with_default_model()?;
    println!("✅ Successfully initialized default model embedder!");
    
    // Test embedding with default model
    let text = "This is a test with the default model.";
    let embedding = embedder.embed_text(text)?;
    println!("✅ Successfully created embedding with {} dimensions", embedding.len());

    println!("\n📝 To use your own local models:");
    println!("1. Uncomment the code examples above");
    println!("2. Replace the file paths with your actual model and tokenizer paths");
    println!("3. Ensure you have all required tokenizer files:");
    println!("   - tokenizer.json");
    println!("   - config.json");
    println!("   - special_tokens_map.json");
    println!("   - tokenizer_config.json");

    Ok(())
}