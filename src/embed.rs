use anyhow::Result;
use fastembed::{InitOptions, TextEmbedding};
use log::{debug, info};
use std::path::PathBuf;
use crate::config::LocalSearchDirs;

/// Local text embedding service using FastEmbed models.
pub struct LocalEmbedder {
    model: TextEmbedding,
}

impl LocalEmbedder {
    /// Creates a new embedder with the specified model or default AllMiniLML6V2.
    /// If cache_dir is provided, uses that; otherwise uses LocalSearchDirs default.
    pub fn new(model_name: Option<fastembed::EmbeddingModel>, cache_dir: Option<PathBuf>) -> Result<Self> {
        let model_name = model_name.unwrap_or(fastembed::EmbeddingModel::AllMiniLML6V2);
        
        let cache_dir = match cache_dir {
            Some(dir) => dir,
            None => {
                let dirs = LocalSearchDirs::new();
                dirs.ensure_cache_dir()?
            }
        };
        
        let init_options = InitOptions::new(model_name.clone())
            .with_cache_dir(cache_dir);
        let model = TextEmbedding::try_new(init_options)?;

        info!("Initialized embedding model: {:?}", model_name);

        Ok(LocalEmbedder { model })
    }

    /// Creates a new embedder with the default model and default cache directory.
    pub fn new_with_default_model() -> Result<Self> {
        Self::new(None, None)
    }

    /// Creates a new embedder with the default model and custom cache directory.
    pub fn new_with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        Self::new(None, Some(cache_dir))
    }

    /// Embeds a single text string and returns a normalized vector.
    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        embeddings
            .into_iter()
            .next()
            .map(|x| Self::normalize_l2(&x))
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedding"))
    }

    /// Embeds multiple text strings and returns normalized vectors.
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings.iter().map(|e| Self::normalize_l2(e)).collect())
    }

    /// Normalizes an embedding vector using L2 normalization.
    pub fn normalize_l2(embedding: &[f32]) -> Vec<f32> {
        let norm = (embedding.iter().map(|x| x * x).sum::<f32>()).sqrt();
        debug!("Normalized embedding with L2 norm: {}", norm);
        if norm < 1e-5 {
            debug!(
                "Embedding norm {} is less than 1e-5, returning original embedding",
                norm
            );
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
        let input: Vec<f32> = vec![0.1, 0.2, 0.3];

        let result = LocalEmbedder::normalize_l2(&input);
        assert_ne!(result, input)
    }

    #[test]
    fn test_embed_text_returns_vector() {
        let embedder = LocalEmbedder::new_with_default_model().expect("Failed to create embedder");
        let text = "Hello world";

        let result = embedder.embed_text(text);
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert!(!embedding.is_empty());
    }

    #[test]
    fn test_embed_batch_same_length() {
        let embedder = LocalEmbedder::new(None, None).expect("Failed to create embedder");
        let texts = vec!["Hello", "World", "Test"];

        let result = embedder.embed_batch(texts.clone());
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), texts.len());
    }
}
