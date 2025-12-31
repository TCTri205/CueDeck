use anyhow::{Result, Context};
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel as FastEmbedModel};
use std::sync::OnceLock;

static EMBEDDING_MODEL: OnceLock<TextEmbedding> = OnceLock::new();

/// Embedding model wrapper for semantic search
pub struct EmbeddingModel;

impl EmbeddingModel {
    /// Get or initialize the global embedding model (lazy + cached)
    fn get_model() -> &'static TextEmbedding {
        EMBEDDING_MODEL.get_or_init(|| {
            tracing::info!("Initializing embedding model (all-MiniLM-L6-v2)...");
            
            let model = TextEmbedding::try_new(
                InitOptions::new(FastEmbedModel::AllMiniLML6V2)
                    .with_show_download_progress(false)
            ).expect("Failed to initialize embedding model"); // Handle error internally
            
            tracing::info!("Embedding model initialized successfully");
            model
        })
    }

    /// Generate embeddings for a text string
    pub fn embed(text: &str) -> Result<Vec<f32>> {
        let model = Self::get_model();  // No longer returns Result
        
        let embeddings = model
            .embed(vec![text], None)
            .context("Failed to generate embeddings")?;
        
        // Extract first (and only) embedding
        embeddings.into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embeddings generated"))
    }

    /// Calculate cosine similarity between two vectors
    /// Returns value in range [-1, 1], where 1 = identical, -1 = opposite
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            tracing::warn!("Vector dimension mismatch: {} vs {}", a.len(), b.len());
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model_init() {
        // This will download model on first run (~22MB)
        let _model = EmbeddingModel::get_model();
        // If we get here, initialization succeeded
    }

    #[test]
    fn test_embed_text() {
        let vec = EmbeddingModel::embed("hello world").unwrap();
        assert_eq!(vec.len(), 384, "MiniLM-L6 should produce 384-dim vectors");
    }

    #[test]
    fn test_embed_consistency() {
        let vec1 = EmbeddingModel::embed("rust programming").unwrap();
        let vec2 = EmbeddingModel::embed("rust programming").unwrap();
        
        let similarity = EmbeddingModel::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001, "Same text should have similarity ~1.0");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        let sim_identical = EmbeddingModel::cosine_similarity(&a, &b);
        assert!((sim_identical - 1.0).abs() < 0.001);

        let sim_orthogonal = EmbeddingModel::cosine_similarity(&a, &c);
        assert!((sim_orthogonal - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_similarity() {
        let vec_async = EmbeddingModel::embed("asynchronous programming").unwrap();
        let vec_concurrent = EmbeddingModel::embed("concurrent execution").unwrap();
        let vec_unrelated = EmbeddingModel::embed("cooking recipes").unwrap();

        let sim_related = EmbeddingModel::cosine_similarity(&vec_async, &vec_concurrent);
        let sim_unrelated = EmbeddingModel::cosine_similarity(&vec_async, &vec_unrelated);

        assert!(sim_related > sim_unrelated, 
            "Related concepts should have higher similarity than unrelated ones");
        assert!(sim_related > 0.5, "Related concepts should have similarity > 0.5");
    }
}
