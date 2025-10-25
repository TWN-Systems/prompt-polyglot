use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported output languages for the optimized prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Mandarin,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

/// Format for the output language directive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectiveFormat {
    Bracketed,   // [output_language: english]
    Instructive, // "Respond in English."
    Xml,         // <output_language>english</output_language>
    Natural,     // "Please respond to me in English."
}

impl Default for DirectiveFormat {
    fn default() -> Self {
        DirectiveFormat::Bracketed
    }
}

/// Types of optimizations that can be applied
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationType {
    BoilerplateRemoval,
    SynonymConsolidation,
    FillerRemoval,
    InstructionCompression,
    MandarinSubstitution,
    FormatConsolidation,
}

/// Bayesian confidence breakdown for an optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfidence {
    pub base_confidence: f64,    // Pattern match confidence
    pub context_penalty: f64,    // Reduction based on ambiguity
    pub frequency_bonus: f64,    // Increase for common patterns
    pub semantic_risk: f64,      // Risk of meaning loss
    pub final_confidence: f64,   // Computed score (0.0-1.0)
}

impl OptimizationConfidence {
    pub fn new(
        base_confidence: f64,
        context_penalty: f64,
        frequency_bonus: f64,
        semantic_risk: f64,
    ) -> Self {
        let final_confidence = (base_confidence
            * (1.0 - context_penalty)
            * (1.0 + frequency_bonus)
            * (1.0 - semantic_risk))
            .clamp(0.0, 1.0);

        Self {
            base_confidence,
            context_penalty,
            frequency_bonus,
            semantic_risk,
            final_confidence,
        }
    }
}

/// A single optimization that can be applied to a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub id: String,
    pub optimization_type: OptimizationType,
    pub original_text: String,
    pub optimized_text: String,
    pub token_savings: i64,
    pub confidence: OptimizationConfidence,
    pub requires_review: bool,
    pub reasoning: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Request to optimize a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub prompt: String,
    pub output_language: Language,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
    #[serde(default)]
    pub aggressive_mode: bool,
    #[serde(default)]
    pub directive_format: DirectiveFormat,
}

fn default_confidence_threshold() -> f64 {
    0.85
}

/// Result of optimizing a prompt
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_prompt: String,
    pub optimized_prompt: String,
    pub original_tokens: usize,
    pub optimized_tokens: usize,
    pub token_savings: i64,
    pub savings_percentage: f64,
    pub optimizations: Vec<Optimization>,
    pub requires_review: Vec<Optimization>,
    pub output_language: Language,
}

/// Decision for a reviewed optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ReviewDecision {
    Accept,
    Reject,
    Modify { alternative: String },
}

/// A review session for low-confidence optimizations
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewSession {
    pub session_id: String,
    pub pending_optimizations: Vec<Optimization>,
    pub decisions: HashMap<String, ReviewDecision>,
}

/// Statistics for a pattern in the corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    pub occurrences: usize,
    pub successful_optimizations: usize,
    pub failed_optimizations: usize,
    pub avg_token_savings: f64,
}

impl Default for PatternStats {
    fn default() -> Self {
        Self {
            occurrences: 0,
            successful_optimizations: 0,
            failed_optimizations: 0,
            avg_token_savings: 0.0,
        }
    }
}

impl PatternStats {
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_optimizations + self.failed_optimizations;
        if total == 0 {
            0.5 // Default prior
        } else {
            self.successful_optimizations as f64 / total as f64
        }
    }

    pub fn update(&mut self, accepted: bool, token_savings: i64) {
        if accepted {
            self.successful_optimizations += 1;
        } else {
            self.failed_optimizations += 1;
        }

        // Update running average
        let total = self.successful_optimizations + self.failed_optimizations;
        let old_avg = self.avg_token_savings;
        self.avg_token_savings =
            (old_avg * (total - 1) as f64 + token_savings as f64) / total as f64;
    }
}

/// Corpus of optimization patterns with Bayesian priors
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationCorpus {
    pub patterns: HashMap<String, PatternStats>,
    pub total_optimizations: usize,
    pub success_rate: f64,
}

impl Default for OptimizationCorpus {
    fn default() -> Self {
        Self {
            patterns: HashMap::new(),
            total_optimizations: 0,
            success_rate: 0.0,
        }
    }
}

impl OptimizationCorpus {
    pub fn update_priors(&mut self, pattern: &str, accepted: bool, token_savings: i64) {
        let stats = self
            .patterns
            .entry(pattern.to_string())
            .or_insert_with(PatternStats::default);

        stats.update(accepted, token_savings);
        self.total_optimizations += 1;

        // Recalculate global success rate
        let total_successes: usize = self
            .patterns
            .values()
            .map(|s| s.successful_optimizations)
            .sum();
        let total_attempts: usize = self
            .patterns
            .values()
            .map(|s| s.successful_optimizations + s.failed_optimizations)
            .sum();

        self.success_rate = if total_attempts > 0 {
            total_successes as f64 / total_attempts as f64
        } else {
            0.0
        };
    }

    pub fn get_frequency_bonus(&self, pattern: &str) -> f64 {
        self.patterns
            .get(pattern)
            .map(|stats| (stats.occurrences as f64).log10() * 0.05)
            .unwrap_or(0.0)
    }
}

/// Configuration for the optimization system
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub optimization: OptimizationConfig,
    pub hitl: HitlConfig,
    pub patterns: PatternsConfig,
    pub bayesian: BayesianConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub confidence_threshold: f64,
    pub aggressive_mode: bool,
    pub output_language: Language,
    pub directive_format: DirectiveFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HitlConfig {
    pub enabled: bool,
    pub auto_accept_threshold: f64,
    pub batch_review: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternsConfig {
    pub boilerplate_enabled: bool,
    pub synonym_consolidation: bool,
    pub filler_removal: bool,
    pub mandarin_substitution: bool,
    pub format_consolidation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BayesianConfig {
    pub prior_corpus_path: String,
    pub update_priors_on_feedback: bool,
    pub min_confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub save_report: bool,
    pub report_format: String,
    pub show_diff: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            optimization: OptimizationConfig {
                confidence_threshold: 0.85,
                aggressive_mode: false,
                output_language: Language::English,
                directive_format: DirectiveFormat::Bracketed,
            },
            hitl: HitlConfig {
                enabled: true,
                auto_accept_threshold: 0.95,
                batch_review: false,
            },
            patterns: PatternsConfig {
                boilerplate_enabled: true,
                synonym_consolidation: true,
                filler_removal: true,
                mandarin_substitution: true,
                format_consolidation: true,
            },
            bayesian: BayesianConfig {
                prior_corpus_path: "data/priors.json".to_string(),
                update_priors_on_feedback: true,
                min_confidence: 0.50,
            },
            output: OutputConfig {
                save_report: true,
                report_format: "json".to_string(),
                show_diff: true,
            },
        }
    }
}
