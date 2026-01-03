use local_search_engine::LocalEmbedder;


#[test]
fn test_embedder_integration() {
    let embedder = LocalEmbedder::default().expect("Failed to create local embedding");
    let texts = vec!["artificial intelligence", "machine learning", "deep learning"];
    let embeddings = embedder.embed_batch(texts).expect("Failed to embed batch");
    
    assert_eq!(embeddings.len(), 3);
    
    // Test that similar texts have similar embeddings
    // This would require implementing cosine similarity
    for embedding in embeddings {
        assert!(!embedding.is_empty());
    }
}