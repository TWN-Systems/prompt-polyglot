/// Phase 3: Concept Optimizer - Main optimization pipeline
///
/// Purpose: Integrate all Phase 3 components into a unified optimization system:
/// 1. Protected regions detection (never optimize code/instructions)
/// 2. Structural optimizations (units, formatting, JSON keys)
/// 3. Concept-based optimizations (Q-ID → cheapest surface form)
/// 4. v0.2 pattern-based optimizations (boilerplate, fillers, etc.)
///
/// Philosophy: Layered optimization with safety guarantees

use crate::concept_resolver::{ConceptResolver, ResolutionPolicy};
use crate::database::Database;
use crate::models::{DirectiveFormat, Language, OptimizationRequest, OptimizationResult};
use crate::optimizer::Optimizer as V2Optimizer;
use crate::protected_regions::{ProtectedRegionDetector, ProtectionPolicy};
use crate::surface_selector::{SelectionPolicy, SurfaceSelector};
use crate::tokenizer_registry::{TokenizerId, TokenizerRegistry};
use anyhow::Result;
use std::sync::Arc;

/// Concept-based optimizer (v0.3)
pub struct ConceptOptimizer {
    // Phase 3 components
    db: Arc<Database>,
    resolver: ConceptResolver,
    selector: SurfaceSelector,
    tokenizer_registry: TokenizerRegistry,
    region_detector: ProtectedRegionDetector,

    // v0.2 fallback optimizer
    v2_optimizer: V2Optimizer,

    // Configuration
    tokenizer_id: TokenizerId,
    protection_policy: ProtectionPolicy,
    resolution_policy: ResolutionPolicy,
    selection_policy: SelectionPolicy,
}

impl ConceptOptimizer {
    /// Create new concept optimizer with database
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let resolver = ConceptResolver::new(Arc::clone(&db), ResolutionPolicy::Normalized);
        let selector = SurfaceSelector::new(Arc::clone(&db));
        let tokenizer_registry = TokenizerRegistry::new()?;
        let region_detector = ProtectedRegionDetector::new(ProtectionPolicy::Conservative);
        let v2_optimizer = V2Optimizer::default();

        Ok(Self {
            db,
            resolver,
            selector,
            tokenizer_registry,
            region_detector,
            v2_optimizer,
            tokenizer_id: TokenizerId::Cl100kBase,
            protection_policy: ProtectionPolicy::Conservative,
            resolution_policy: ResolutionPolicy::Normalized,
            selection_policy: SelectionPolicy::MinTokens,
        })
    }

    /// Configure tokenizer to use
    pub fn with_tokenizer(mut self, tokenizer_id: TokenizerId) -> Self {
        self.tokenizer_id = tokenizer_id;
        self
    }

    /// Configure protection policy
    pub fn with_protection_policy(mut self, policy: ProtectionPolicy) -> Self {
        self.protection_policy = policy;
        self.region_detector = ProtectedRegionDetector::new(policy);
        self
    }

    /// Configure resolution policy
    pub fn with_resolution_policy(mut self, policy: ResolutionPolicy) -> Self {
        self.resolution_policy = policy;
        self.resolver = ConceptResolver::new(Arc::clone(&self.db), policy);
        self
    }

    /// Configure selection policy
    pub fn with_selection_policy(mut self, policy: SelectionPolicy) -> Self {
        self.selection_policy = policy;
        self
    }

    /// Main optimization pipeline
    pub fn optimize(&mut self, request: &OptimizationRequest) -> Result<OptimizationResult> {
        // Step 1: Detect protected regions
        let protected_regions = self.region_detector.detect(&request.prompt);

        // Step 2: Try concept-based optimization first
        let concept_optimized = self.try_concept_optimization(&request.prompt, &protected_regions)?;

        // Step 3: Fall back to v0.2 pattern-based optimization
        // (This handles boilerplate, fillers, structural patterns, etc.)
        let fully_optimized = self.v2_optimizer.optimize(&OptimizationRequest {
            prompt: concept_optimized.clone(),
            output_language: request.output_language.clone(),
            confidence_threshold: request.confidence_threshold,
            aggressive_mode: request.aggressive_mode,
            directive_format: request.directive_format.clone(),
        })?;

        Ok(fully_optimized)
    }

    /// Try concept-based optimization
    fn try_concept_optimization(
        &self,
        prompt: &str,
        protected_regions: &[crate::protected_regions::ProtectedRegion],
    ) -> Result<String> {
        let mut result = prompt.to_string();
        let words = self.extract_words(prompt);

        let tokenizer = self.tokenizer_registry
            .get(self.tokenizer_id)
            .ok_or_else(|| anyhow::anyhow!("Tokenizer not available"))?;

        for word in words {
            // Skip if word is in protected region
            if self.is_word_protected(&word, protected_regions, prompt) {
                continue;
            }

            // Try to resolve to concept
            if let Some(concept) = self.resolver.resolve(&word.text)? {
                // Get original token count
                let original_tokens = tokenizer.count_tokens(&word.text);

                // Try to find better surface form
                if let Some(candidate) = self.selector.calculate_savings(
                    &concept.qid,
                    self.tokenizer_id,
                    &word.text,
                    original_tokens,
                    &self.selection_policy,
                )? {
                    // Only apply if we save tokens and have high confidence
                    if candidate.token_savings > 0 {
                        result = result.replace(&word.text, &candidate.optimized_form);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Extract words from text
    fn extract_words(&self, text: &str) -> Vec<Word> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut start_pos = 0;

        for (pos, ch) in text.char_indices() {
            if ch.is_alphabetic() {
                if current_word.is_empty() {
                    start_pos = pos;
                }
                current_word.push(ch);
            } else if !current_word.is_empty() {
                words.push(Word {
                    text: current_word.clone(),
                    start_pos,
                    end_pos: pos,
                });
                current_word.clear();
            }
        }

        // Handle last word
        if !current_word.is_empty() {
            words.push(Word {
                text: current_word,
                start_pos,
                end_pos: text.len(),
            });
        }

        words
    }

    /// Check if word is in protected region
    fn is_word_protected(
        &self,
        word: &Word,
        protected_regions: &[crate::protected_regions::ProtectedRegion],
        _text: &str,
    ) -> bool {
        self.region_detector.is_protected(protected_regions, word.start_pos, word.end_pos)
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> OptimizerStats {
        OptimizerStats {
            cache_stats: self.resolver.cache_stats(),
            db_stats: self.db.get_stats().unwrap_or_else(|_| crate::database::DatabaseStats {
                total_concepts: 0,
                total_surface_forms: 0,
                cache_size: 0,
            }),
        }
    }
}

/// Word extracted from text
#[derive(Debug, Clone)]
struct Word {
    text: String,
    start_pos: usize,
    end_pos: usize,
}

/// Optimizer statistics
#[derive(Debug, Clone)]
pub struct OptimizerStats {
    pub cache_stats: crate::concept_resolver::CacheStats,
    pub db_stats: crate::database::DatabaseStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Concept, Database, SurfaceForm};

    fn setup_test_optimizer() -> ConceptOptimizer {
        let db = Database::in_memory().unwrap();

        // Add test concept: hospital
        db.upsert_concept(&Concept {
            qid: "Q16917".to_string(),
            label_en: "hospital".to_string(),
            description: Some("healthcare facility".to_string()),
            category: Some("medical".to_string()),
        }).unwrap();

        // Add surface forms
        db.insert_surface_form(&SurfaceForm {
            qid: "Q16917".to_string(),
            tokenizer_id: "cl100k_base".to_string(),
            lang: "en".to_string(),
            form: "hospital".to_string(),
            token_count: 1,
            char_count: 8,
        }).unwrap();

        db.insert_surface_form(&SurfaceForm {
            qid: "Q16917".to_string(),
            tokenizer_id: "cl100k_base".to_string(),
            lang: "es".to_string(),
            form: "hospital".to_string(),
            token_count: 1,
            char_count: 8,
        }).unwrap();

        ConceptOptimizer::new(Arc::new(db)).unwrap()
    }

    #[test]
    fn test_create_optimizer() {
        let optimizer = setup_test_optimizer();
        let stats = optimizer.get_stats();
        assert_eq!(stats.db_stats.total_concepts, 1);
        assert_eq!(stats.db_stats.total_surface_forms, 2);
    }

    #[test]
    fn test_extract_words() {
        let optimizer = setup_test_optimizer();
        let words = optimizer.extract_words("The hospital is open today.");

        assert!(words.len() >= 4); // The, hospital, is, open, today
        assert!(words.iter().any(|w| w.text == "hospital"));
        assert!(words.iter().any(|w| w.text == "today"));
    }

    #[test]
    fn test_protected_region_skip() {
        let optimizer = setup_test_optimizer();
        let prompt = "Check the `hospital` variable in code.";

        let protected = optimizer.region_detector.detect(prompt);

        // `hospital` should be protected as inline code
        let word = Word {
            text: "hospital".to_string(),
            start_pos: prompt.find("hospital").unwrap(),
            end_pos: prompt.find("hospital").unwrap() + "hospital".len(),
        };

        assert!(optimizer.is_word_protected(&word, &protected, prompt));
    }

    #[test]
    fn test_full_optimization_pipeline() {
        let mut optimizer = setup_test_optimizer();

        let request = OptimizationRequest {
            prompt: "I would really appreciate if you could help with this task.".to_string(),
            output_language: Language::English,
            confidence_threshold: 0.85,
            aggressive_mode: false,
            directive_format: DirectiveFormat::Bracketed,
        };

        let result = optimizer.optimize(&request).unwrap();

        // Should have some optimizations from v0.2 patterns
        assert!(result.token_savings > 0);
        assert!(result.savings_percentage > 0.0);
    }

    #[test]
    fn test_concept_optimization() {
        let optimizer = setup_test_optimizer();
        let prompt = "Visit the hospital today.";

        let protected = optimizer.region_detector.detect(prompt);
        let optimized = optimizer.try_concept_optimization(prompt, &protected).unwrap();

        // Should keep the text (hospital is already optimal in English)
        assert!(optimized.contains("hospital"));
    }

    #[test]
    fn test_with_configuration() {
        let optimizer = setup_test_optimizer()
            .with_protection_policy(ProtectionPolicy::Aggressive)
            .with_resolution_policy(ResolutionPolicy::ExactOnly)
            .with_selection_policy(SelectionPolicy::SameLanguage { lang: "en".to_string() });

        let stats = optimizer.get_stats();
        assert_eq!(stats.db_stats.total_concepts, 1);
    }
}
