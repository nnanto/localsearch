use anyhow::{Result};
use local_search_engine::{SearchType, search::*};

fn main() -> Result<()> {
    env_logger::init();
    
    // Test the search functionality
    let search_engine = SqliteLocalSearch::new("local_search.db")?;
    
    // Insert some test documents
    search_engine.insert_document("1", "notes", "The quick brown fox jumps over the lazy dog. This is about animals and speed.", 0.0, 0.0)?;
    search_engine.insert_document("2", "notes", "Machine learning algorithms are used for data science and artificial intelligence.", 0.0, 0.0)?;
    search_engine.insert_document("3", "notes", "Rust is a systems programming language focused on safety and performance.", 0.0, 0.0)?;
    search_engine.insert_document("4", "notes", "The fox is a clever animal that lives in the forest. Foxes are known for their cunning.", 0.0, 0.0)?;
    search_engine.insert_document("5", "notes", "Neural networks and deep learning are subfields of machine learning.", 0.0, 0.0)?;

    println!("=== Full Text Search ===");
    let fts_results = search_engine.search("fox", SearchType::FullText)?;
    for result in &fts_results {
        println!("ID: {}, Container: {}, FTS Score: {:?}, Semantic Score: {:?}, Final Score: {:.4}", 
                 result.id, result.container, result.fts_score, result.semantic_score, result.final_score);
    }

    println!("\n=== Semantic Search ===");
    let semantic_results = search_engine.search("artificial intelligence", SearchType::Semantic)?;
    for result in &semantic_results {
        println!("ID: {}, Container: {}, FTS Score: {:?}, Semantic Score: {:?}, Final Score: {:.4}", 
                 result.id, result.container, result.fts_score, result.semantic_score, result.final_score);
    }

    println!("\n=== Hybrid Search ===");
    let hybrid_results = search_engine.search("machine learning", SearchType::Hybrid)?;
    for result in &hybrid_results {
        println!("ID: {}, Container: {}, FTS Score: {:?}, Semantic Score: {:?}, Final Score: {:.4}", 
                 result.id, result.container, result.fts_score, result.semantic_score, result.final_score);
    }

    // Clean up
    search_engine.clear_db()?;
    Ok(())
}