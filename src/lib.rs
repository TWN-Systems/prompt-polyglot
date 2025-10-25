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
pub use database::{Concept, Database, DatabaseStats, SurfaceForm};
pub use concept_resolver::{CacheStats, ConceptResolver, ResolutionPolicy};
pub use surface_selector::{OptimizationCandidate, SelectionPolicy, SurfaceSelector};
pub use protected_regions::{ProtectedRegion, ProtectedRegionDetector, ProtectionPolicy, RegionType};
pub use concept_optimizer::{ConceptOptimizer, OptimizerStats};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the optimizer with default settings
pub fn init_optimizer() -> anyhow::Result<Optimizer> {
    let tokenizer = Tokenizer::new()?;
    let calculator = ConfidenceCalculator::default();
    Ok(Optimizer::new(calculator, tokenizer))
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
