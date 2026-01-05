use anyhow::{Context, Result};
use fastembed::{EmbeddingModel as FastEmbedModel, InitOptions, TextEmbedding};
use std::sync::OnceLock;

static EMBEDDING_MODEL: OnceLock<TextEmbedding> = OnceLock::new();

/// Embedding model wrapper for semantic search
pub struct EmbeddingModel;

impl EmbeddingModel {
    /// Pre-initialize the model (call during app startup to reduce first-search latency)
    pub fn init() -> Result<()> {
        let start = std::time::Instant::now();
        let _ = Self::get_model(); // Force initialization
        tracing::info!(
            "Embedding model pre-warmed in {}ms",
            start.elapsed().as_millis()
        );
        Ok(())
    }

    /// Get or initialize the global embedding model (lazy + cached)
    fn get_model() -> &'static TextEmbedding {
        EMBEDDING_MODEL.get_or_init(|| {
            tracing::info!("Initializing embedding model (all-MiniLM-L6-v2)...");
            let start = std::time::Instant::now();

            // Use explicit cache directory for better control
            let cache_dir = std::env::current_dir()
                .ok()
                .map(|p| p.join(".fastembed_cache"))
                .unwrap_or_else(|| std::path::PathBuf::from(".fastembed_cache"));

            let model = TextEmbedding::try_new(
                InitOptions::new(FastEmbedModel::AllMiniLML6V2)
                    .with_show_download_progress(false)
                    .with_cache_dir(cache_dir),
            )
            .expect("Failed to initialize embedding model");

            tracing::info!(
                "Embedding model initialized in {}ms",
                start.elapsed().as_millis()
            );
            model
        })
    }

    /// Generate embeddings for a text string
    pub fn embed(text: &str) -> Result<Vec<f32>> {
        let start = std::time::Instant::now();
        let model = Self::get_model();
        let model_time = start.elapsed();

        let start = std::time::Instant::now();
        let embeddings = model
            .embed(vec![text], None)
            .context("Failed to generate embeddings")?;
        let embed_time = start.elapsed();

        tracing::debug!(
            "Embedding timing: model_get={}ms, embed_gen={}ms, total={}ms",
            model_time.as_millis(),
            embed_time.as_millis(),
            (model_time + embed_time).as_millis()
        );

        // Extract first (and only) embedding
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embeddings generated"))
    }

    /// Batch embed multiple texts (more efficient than calling embed() repeatedly)
    pub fn embed_batch(texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let start = std::time::Instant::now();
        let model = Self::get_model();

        let embeddings = model
            .embed(texts.clone(), None)
            .context("Failed to generate batch embeddings")?;

        tracing::debug!(
            "Batch embedding: {} texts in {}ms ({}ms/text avg)",
            texts.len(),
            start.elapsed().as_millis(),
            start.elapsed().as_millis() / texts.len() as u128
        );

        Ok(embeddings)
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
        assert!(
            (similarity - 1.0).abs() < 0.001,
            "Same text should have similarity ~1.0"
        );
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

        assert!(
            sim_related > sim_unrelated,
            "Related concepts should have higher similarity than unrelated ones"
        );
        assert!(
            sim_related > 0.5,
            "Related concepts should have similarity > 0.5"
        );
    }
}
