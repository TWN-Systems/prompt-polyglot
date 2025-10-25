use crate::confidence::{extract_context, ConfidenceCalculator};
use crate::models::{
    DirectiveFormat, Language, Optimization, OptimizationRequest, OptimizationResult,
};
use crate::patterns::PatternDetector;
use crate::tokenizer::Tokenizer;
use anyhow::Result;
use uuid::Uuid;

/// Main optimization engine
pub struct Optimizer {
    detector: PatternDetector,
    calculator: ConfidenceCalculator,
    tokenizer: Tokenizer,
}

impl Optimizer {
    pub fn new(calculator: ConfidenceCalculator, tokenizer: Tokenizer) -> Self {
        Self {
            detector: PatternDetector::new(),
            calculator,
            tokenizer,
        }
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
            let auto_apply_threshold = if request.aggressive_mode { 0.70 } else { request.confidence_threshold };

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
            original_prompt: original_prompt.clone(),
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

    /// Resolve overlapping optimizations by selecting the best ones
    fn resolve_conflicts(&self, mut optimizations: Vec<Optimization>) -> Vec<Optimization> {
        // Sort by start position
        optimizations.sort_by_key(|opt| opt.start_pos);

        let mut resolved: Vec<Optimization> = Vec::new();
        let mut last_end = 0;

        for opt in optimizations {
            // Skip if overlaps with previous
            if opt.start_pos < last_end {
                // Keep the one with higher confidence or more token savings
                if let Some(last) = resolved.last_mut() {
                    if opt.confidence.final_confidence > last.confidence.final_confidence
                        || (opt.confidence.final_confidence == last.confidence.final_confidence
                            && opt.token_savings > last.token_savings)
                    {
                        // Replace with better optimization
                        *last = opt.clone();
                        last_end = opt.end_pos;
                    }
                }
            } else {
                last_end = opt.end_pos;
                resolved.push(opt);
            }
        }

        resolved
    }

    /// Apply optimizations to text
    fn apply_optimizations(&self, text: &str, optimizations: &[Optimization]) -> String {
        if optimizations.is_empty() {
            return text.to_string();
        }

        let mut result = String::new();
        let mut last_pos = 0;

        for opt in optimizations {
            // Add text before this optimization
            result.push_str(&text[last_pos..opt.start_pos]);

            // Add optimized text
            result.push_str(&opt.optimized_text);

            last_pos = opt.end_pos;
        }

        // Add remaining text
        result.push_str(&text[last_pos..]);

        // Clean up extra whitespace
        self.clean_whitespace(&result)
    }

    /// Clean up extra whitespace
    fn clean_whitespace(&self, text: &str) -> String {
        // Remove multiple spaces
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

        // Clean up punctuation spacing
        let text = text.replace(" .", ".").replace(" ,", ",");
        let text = text.replace(" !", "!").replace(" ?", "?");

        let text = text.trim().to_string();

        // Capitalize sentence starts
        self.capitalize_sentences(&text)
    }

    /// Capitalize the first letter after sentence boundaries
    fn capitalize_sentences(&self, text: &str) -> String {
        if text.is_empty() {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        let mut capitalize_next = true;  // First character should be capitalized

        while let Some(ch) = chars.next() {
            if capitalize_next && ch.is_alphabetic() {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);

                // Set flag to capitalize after sentence boundaries
                if matches!(ch, '.' | '!' | '?') {
                    // Skip whitespace after punctuation
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_whitespace() {
                            result.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
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
            DirectiveFormat::Instructive => format!("Respond in {}.", Self::capitalize(lang_str)),
            DirectiveFormat::Xml => {
                format!("<output_language>{}</output_language>", lang_str)
            }
            DirectiveFormat::Natural => {
                format!("Please respond to me in {}.", Self::capitalize(lang_str))
            }
        };

        format!("{}\n\n{}", prompt.trim(), directive)
    }

    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Get reference to confidence calculator
    pub fn calculator(&self) -> &ConfidenceCalculator {
        &self.calculator
    }

    /// Get mutable reference to confidence calculator
    pub fn calculator_mut(&mut self) -> &mut ConfidenceCalculator {
        &mut self.calculator
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(
            ConfidenceCalculator::default(),
            Tokenizer::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Language;

    #[test]
    fn test_basic_optimization() {
        let mut optimizer = Optimizer::default();

        let request = OptimizationRequest {
            prompt: "I would really appreciate it if you could please help me with this task."
                .to_string(),
            output_language: Language::English,
            confidence_threshold: 0.85,
            aggressive_mode: false,
            directive_format: DirectiveFormat::Bracketed,
        };

        let result = optimizer.optimize(&request).unwrap();

        assert!(result.token_savings > 0);
        assert!(result.savings_percentage > 0.0);
        assert!(!result.optimized_prompt.is_empty());
    }

    #[test]
    fn test_conflict_resolution() {
        let optimizer = Optimizer::default();

        let opt1 = Optimization {
            id: "1".to_string(),
            optimization_type: crate::models::OptimizationType::BoilerplateRemoval,
            original_text: "test".to_string(),
            optimized_text: "".to_string(),
            token_savings: 1,
            confidence: crate::models::OptimizationConfidence::new(0.9, 0.0, 0.0, 0.0),
            requires_review: false,
            reasoning: "test".to_string(),
            start_pos: 0,
            end_pos: 4,
        };

        let opt2 = Optimization {
            id: "2".to_string(),
            optimization_type: crate::models::OptimizationType::FillerRemoval,
            original_text: "test overlap".to_string(),
            optimized_text: "overlap".to_string(),
            token_savings: 2,
            confidence: crate::models::OptimizationConfidence::new(0.95, 0.0, 0.0, 0.0),
            requires_review: false,
            reasoning: "test".to_string(),
            start_pos: 2,
            end_pos: 14,
        };

        let resolved = optimizer.resolve_conflicts(vec![opt1, opt2]);

        // Should keep the higher confidence one
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].confidence.final_confidence >= 0.9);
    }

    #[test]
    fn test_language_directive() {
        let optimizer = Optimizer::default();

        let text = "Test prompt";

        let result = optimizer.add_language_directive(
            text,
            &Language::English,
            &DirectiveFormat::Bracketed,
        );

        assert!(result.contains("[output_language: english]"));
    }

    #[test]
    fn test_capitalize_sentences() {
        let optimizer = Optimizer::default();

        assert_eq!(
            optimizer.capitalize_sentences("hello. world"),
            "Hello. World"
        );
        assert_eq!(
            optimizer.capitalize_sentences("test! another. one?"),
            "Test! Another. One?"
        );
        assert_eq!(
            optimizer.capitalize_sentences("already Capitalized."),
            "Already Capitalized."
        );
    }

    #[test]
    fn test_no_orphaned_phrases() {
        let mut optimizer = Optimizer::default();

        let request = OptimizationRequest {
            prompt: "Thank you so much in advance for your help with this!".to_string(),
            output_language: Language::English,
            confidence_threshold: 0.85,
            aggressive_mode: false,
            directive_format: DirectiveFormat::Bracketed,
        };

        let result = optimizer.optimize(&request).unwrap();

        // Should not contain orphaned "for your help with this!"
        assert!(!result.optimized_prompt.contains("for your help"));
    }
}
