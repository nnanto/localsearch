use crate::config::LocalSearchDirs;
use anyhow::Result;
use fastembed::{
    InitOptions, InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use log::{debug, info};
use std::{fs, path::PathBuf};

/// Local text embedding service using FastEmbed models.
/// 
/// Supports both pre-built models from the FastEmbed library and local ONNX models
/// with custom tokenizers. Local models require an ONNX file and four tokenizer files:
/// tokenizer.json, config.json, special_tokens_map.json, and tokenizer_config.json.
pub struct LocalEmbedder {
    model: TextEmbedding,
}

impl LocalEmbedder {
    /// Creates a new embedder with the specified model or default AllMiniLML6V2.
    /// If cache_dir is provided, uses that; otherwise uses LocalSearchDirs default.
    pub fn new(
        model_name: Option<fastembed::EmbeddingModel>,
        cache_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let model_name = model_name.unwrap_or(fastembed::EmbeddingModel::AllMiniLML6V2);

        let cache_dir = match cache_dir {
            Some(dir) => dir,
            None => {
                let dirs = LocalSearchDirs::new();
                dirs.ensure_cache_dir()?
            }
        };

        let init_options = InitOptions::new(model_name.clone()).with_cache_dir(cache_dir);
        let model = TextEmbedding::try_new(init_options)?;

        info!("Initialized embedding model: {:?}", model_name);

        Ok(LocalEmbedder { model })
    }

    /// Creates a new embedder with local model files.
    /// 
    /// # Arguments
    /// * `onnx_model_path` - Path to the ONNX model file
    /// * `tokenizer_dir` - Path to directory containing tokenizer files:
    ///   - tokenizer.json
    ///   - config.json  
    ///   - special_tokens_map.json
    ///   - tokenizer_config.json
    /// * `max_length` - Optional maximum sequence length (default: 512)
    pub fn new_with_local_model(
        onnx_model_path: PathBuf,
        tokenizer_dir: PathBuf,
        max_length: Option<usize>,
    ) -> Result<Self> {
        // Load ONNX model file
        let onnx_file = fs::read(&onnx_model_path)
            .map_err(|e| anyhow::anyhow!("Failed to read ONNX model from {:?}: {}", onnx_model_path, e))?;

        // Load tokenizer files
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: fs::read(tokenizer_dir.join("tokenizer.json"))
                .map_err(|e| anyhow::anyhow!("Failed to read tokenizer.json: {}", e))?,
            config_file: fs::read(tokenizer_dir.join("config.json"))
                .map_err(|e| anyhow::anyhow!("Failed to read config.json: {}", e))?,
            special_tokens_map_file: fs::read(tokenizer_dir.join("special_tokens_map.json"))
                .map_err(|e| anyhow::anyhow!("Failed to read special_tokens_map.json: {}", e))?,
            tokenizer_config_file: fs::read(tokenizer_dir.join("tokenizer_config.json"))
                .map_err(|e| anyhow::anyhow!("Failed to read tokenizer_config.json: {}", e))?,
        };

        // Create user-defined model
        let user_defined_model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);

        // Set up initialization options
        let mut init_options = InitOptionsUserDefined::new();
        if let Some(max_len) = max_length {
            init_options = init_options.with_max_length(max_len);
        }

        // Initialize the model
        let model = TextEmbedding::try_new_from_user_defined(user_defined_model, init_options)?;

        info!(
            "Initialized local embedding model from {:?} with tokenizer from {:?}",
            onnx_model_path, tokenizer_dir
        );

        Ok(LocalEmbedder { model })
    }

    /// Creates a new embedder with local model files using individual file paths.
    /// 
    /// # Arguments
    /// * `onnx_model_path` - Path to the ONNX model file
    /// * `tokenizer_json_path` - Path to tokenizer.json
    /// * `config_json_path` - Path to config.json
    /// * `special_tokens_map_path` - Path to special_tokens_map.json
    /// * `tokenizer_config_path` - Path to tokenizer_config.json
    /// * `max_length` - Optional maximum sequence length (default: 512)
    pub fn new_with_local_files(
        onnx_model_path: PathBuf,
        tokenizer_json_path: PathBuf,
        config_json_path: PathBuf,
        special_tokens_map_path: PathBuf,
        tokenizer_config_path: PathBuf,
        max_length: Option<usize>,
    ) -> Result<Self> {
        // Load ONNX model file
        let onnx_file = fs::read(&onnx_model_path)
            .map_err(|e| anyhow::anyhow!("Failed to read ONNX model from {:?}: {}", onnx_model_path, e))?;

        // Load tokenizer files
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: fs::read(&tokenizer_json_path)
                .map_err(|e| anyhow::anyhow!("Failed to read tokenizer.json from {:?}: {}", tokenizer_json_path, e))?,
            config_file: fs::read(&config_json_path)
                .map_err(|e| anyhow::anyhow!("Failed to read config.json from {:?}: {}", config_json_path, e))?,
            special_tokens_map_file: fs::read(&special_tokens_map_path)
                .map_err(|e| anyhow::anyhow!("Failed to read special_tokens_map.json from {:?}: {}", special_tokens_map_path, e))?,
            tokenizer_config_file: fs::read(&tokenizer_config_path)
                .map_err(|e| anyhow::anyhow!("Failed to read tokenizer_config.json from {:?}: {}", tokenizer_config_path, e))?,
        };

        // Create user-defined model
        let user_defined_model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);

        // Set up initialization options
        let mut init_options = InitOptionsUserDefined::new();
        if let Some(max_len) = max_length {
            init_options = init_options.with_max_length(max_len);
        }

        // Initialize the model
        let model = TextEmbedding::try_new_from_user_defined(user_defined_model, init_options)?;

        info!("Initialized local embedding model from individual files");

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

    #[test]
    fn test_new_with_local_model_invalid_paths() {
        let onnx_path = PathBuf::from("/invalid/path/model.onnx");
        let tokenizer_dir = PathBuf::from("/invalid/path/tokenizer");

        let result = LocalEmbedder::new_with_local_model(onnx_path, tokenizer_dir, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_with_local_files_invalid_paths() {
        let onnx_path = PathBuf::from("/invalid/path/model.onnx");
        let tokenizer_json = PathBuf::from("/invalid/path/tokenizer.json");
        let config_json = PathBuf::from("/invalid/path/config.json");
        let special_tokens = PathBuf::from("/invalid/path/special_tokens_map.json");
        let tokenizer_config = PathBuf::from("/invalid/path/tokenizer_config.json");

        let result = LocalEmbedder::new_with_local_files(
            onnx_path,
            tokenizer_json,
            config_json,
            special_tokens,
            tokenizer_config,
            None,
        );
        assert!(result.is_err());
    }
}
