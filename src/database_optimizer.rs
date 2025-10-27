/// Database-backed Optimizer
/// Similar to Optimizer but loads patterns from SQLite database

use crate::confidence::{extract_context, ConfidenceCalculator};
use crate::database::Database;
use crate::database_pattern_detector::DatabasePatternDetector;
use crate::models::{
    DirectiveFormat, Language, Optimization, OptimizationRequest, OptimizationResult,
};
use crate::tokenizer::Tokenizer;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

/// Database-backed optimization engine
pub struct DatabaseOptimizer {
    detector: DatabasePatternDetector,
    calculator: ConfidenceCalculator,
    tokenizer: Tokenizer,
    db: Arc<Database>,
}

impl DatabaseOptimizer {
    /// Create new optimizer with database-backed patterns
    pub fn new(
        db: Arc<Database>,
        calculator: ConfidenceCalculator,
        tokenizer: Tokenizer,
    ) -> Result<Self> {
        let detector = DatabasePatternDetector::new(db.clone())?;

        Ok(Self {
            detector,
            calculator,
            tokenizer,
            db,
        })
    }

    /// Create optimizer with minimum confidence threshold
    pub fn with_confidence(
        db: Arc<Database>,
        calculator: ConfidenceCalculator,
        tokenizer: Tokenizer,
        min_confidence: f64,
    ) -> Result<Self> {
        let detector = DatabasePatternDetector::with_confidence(db.clone(), min_confidence)?;

        Ok(Self {
            detector,
            calculator,
            tokenizer,
            db,
        })
    }

    /// Optimize a prompt according to the request
    pub fn optimize(&mut self, request: &OptimizationRequest) -> Result<OptimizationResult> {
        let original_prompt = &request.prompt;
        let original_tokens = self.tokenizer.count_tokens(original_prompt);

        // Detect all patterns
        let detected = self.detector.detect_all(original_prompt);

        // Calculate confidence for each pattern
        let mut optimizations: Vec<Optimization> = Vec::new();

        for pattern in detected {
            let context = extract_context(
                original_prompt,
                pattern.start_pos,
                pattern.end_pos,
                50, // context window
            );

            let confidence = self.calculator.calculate_confidence_with_mode(
                &pattern,
                &context,
                request.aggressive_mode,
            );

            // Calculate token savings for this optimization
            let token_savings = self
                .tokenizer
                .estimate_savings(&pattern.original_text, &pattern.optimized_text);

            // Adjust threshold based on mode
            let min_confidence = if request.aggressive_mode { 0.4 } else { 0.5 };
            let auto_apply_threshold = if request.aggressive_mode {
                0.70
            } else {
                request.confidence_threshold
            };

            // Only include if meets minimum confidence and saves tokens
            if confidence.final_confidence >= min_confidence && token_savings > 0 {
                let requires_review = confidence.final_confidence < auto_apply_threshold;

                optimizations.push(Optimization {
                    id: Uuid::new_v4().to_string(),
                    optimization_type: pattern.pattern_type,
                    original_text: pattern.original_text,
                    optimized_text: pattern.optimized_text,
                    token_savings,
                    confidence,
                    requires_review,
                    reasoning: pattern.reasoning,
                    start_pos: pattern.start_pos,
                    end_pos: pattern.end_pos,
                });
            }
        }

        // Resolve conflicts (overlapping optimizations)
        let optimizations = self.resolve_conflicts(optimizations);

        // Split into auto-apply and requires-review
        let (auto_apply, requires_review): (Vec<_>, Vec<_>) = optimizations
            .into_iter()
            .partition(|opt| !opt.requires_review);

        // Apply auto-approved optimizations
        let mut optimized_prompt = self.apply_optimizations(original_prompt, &auto_apply);

        // Add output language directive
        optimized_prompt = self.add_language_directive(
            &optimized_prompt,
            &request.output_language,
            &request.directive_format,
        );

        let optimized_tokens = self.tokenizer.count_tokens(&optimized_prompt);
        let token_savings = original_tokens as i64 - optimized_tokens as i64;
        let savings_percentage = if original_tokens > 0 {
            (token_savings as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        Ok(OptimizationResult {
            original_prompt: original_prompt.to_string(),
            optimized_prompt,
            original_tokens,
            optimized_tokens,
            token_savings,
            savings_percentage,
            optimizations: auto_apply,
            requires_review,
            output_language: request.output_language.clone(),
        })
    }

    /// Resolve overlapping optimizations
    fn resolve_conflicts(&self, optimizations: Vec<Optimization>) -> Vec<Optimization> {
        let mut resolved = Vec::new();
        let mut covered_ranges: Vec<(usize, usize)> = Vec::new();

        // Sort by confidence (highest first)
        let mut sorted = optimizations;
        sorted.sort_by(|a, b| {
            b.confidence
                .final_confidence
                .partial_cmp(&a.confidence.final_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for opt in sorted {
            let range = (opt.start_pos, opt.end_pos);

            // Check if this optimization overlaps with any accepted optimization
            let overlaps = covered_ranges
                .iter()
                .any(|(start, end)| range.0 < *end && range.1 > *start);

            if !overlaps {
                covered_ranges.push(range);
                resolved.push(opt);
            }
        }

        // Re-sort by position
        resolved.sort_by_key(|opt| opt.start_pos);
        resolved
    }

    /// Apply optimizations to text
    fn apply_optimizations(&self, text: &str, optimizations: &[Optimization]) -> String {
        let mut result = text.to_string();
        let mut offset: i64 = 0;

        // Optimizations should be sorted by position
        for opt in optimizations {
            let start = (opt.start_pos as i64 + offset) as usize;
            let end = (opt.end_pos as i64 + offset) as usize;

            if start <= result.len() && end <= result.len() && start <= end {
                result.replace_range(start..end, &opt.optimized_text);

                // Update offset for next optimization
                let original_len = opt.end_pos - opt.start_pos;
                let new_len = opt.optimized_text.len();
                offset += new_len as i64 - original_len as i64;
            }
        }

        // Clean up whitespace
        self.clean_whitespace(&result)
    }

    /// Clean whitespace and formatting
    fn clean_whitespace(&self, text: &str) -> String {
        use regex::Regex;

        let mut result = text.to_string();

        // Remove excessive newlines
        let newline_re = Regex::new(r"\n\n\n+").unwrap();
        result = newline_re.replace_all(&result, "\n\n").to_string();

        // Collapse multiple spaces
        let space_re = Regex::new(r"  +").unwrap();
        result = space_re.replace_all(&result, " ").to_string();

        // Remove spaces before punctuation
        let punct_re = Regex::new(r" ([.,!?;:])").unwrap();
        result = punct_re.replace_all(&result, "$1").to_string();

        // Capitalize sentences
        self.capitalize_sentences(&result)
    }

    /// Capitalize first letter after sentence-ending punctuation
    fn capitalize_sentences(&self, text: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in text.chars() {
            if capitalize_next && ch.is_alphabetic() {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
                if ch == '.' || ch == '!' || ch == '?' {
                    capitalize_next = true;
                }
            }
        }

        result
    }

    /// Add language directive to prompt
    fn add_language_directive(
        &self,
        prompt: &str,
        language: &Language,
        format: &DirectiveFormat,
    ) -> String {
        let lang_str = match language {
            Language::English => "english",
            Language::Mandarin => "mandarin",
        };

        let directive = match format {
            DirectiveFormat::Bracketed => format!("[output_language: {}]", lang_str),
            DirectiveFormat::Instructive => format!("Respond in {}.", lang_str),
            DirectiveFormat::Xml => {
                format!("<output_language>{}</output_language>", lang_str)
            }
            DirectiveFormat::Natural => {
                format!("Please respond to me in {}.", lang_str)
            }
        };

        format!("{}\n\n{}", prompt.trim(), directive)
    }

    /// Reload patterns from database
    pub fn reload_patterns(&mut self) -> Result<()> {
        self.detector.reload_patterns()
    }

    /// Get database reference
    pub fn database(&self) -> &Arc<Database> {
        &self.db
    }

    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.detector.pattern_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Language;

    #[test]
    fn test_database_optimizer() {
        let db = Database::in_memory().unwrap();

        // Insert test patterns
        db.connection()
            .execute(
                "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
                 VALUES ('boilerplate', '(?i)I would really appreciate', '', 0.95, 'Test')",
                [],
            )
            .unwrap();

        let tokenizer = Tokenizer::new().unwrap();
        let calculator = ConfidenceCalculator::default();

        let mut optimizer =
            DatabaseOptimizer::new(Arc::new(db), calculator, tokenizer).unwrap();

        let request = OptimizationRequest {
            prompt: "I would really appreciate your help with this.".to_string(),
            output_language: Language::English,
            confidence_threshold: 0.85,
            aggressive_mode: false,
            directive_format: DirectiveFormat::Bracketed,
        };

        let result = optimizer.optimize(&request).unwrap();

        assert!(result.token_savings > 0);
        assert!(result.savings_percentage > 0.0);
    }
}
