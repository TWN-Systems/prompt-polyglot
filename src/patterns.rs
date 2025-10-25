use crate::models::OptimizationType;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// A pattern for detection
#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: OptimizationType,
    pub regex: Regex,
    pub replacement: String,
    pub base_confidence: f64,
    pub reasoning: String,
}

/// A detected pattern match in text
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub pattern_type: OptimizationType,
    pub original_text: String,
    pub optimized_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub base_confidence: f64,
    pub reasoning: String,
}

/// Boilerplate patterns with high confidence
pub static BOILERPLATE_PATTERNS: &[(&str, &str, f64, &str)] = &[
    (
        r"(?i)I would (really )?appreciate (it )?if you could\s*",
        "",
        0.97,
        "Common politeness boilerplate with no semantic value",
    ),
    (
        r"(?i)Please make sure to\s*",
        "",
        0.95,
        "Redundant instruction emphasis",
    ),
    (
        r"(?i)If you don't mind,?\s*",
        "",
        0.94,
        "Politeness filler",
    ),
    (
        r"(?i)Thank you (so much )?in advance\s*",
        "",
        0.96,
        "Boilerplate gratitude",
    ),
    (
        r"(?i)I('m| am) looking for help with\s*",
        "",
        0.93,
        "Verbose help request prefix",
    ),
    (
        r"(?i)Could you please\s*",
        "",
        0.95,
        "Polite request prefix",
    ),
    (
        r"(?i)Would you mind\s*",
        "",
        0.94,
        "Polite request prefix",
    ),
    (
        r"(?i)I would like you to\s*",
        "",
        0.96,
        "Verbose instruction prefix",
    ),
    (
        r"(?i)It would be great if\s*",
        "",
        0.93,
        "Polite request prefix",
    ),
    (
        r"(?i)I need you to\s*",
        "",
        0.92,
        "Direct instruction prefix",
    ),
    (
        r"(?i)I was wondering if\s*",
        "",
        0.91,
        "Indirect question prefix",
    ),
    (
        r"(?i)I hope you('re| are) doing well\.?\s*",
        "",
        0.95,
        "Greeting boilerplate",
    ),
    (
        r"(?i)Hello!?\s*",
        "",
        0.90,
        "Greeting (unnecessary for prompts)",
    ),
];

/// Filler words that can usually be removed
pub static FILLER_WORDS: &[(&str, f64, &str)] = &[
    (r"\breally\b", 0.88, "Intensity modifier with minimal semantic value"),
    (r"\bvery\b", 0.85, "Intensity modifier, often redundant"),
    (r"\bquite\b", 0.87, "Vague intensity modifier"),
    (r"\bjust\b", 0.82, "Minimizer, often unnecessary"),
    (r"\bactually\b", 0.89, "Filler word"),
    (r"\bbasically\b", 0.90, "Approximation filler"),
    (r"\bessentially\b", 0.89, "Approximation filler"),
    (r"\bdefinitely\b", 0.86, "Emphasis filler"),
    (r"\babsolutely\b", 0.87, "Emphasis filler"),
    (r"\bcertainly\b", 0.85, "Emphasis filler"),
    (r"\bprobably\b", 0.80, "Hedge word"),
    (r"\bmaybe\b", 0.78, "Hedge word"),
];

/// Synonym pairs where consolidation saves tokens
/// (preferred_term, alternatives, base_confidence, reasoning)
pub static SYNONYM_PAIRS: &[(&str, &[&str], f64, &str)] = &[
    (
        "analyze",
        &["look at", "examine", "inspect", "review"],
        0.89,
        "Consolidate to stronger verb 'analyze'",
    ),
    (
        "research",
        &["look into", "investigate"],
        0.88,
        "Consolidate to 'research'",
    ),
    (
        "verify",
        &["check", "confirm"],
        0.85,
        "Consolidate to 'verify'",
    ),
    (
        "improve",
        &["enhance", "optimize"],
        0.87,
        "Consolidate to 'improve'",
    ),
    (
        "explain",
        &["describe", "clarify"],
        0.84,
        "Consolidate to 'explain'",
    ),
    (
        "provide",
        &["give", "supply"],
        0.86,
        "Consolidate to 'provide'",
    ),
    (
        "create",
        &["make", "build", "generate"],
        0.83,
        "Consolidate to 'create'",
    ),
    (
        "identify",
        &["find", "locate", "detect"],
        0.82,
        "Consolidate to 'identify'",
    ),
];

/// Mandarin substitutions that save tokens
/// (english_phrase, mandarin_equivalent, en_tokens, zh_tokens, base_confidence, reasoning)
pub static MANDARIN_SUBSTITUTIONS: &[(&str, &str, usize, usize, f64, &str)] = &[
    (
        "Be thorough and detailed",
        "要详细",
        5,
        3,
        0.86,
        "Common instruction for thoroughness",
    ),
    (
        "Step by step",
        "逐步",
        3,
        1,
        0.89,
        "Sequential instruction",
    ),
    (
        "Make sure to",
        "确保",
        3,
        2,
        0.84,
        "Ensure/make sure",
    ),
    (
        "Focus on",
        "专注于",
        2,
        3,
        0.87,
        "Attention directive",
    ),
    (
        "Pay attention to",
        "注意",
        3,
        2,
        0.88,
        "Attention directive",
    ),
    (
        "Be comprehensive",
        "要全面",
        2,
        3,
        0.85,
        "Comprehensiveness instruction",
    ),
    (
        "In detail",
        "详细地",
        2,
        3,
        0.86,
        "Detail modifier",
    ),
];

lazy_static! {
    /// Compiled boilerplate patterns
    pub static ref BOILERPLATE_REGEXES: Vec<Pattern> = {
        BOILERPLATE_PATTERNS
            .iter()
            .filter_map(|(pattern, replacement, confidence, reasoning)| {
                Regex::new(pattern).ok().map(|regex| Pattern {
                    pattern_type: OptimizationType::BoilerplateRemoval,
                    regex,
                    replacement: replacement.to_string(),
                    base_confidence: *confidence,
                    reasoning: reasoning.to_string(),
                })
            })
            .collect()
    };

    /// Compiled filler word patterns
    pub static ref FILLER_REGEXES: Vec<Pattern> = {
        FILLER_WORDS
            .iter()
            .filter_map(|(pattern, confidence, reasoning)| {
                Regex::new(pattern).ok().map(|regex| Pattern {
                    pattern_type: OptimizationType::FillerRemoval,
                    regex,
                    replacement: String::new(),
                    base_confidence: *confidence,
                    reasoning: reasoning.to_string(),
                })
            })
            .collect()
    };

    /// Mandarin substitution lookup
    pub static ref MANDARIN_MAP: HashMap<String, (String, f64, String)> = {
        MANDARIN_SUBSTITUTIONS
            .iter()
            .map(|(en, zh, _en_tok, _zh_tok, conf, reasoning)| {
                (
                    en.to_lowercase(),
                    (zh.to_string(), *conf, reasoning.to_string()),
                )
            })
            .collect()
    };
}

/// Pattern detector engine
pub struct PatternDetector;

impl PatternDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect all patterns in text
    pub fn detect_all(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        detected.extend(self.detect_boilerplate(text));
        detected.extend(self.detect_fillers(text));
        detected.extend(self.detect_synonyms(text));
        detected.extend(self.detect_mandarin(text));

        // Sort by position to handle overlaps later
        detected.sort_by_key(|d| d.start_pos);
        detected
    }

    /// Detect boilerplate patterns
    fn detect_boilerplate(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for pattern in BOILERPLATE_REGEXES.iter() {
            for mat in pattern.regex.find_iter(text) {
                detected.push(DetectedPattern {
                    pattern_type: OptimizationType::BoilerplateRemoval,
                    original_text: mat.as_str().to_string(),
                    optimized_text: pattern.replacement.clone(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    base_confidence: pattern.base_confidence,
                    reasoning: pattern.reasoning.clone(),
                });
            }
        }

        detected
    }

    /// Detect filler words
    fn detect_fillers(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for pattern in FILLER_REGEXES.iter() {
            for mat in pattern.regex.find_iter(text) {
                detected.push(DetectedPattern {
                    pattern_type: OptimizationType::FillerRemoval,
                    original_text: mat.as_str().to_string(),
                    optimized_text: String::new(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    base_confidence: pattern.base_confidence,
                    reasoning: pattern.reasoning.clone(),
                });
            }
        }

        detected
    }

    /// Detect synonym consolidation opportunities
    fn detect_synonyms(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();
        let text_lower = text.to_lowercase();

        for (preferred, alternatives, confidence, reasoning) in SYNONYM_PAIRS {
            // Look for patterns like "X and Y" where Y is the alternative
            for alt in *alternatives {
                // Pattern: "alternative and/or preferred" or "preferred and/or alternative"
                let patterns = [
                    format!(r"\b{}\s+and\s+{}\b", alt, preferred),
                    format!(r"\b{}\s+and\s+{}\b", preferred, alt),
                    format!(r"\b{}\s+or\s+{}\b", alt, preferred),
                    format!(r"\b{}\s+or\s+{}\b", preferred, alt),
                ];

                for pattern_str in &patterns {
                    if let Ok(regex) = Regex::new(pattern_str) {
                        for mat in regex.find_iter(&text_lower) {
                            detected.push(DetectedPattern {
                                pattern_type: OptimizationType::SynonymConsolidation,
                                original_text: text[mat.start()..mat.end()].to_string(),
                                optimized_text: preferred.to_string(),
                                start_pos: mat.start(),
                                end_pos: mat.end(),
                                base_confidence: *confidence,
                                reasoning: reasoning.to_string(),
                            });
                        }
                    }
                }
            }
        }

        detected
    }

    /// Detect Mandarin substitution opportunities
    fn detect_mandarin(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();
        let text_lower = text.to_lowercase();

        for (english, (mandarin, confidence, reasoning)) in MANDARIN_MAP.iter() {
            if let Some(pos) = text_lower.find(english) {
                detected.push(DetectedPattern {
                    pattern_type: OptimizationType::MandarinSubstitution,
                    original_text: text[pos..pos + english.len()].to_string(),
                    optimized_text: mandarin.clone(),
                    start_pos: pos,
                    end_pos: pos + english.len(),
                    base_confidence: *confidence,
                    reasoning: reasoning.clone(),
                });
            }
        }

        detected
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boilerplate_detection() {
        let detector = PatternDetector::new();
        let text = "I would really appreciate it if you could help me with this task.";

        let detected = detector.detect_boilerplate(text);
        assert!(!detected.is_empty());
        assert!(detected[0].base_confidence > 0.9);
    }

    #[test]
    fn test_filler_detection() {
        let detector = PatternDetector::new();
        let text = "This is really very important and definitely needs attention.";

        let detected = detector.detect_fillers(text);
        assert!(detected.len() >= 3); // really, very, definitely
    }

    #[test]
    fn test_synonym_detection() {
        let detector = PatternDetector::new();
        let text = "Please analyze and examine this code carefully.";

        let detected = detector.detect_synonyms(text);
        assert!(!detected.is_empty());
    }

    #[test]
    fn test_mandarin_detection() {
        let detector = PatternDetector::new();
        let text = "Be thorough and detailed in your analysis.";

        let detected = detector.detect_mandarin(text);
        assert!(!detected.is_empty());
    }
}
