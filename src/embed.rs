use fastembed::{InitOptions, TextEmbedding};
use anyhow::Result;
use log::{debug, info};

pub struct LocalEmbedder {
    model: TextEmbedding,
}

impl LocalEmbedder {
    pub fn new() -> Result<Self> {
        let model_name: fastembed::EmbeddingModel = fastembed::EmbeddingModel::AllMiniLML6V2;
        let model = TextEmbedding::try_new(InitOptions::new(model_name.clone()))?;
        
        info!("Initialized embedding model: {:?}", model_name);
        
        Ok(LocalEmbedder { model })
    }
    
    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        embeddings.into_iter().next().map(|x| self.normalize_l2(&x)).ok_or_else(|| anyhow::anyhow!("Failed to get embedding"))
    }
    
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings.iter().map(|e| self.normalize_l2(e)).collect())
    }

    pub fn normalize_l2(&self, embedding: &[f32]) -> Vec<f32> {
        let norm = (embedding.iter().map(|x| x * x).sum::<f32>()).sqrt();
        debug!("Normalized embedding with L2 norm: {}", norm);
        if norm < 1e-5 || (norm - 1.0).abs() < 1e-5 {
            debug!("Embedding norm {} is less than 2.0, returning original embedding", norm);
            embedding.to_vec()
        } else {
            embedding.iter().map(|x| x / norm).collect()
        }
        
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_l2_small_norm() {
        let embedder = LocalEmbedder::new().unwrap();
        let input: Vec<f32> = vec![0.1, 0.2, 0.3];

        let result = embedder.normalize_l2(&input);
        assert_ne!(result, input)
    }

    #[test]
    fn test_embed_text_returns_vector() {
        let embedder = LocalEmbedder::new().expect("Failed to create embedder");
        let text = "Hello world";
        
        let result = embedder.embed_text(text);
        assert!(result.is_ok());
        
        let embedding = result.unwrap();
        assert!(!embedding.is_empty());
        assert!(embedding.len() > 0);
    }

    #[test]
    fn test_embed_batch_same_length() {
        let embedder = LocalEmbedder::new().expect("Failed to create embedder");
        let texts = vec!["Hello", "World", "Test"];
        
        let result = embedder.embed_batch(texts.clone());
        assert!(result.is_ok());
        
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), texts.len());
    }
}