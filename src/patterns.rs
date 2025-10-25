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
        r"(?i)Thank you (so much )?in advance for .+?[.!]",
        "",
        0.96,
        "Boilerplate gratitude (complete sentence)",
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
    // New patterns for v0.2
    (
        r"(?i)I appreciate your help\.?\s*",
        "",
        0.94,
        "Gratitude boilerplate",
    ),
    (
        r"(?i)Thanks (so much )?for your (time|help)\.?\s*",
        "",
        0.95,
        "Gratitude boilerplate",
    ),
    (
        r"(?i)I hope this makes sense\.?\s*",
        "",
        0.91,
        "Uncertainty filler",
    ),
    (
        r"(?i)Let me know if you have (any )?questions\.?\s*",
        "",
        0.93,
        "Closing boilerplate",
    ),
    (
        r"(?i)Feel free to (ask|reach out)\.?\s*",
        "",
        0.92,
        "Permission boilerplate",
    ),
    (
        r"(?i)Any help would be (greatly )?appreciated\.?\s*",
        "",
        0.94,
        "Request boilerplate",
    ),
    (
        r"(?i)I('m| am) having trouble with\s*",
        "",
        0.90,
        "Problem statement prefix",
    ),
    (
        r"(?i)Can you help me (with )?\s*",
        "",
        0.93,
        "Help request prefix",
    ),
    (
        r"(?i)\bplease\b\s+",
        "",
        0.88,
        "Politeness filler (standalone)",
    ),
    (
        r"(?i)\bkindly\b\s+",
        "",
        0.85,
        "Politeness filler",
    ),
];

/// Filler words that can usually be removed
pub static FILLER_WORDS: &[(&str, f64, &str)] = &[
    (r"(?i)\breally\b", 0.88, "Intensity modifier with minimal semantic value"),
    (r"(?i)\bvery\b", 0.85, "Intensity modifier, often redundant"),
    (r"(?i)\bquite\b", 0.87, "Vague intensity modifier"),
    (r"(?i)\bjust\b", 0.82, "Minimizer, often unnecessary"),
    (r"(?i)\bactually\b", 0.89, "Filler word"),
    (r"(?i)\bbasically\b", 0.90, "Approximation filler"),
    (r"(?i)\bessentially\b", 0.89, "Approximation filler"),
    (r"(?i)\bdefinitely\b", 0.86, "Emphasis filler"),
    (r"(?i)\babsolutely\b", 0.87, "Emphasis filler"),
    (r"(?i)\bcertainly\b", 0.85, "Emphasis filler"),
    (r"(?i)\bprobably\b", 0.80, "Hedge word"),
    (r"(?i)\bmaybe\b", 0.78, "Hedge word"),
    // New filler words for v0.2
    (r"(?i)\bcarefully\b", 0.83, "Manner adverb, often implicit"),
    (r"(?i)\balso\b", 0.81, "Additive conjunction, often redundant"),
    (r"(?i)\bfurthermore\b", 0.84, "Formal transition word"),
    (r"(?i)\bmoreover\b", 0.84, "Formal transition word"),
    (r"(?i)\bindeed\b", 0.86, "Emphatic filler"),
    (r"(?i)\bin fact\b", 0.85, "Emphatic phrase"),
    (r"(?i)\bclearly\b", 0.87, "Obviousness marker"),
    (r"(?i)\bobviously\b", 0.88, "Obviousness marker"),
    (r"(?i)\bsimply\b", 0.84, "Minimizer filler"),
    (r"(?i)\bmerely\b", 0.83, "Minimizer filler"),
    (r"(?i)\bsomewhat\b", 0.82, "Hedge word"),
    (r"(?i)\brather\b", 0.80, "Hedge word"),
    (r"(?i)\bpotentially\b", 0.81, "Hedge word"),
    (r"(?i)\bpossibly\b", 0.82, "Hedge word"),
    (r"(?i)\bgenerally\b", 0.83, "Generalization filler"),
    (r"(?i)\bspecifically\b", 0.79, "Specificity marker (may be important)"),
    (r"(?i)\bparticularly\b", 0.80, "Specificity marker (may be important)"),
    (r"(?i)\bespecially\b", 0.81, "Emphasis marker"),
    (r"(?i)\bliterally\b", 0.89, "Overused intensifier"),
];

/// Instruction compression patterns - verbose instructions to imperatives
/// (pattern, replacement, confidence, reasoning)
pub static INSTRUCTION_PATTERNS: &[(&str, &str, f64, &str)] = &[
    (r"(?i)I want you to\s+", "", 0.92, "Verbose instruction prefix"),
    (r"(?i)I would like you to\s+", "", 0.91, "Verbose instruction prefix"),
    (r"(?i)I need you to\s+", "", 0.93, "Direct instruction prefix"),
    (r"(?i)I would also like you to\s+", "", 0.91, "Verbose continuation"),
    (r"(?i)take the time to\s+", "", 0.94, "Verbose padding"),
    (r"(?i)carefully\s+", "", 0.83, "Implicit in technical tasks"),
];

/// Redundant phrase consolidation
/// (pattern, replacement, confidence, reasoning)
pub static REDUNDANT_PHRASES: &[(&str, &str, f64, &str)] = &[
    // Redundant qualifiers
    (r"(?i)very\s+detailed\s+and\s+thorough", "detailed", 0.92, "Redundant qualifiers"),
    (r"(?i)detailed\s+and\s+thorough", "detailed", 0.91, "Redundant qualifiers"),

    // Synonym pairs
    (r"(?i)problems?\s+(or|and)\s+issues", "issues", 0.89, "Synonyms"),
    (r"(?i)bugs?\s+(or|and)\s+issues", "bugs", 0.88, "Synonyms"),
    (r"(?i)improve(d)?\s+or\s+optimize(d)?", "optimized", 0.90, "Optimize is subset of improve"),

    // Implied context
    (r"(?i)that\s+I'?m\s+working\s+on", "", 0.87, "Implied context"),
    (r"(?i)that\s+you\s+might\s+find", "", 0.86, "Implied action"),
    (r"(?i)this\s+code\s+snippet", "this code", 0.88, "Redundant 'snippet'"),
    (r"(?i)any\s+potential\s+", "", 0.85, "Redundant qualifiers"),

    // Conjunction compression
    (r"(?i),?\s+and\s+why\s+it\s+was\s+implemented", ", why implemented", 0.87, "Concise phrasing"),
    (r"(?i)how\s+it\s+works,?\s+and\s+why", "how/why", 0.86, "Conjunction slash"),

    // Verbose phrases
    (r"(?i)provide\s+detailed\s+suggestions\s+on\s+how\s+to\s+fix", "suggest fixes for", 0.89, "Concise phrasing"),
    (r"(?i)If\s+you\s+find\s+any\s+", "For any ", 0.84, "Passive conditional"),
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
/// These are chosen for single unambiguous meanings that LLMs understand clearly
pub static MANDARIN_SUBSTITUTIONS: &[(&str, &str, usize, usize, f64, &str)] = &[
    // Core instructions - single unambiguous meanings
    (
        "analyze",
        "分析",
        1,
        2,
        0.94,
        "Analyze/examine - precise meaning",
    ),
    (
        "explain",
        "解释",
        1,
        2,
        0.94,
        "Explain - clear single meaning",
    ),
    (
        "identify",
        "识别",
        1,
        2,
        0.93,
        "Identify/recognize - unambiguous",
    ),
    (
        "provide",
        "提供",
        1,
        2,
        0.92,
        "Provide/supply - clear intent",
    ),
    (
        "suggest",
        "建议",
        1,
        2,
        0.93,
        "Suggest/recommend - single meaning",
    ),
    (
        "verify",
        "验证",
        1,
        2,
        0.94,
        "Verify/validate - precise",
    ),

    // Quality modifiers - unambiguous instructions
    (
        "in detail",
        "详细",
        2,
        2,
        0.91,
        "In detail/detailed - clear",
    ),
    (
        "detailed",
        "详细",
        1,
        2,
        0.90,
        "Detailed - saves on multi-byte encoding",
    ),
    (
        "thorough",
        "彻底",
        1,
        2,
        0.89,
        "Thorough/complete - unambiguous",
    ),
    (
        "comprehensive",
        "全面",
        1,
        2,
        0.90,
        "Comprehensive - clear meaning",
    ),

    // Action qualifiers
    (
        "step by step",
        "逐步",
        3,
        2,
        0.92,
        "Step by step - sequential, unambiguous",
    ),
    (
        "carefully",
        "仔细",
        1,
        2,
        0.88,
        "Carefully - single meaning",
    ),

    // Common phrases with savings
    (
        "best practices",
        "最佳实践",
        2,
        4,
        0.91,
        "Best practices - technical term, clear",
    ),
    (
        "performance",
        "性能",
        1,
        2,
        0.93,
        "Performance - technical, unambiguous",
    ),
    (
        "optimization",
        "优化",
        1,
        2,
        0.93,
        "Optimization - clear technical meaning",
    ),
    (
        "implementation",
        "实现",
        1,
        2,
        0.92,
        "Implementation - technical, precise",
    ),
    (
        "issues",
        "问题",
        1,
        2,
        0.92,
        "Issues/problems - clear",
    ),
    (
        "bugs",
        "错误",
        1,
        2,
        0.93,
        "Bugs/errors - technical, unambiguous",
    ),
    (
        "code",
        "代码",
        1,
        2,
        0.94,
        "Code - technical term, precise",
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
        detected.extend(self.detect_instructions(text));
        detected.extend(self.detect_redundant_phrases(text));
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

    /// Detect instruction compression opportunities
    fn detect_instructions(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for (pattern_str, replacement, confidence, reasoning) in INSTRUCTION_PATTERNS {
            if let Ok(regex) = Regex::new(pattern_str) {
                for mat in regex.find_iter(text) {
                    detected.push(DetectedPattern {
                        pattern_type: OptimizationType::InstructionCompression,
                        original_text: mat.as_str().to_string(),
                        optimized_text: replacement.to_string(),
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        base_confidence: *confidence,
                        reasoning: reasoning.to_string(),
                    });
                }
            }
        }

        detected
    }

    /// Detect redundant phrases
    fn detect_redundant_phrases(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for (pattern_str, replacement, confidence, reasoning) in REDUNDANT_PHRASES {
            if let Ok(regex) = Regex::new(pattern_str) {
                for mat in regex.find_iter(text) {
                    detected.push(DetectedPattern {
                        pattern_type: OptimizationType::FormatConsolidation,
                        original_text: mat.as_str().to_string(),
                        optimized_text: replacement.to_string(),
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        base_confidence: *confidence,
                        reasoning: reasoning.to_string(),
                    });
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
