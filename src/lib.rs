pub mod api;
pub mod confidence;
pub mod models;
pub mod optimizer;
pub mod patterns;
pub mod tokenizer;

// Phase 3: Concept Atlas modules
pub mod tokenizer_registry;
pub mod database;
pub mod concept_resolver;
pub mod surface_selector;
pub mod protected_regions;
pub mod concept_optimizer;
pub mod database_pattern_detector;
pub mod database_optimizer;

pub use confidence::{extract_context, ConfidenceCalculator, Context};
pub use models::{
    Config, DirectiveFormat, Language, Optimization, OptimizationConfidence, OptimizationCorpus,
    OptimizationRequest, OptimizationResult, OptimizationType, PatternStats, ReviewDecision,
    ReviewSession,
};
pub use optimizer::Optimizer;
pub use patterns::{DetectedPattern, Pattern, PatternDetector};
pub use tokenizer::Tokenizer;
pub use tokenizer_registry::{TokenizerBackend, TokenizerId, TokenizerRegistry};
pub use database::{Concept, Database, DatabaseStats, SurfaceForm, PatternRecord, HitlDecision, PatternTypeStats};
pub use concept_resolver::{CacheStats, ConceptResolver, ResolutionPolicy};
pub use surface_selector::{OptimizationCandidate, SelectionPolicy, SurfaceSelector};
pub use protected_regions::{ProtectedRegion, ProtectedRegionDetector, ProtectionPolicy, RegionType};
pub use concept_optimizer::{ConceptOptimizer, OptimizerStats};
pub use database_pattern_detector::DatabasePatternDetector;
pub use database_optimizer::DatabaseOptimizer;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the optimizer with default settings (hardcoded patterns)
///
/// **Deprecated**: Use `init_database_optimizer()` for production use.
/// This function uses hardcoded patterns and is kept for backward compatibility.
pub fn init_optimizer() -> anyhow::Result<Optimizer> {
    let tokenizer = Tokenizer::new()?;
    let calculator = ConfidenceCalculator::default();
    Ok(Optimizer::new(calculator, tokenizer))
}

/// Initialize the database-backed optimizer
///
/// This is the recommended way to initialize the optimizer for production use.
/// Patterns are loaded from SQLite and can be updated via HITL feedback.
///
/// # Arguments
/// * `db_path` - Path to the SQLite database file (e.g., "atlas.db")
///
/// # Example
/// ```no_run
/// use prompt_compress::init_database_optimizer;
///
/// let optimizer = init_database_optimizer("atlas.db").unwrap();
/// ```
pub fn init_database_optimizer(db_path: &str) -> anyhow::Result<DatabaseOptimizer> {
    use std::sync::Arc;

    let db = Database::open(db_path)?;
    let tokenizer = Tokenizer::new()?;
    let calculator = ConfidenceCalculator::default();

    DatabaseOptimizer::new(Arc::new(db), calculator, tokenizer)
}

/// Initialize database-backed optimizer with minimum confidence threshold
///
/// Only patterns with confidence >= min_confidence will be loaded.
pub fn init_database_optimizer_with_confidence(
    db_path: &str,
    min_confidence: f64,
) -> anyhow::Result<DatabaseOptimizer> {
    use std::sync::Arc;

    let db = Database::open(db_path)?;
    let tokenizer = Tokenizer::new()?;
    let calculator = ConfidenceCalculator::default();

    DatabaseOptimizer::with_confidence(Arc::new(db), calculator, tokenizer, min_confidence)
}

/// Load corpus from file
pub fn load_corpus(path: &str) -> anyhow::Result<OptimizationCorpus> {
    let data = std::fs::read_to_string(path)?;
    let corpus: OptimizationCorpus = serde_json::from_str(&data)?;
    Ok(corpus)
}

/// Save corpus to file
pub fn save_corpus(corpus: &OptimizationCorpus, path: &str) -> anyhow::Result<()> {
    let data = serde_json::to_string_pretty(corpus)?;
    std::fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init_optimizer() {
        let optimizer = init_optimizer();
        assert!(optimizer.is_ok());
    }
}
