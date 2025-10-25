use crate::models::{OptimizationConfidence, OptimizationCorpus};
use crate::patterns::DetectedPattern;

/// Context information for confidence calculation
#[derive(Debug, Clone)]
pub struct Context {
    pub surrounding_text: String,
    pub is_technical: bool,
    pub has_code_blocks: bool,
    pub sentence_position: SentencePosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SentencePosition {
    Beginning,
    Middle,
    End,
}

/// Confidence calculator using Bayesian inference
pub struct ConfidenceCalculator {
    corpus: OptimizationCorpus,
}

impl ConfidenceCalculator {
    pub fn new(corpus: OptimizationCorpus) -> Self {
        Self { corpus }
    }

    /// Calculate confidence for a detected pattern
    pub fn calculate_confidence(
        &self,
        pattern: &DetectedPattern,
        context: &Context,
    ) -> OptimizationConfidence {
        let base_confidence = pattern.base_confidence;

        // Calculate context penalty
        let context_penalty = self.assess_context_risk(pattern, context);

        // Get frequency bonus from corpus
        let frequency_bonus = self.corpus.get_frequency_bonus(&pattern.original_text);

        // Calculate semantic risk
        let semantic_risk = self.calculate_semantic_risk(pattern, context);

        OptimizationConfidence::new(
            base_confidence,
            context_penalty,
            frequency_bonus,
            semantic_risk,
        )
    }

    /// Assess risk based on context
    fn assess_context_risk(&self, pattern: &DetectedPattern, context: &Context) -> f64 {
        let mut penalty: f64 = 0.0;

        // Technical contexts may need more precision
        if context.is_technical {
            penalty += 0.05;
        }

        // Code blocks nearby increase risk
        if context.has_code_blocks {
            penalty += 0.03;
        }

        // Position matters for some patterns
        match context.sentence_position {
            SentencePosition::Beginning => {
                // Boilerplate at beginning is safer to remove
                if matches!(
                    pattern.pattern_type,
                    crate::models::OptimizationType::BoilerplateRemoval
                ) {
                    penalty -= 0.02;
                }
            }
            SentencePosition::Middle => {
                // Middle positions need more care
                penalty += 0.05;
            }
            SentencePosition::End => {
                // End positions might be important
                penalty += 0.03;
            }
        }

        // Ambiguous surrounding text increases risk
        if self.is_ambiguous_context(&context.surrounding_text) {
            penalty += 0.10;
        }

        penalty.clamp(0.0, 0.5)
    }

    /// Calculate semantic risk of losing meaning
    fn calculate_semantic_risk(&self, pattern: &DetectedPattern, context: &Context) -> f64 {
        let mut risk: f64 = 0.0;

        // Empty replacements have higher risk if not pure boilerplate
        if pattern.optimized_text.is_empty() {
            match pattern.pattern_type {
                crate::models::OptimizationType::BoilerplateRemoval => risk += 0.02,
                crate::models::OptimizationType::FillerRemoval => risk += 0.05,
                _ => risk += 0.15,
            }
        }

        // Very short original text might be important
        if pattern.original_text.len() < 5 {
            risk += 0.10;
        }

        // Mandarin substitution has cultural/comprehension risk
        if matches!(
            pattern.pattern_type,
            crate::models::OptimizationType::MandarinSubstitution
        ) {
            risk += 0.08;
        }

        // Synonym consolidation needs careful analysis
        if matches!(
            pattern.pattern_type,
            crate::models::OptimizationType::SynonymConsolidation
        ) {
            // If replacing multiple words with one, risk is higher
            let original_words = pattern.original_text.split_whitespace().count();
            let optimized_words = pattern.optimized_text.split_whitespace().count();
            if original_words > optimized_words + 1 {
                risk += 0.12;
            }
        }

        // Technical context increases semantic risk
        if context.is_technical {
            risk += 0.05;
        }

        risk.clamp(0.0, 0.5)
    }

    /// Check if context is ambiguous
    fn is_ambiguous_context(&self, text: &str) -> bool {
        // Look for indicators of ambiguity
        let ambiguity_markers = [
            "might",
            "could",
            "possibly",
            "perhaps",
            "seems",
            "appears",
            "unclear",
            "ambiguous",
        ];

        let text_lower = text.to_lowercase();
        ambiguity_markers.iter().any(|marker| text_lower.contains(marker))
    }

    /// Update corpus with feedback
    pub fn update_corpus(&mut self, pattern_text: &str, accepted: bool, token_savings: i64) {
        self.corpus.update_priors(pattern_text, accepted, token_savings);
    }

    /// Get reference to corpus
    pub fn corpus(&self) -> &OptimizationCorpus {
        &self.corpus
    }

    /// Get mutable reference to corpus
    pub fn corpus_mut(&mut self) -> &mut OptimizationCorpus {
        &mut self.corpus
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new(OptimizationCorpus::default())
    }
}

/// Extract context around a pattern match
pub fn extract_context(text: &str, start_pos: usize, end_pos: usize, window: usize) -> Context {
    let context_start = start_pos.saturating_sub(window);
    let context_end = (end_pos + window).min(text.len());

    let surrounding_text = text[context_start..context_end].to_string();

    // Determine if technical based on keywords
    let is_technical = is_technical_text(&surrounding_text);

    // Check for code blocks
    let has_code_blocks = surrounding_text.contains("```") || surrounding_text.contains("    ");

    // Determine sentence position
    let sentence_position = determine_position(text, start_pos);

    Context {
        surrounding_text,
        is_technical,
        has_code_blocks,
        sentence_position,
    }
}

/// Determine if text is technical
fn is_technical_text(text: &str) -> bool {
    let technical_keywords = [
        "function",
        "class",
        "algorithm",
        "code",
        "variable",
        "method",
        "API",
        "database",
        "server",
        "client",
    ];

    let text_lower = text.to_lowercase();
    technical_keywords.iter().filter(|kw| text_lower.contains(*kw)).count() >= 2
}

/// Determine position within sentence/text
fn determine_position(text: &str, pos: usize) -> SentencePosition {
    // Find sentence boundaries
    let before = &text[..pos];
    let after = &text[pos..];

    let is_start = before.trim_start().is_empty()
        || before.trim_end().ends_with('.')
        || before.trim_end().ends_with('!')
        || before.trim_end().ends_with('?');

    let is_end = after.trim_start().is_empty()
        || after.trim_start().starts_with('.')
        || after.trim_start().starts_with('!')
        || after.trim_start().starts_with('?');

    if is_start {
        SentencePosition::Beginning
    } else if is_end {
        SentencePosition::End
    } else {
        SentencePosition::Middle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OptimizationType;

    #[test]
    fn test_confidence_calculation() {
        let calculator = ConfidenceCalculator::default();

        let pattern = DetectedPattern {
            pattern_type: OptimizationType::BoilerplateRemoval,
            original_text: "I would really appreciate it if you could".to_string(),
            optimized_text: String::new(),
            start_pos: 0,
            end_pos: 41,
            base_confidence: 0.97,
            reasoning: "Common boilerplate".to_string(),
        };

        let context = Context {
            surrounding_text: "I would really appreciate it if you could help me.".to_string(),
            is_technical: false,
            has_code_blocks: false,
            sentence_position: SentencePosition::Beginning,
        };

        let confidence = calculator.calculate_confidence(&pattern, &context);

        assert!(confidence.final_confidence >= 0.9);
        assert!(confidence.final_confidence <= 1.0);
    }

    #[test]
    fn test_context_extraction() {
        let text = "This is a test. I would like help. Thank you.";
        let context = extract_context(text, 16, 32, 20);

        assert!(!context.surrounding_text.is_empty());
    }

    #[test]
    fn test_technical_detection() {
        let technical = "This function uses an algorithm to process the API.";
        let non_technical = "This is a simple request for help.";

        assert!(is_technical_text(technical));
        assert!(!is_technical_text(non_technical));
    }

    #[test]
    fn test_position_detection() {
        let text = "Hello. This is middle. End.";

        assert_eq!(determine_position(text, 0), SentencePosition::Beginning);
        assert_eq!(determine_position(text, 10), SentencePosition::Middle);
    }
}
