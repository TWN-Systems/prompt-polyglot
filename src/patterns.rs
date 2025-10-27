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
        r"(?i)I would (also )?like you to\s*",
        "",
        0.96,
        "Verbose instruction prefix",
    ),
    (
        r"(?i)\bmake sure to\s+",
        "",
        0.94,
        "Redundant instruction (standalone)",
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

    // AGGRESSIVE v0.3: Ultra-compression patterns
    // Apply these BEFORE other patterns for better matching

    // Complete sentence compressions (most specific first)
    (r"(?i)Provide\s+a\s+(?:very\s+)?detailed\s+(?:and\s+thorough\s+)?explanation\s+of\s+what\s+(?:the\s+)?code\s+does,?\s+how\s+it\s+works,?\s+and\s+why\s+it\s+was\s+implemented(?:\s+in\s+this\s+particular\s+way)?\.?",
     "Explain: functionality, implementation, rationale.", 0.92, "Complete explanation compression"),

    (r"(?i)look\s+into\s+(?:any\s+)?(?:potential\s+)?bugs?\s+or\s+issues\s+(?:that\s+you\s+might\s+find)?,?\s+and\s+(?:also\s+)?check\s+for\s+(?:any\s+)?performance\s+problems?\s+or\s+areas\s+where\s+(?:the\s+)?code\s+could\s+be\s+improved\s+or\s+optimized\.?",
     "Identify: bugs, performance issues, improvements.", 0.91, "Combined bugs+performance compression"),

    (r"(?i)Research\s+and\s+explain\s+whether\s+(?:this\s+)?code\s+follows\s+best\s+practices\s+and\s+coding\s+standards\.?",
     "Verify best practices.", 0.90, "Research→Verify compression"),

    (r"(?i)If\s+you\s+find\s+(?:any\s+)?problems?\s+or\s+issues?,?\s+(?:please\s+)?provide\s+detailed\s+suggestions\s+on\s+how\s+to\s+fix\s+them\.?",
     "Suggest fixes.", 0.91, "Final sentence compression"),

    // Partial phrase compressions (for cases where full match doesn't work)
    (r"(?i)Provide\s+a\s+detailed\s+explanation\s+of\s+", "Explain: ", 0.89, "Verbose to colon format"),
    (r"(?i)Look\s+into\s+any\s+", "Identify ", 0.87, "Look into→Identify"),
    (r"(?i)check\s+for\s+any\s+", "", 0.86, "Redundant check phrase"),

    // Context removals
    (r"(?i)in\s+this\s+particular\s+way", "", 0.85, "Implied by context"),
    (r"(?i)that\s+you\s+might\s+find", "", 0.84, "Implied by 'look'"),
    (r"(?i)or\s+areas\s+where", "", 0.83, "Redundant qualifier"),
    (r"(?i)best\s+practices\s+and\s+coding\s+standards", "best practices", 0.87, "Redundant pair"),
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
/// IMPORTANT: Only includes PROVEN token-efficient substitutions (actual token counts from tiktoken tests)
/// Criteria: ZH tokens <= EN tokens (never makes it worse)
pub static MANDARIN_SUBSTITUTIONS: &[(&str, &str, usize, usize, f64, &str)] = &[
    // ✅ PROVEN EFFICIENT: Equal token count, maintains quality
    (
        "verify",
        "验证",
        1,
        1,
        0.94,
        "Verify - EQUAL tokens (1=1), unambiguous meaning",
    ),
    (
        "comprehensive",
        "全面",
        2,
        2,
        0.90,
        "Comprehensive - EQUAL tokens (2=2), clear meaning",
    ),
    (
        "optimization",
        "优化",
        2,
        2,
        0.93,
        "Optimization - EQUAL tokens (2=2), technical term",
    ),
    (
        "step by step",
        "逐步",
        3,
        3,
        0.92,
        "Step by step - EQUAL tokens (3=3), sequential",
    ),
    (
        "issues",
        "问题",
        1,
        1,
        0.92,
        "Issues - EQUAL tokens (1=1), clear",
    ),
    (
        "bugs",
        "错误",
        1,
        1,
        0.93,
        "Bugs - EQUAL tokens (1=1), unambiguous",
    ),
    (
        "code",
        "代码",
        1,
        1,
        0.94,
        "Code - EQUAL tokens (1=1), technical term",
    ),

    // NOTE: Removed inefficient substitutions that INCREASE token count:
    // - analyze (1→2 tokens), explain (1→2), identify (1→3), provide (1→2)
    // - suggest (1→2), detailed (2→3), thorough (2→4), carefully (2→4)
    // - best practices (2→6), performance (1→2), implementation (1→2)
    // These hurt token efficiency and were removed based on test evidence.
];

/// Structural optimizations - Units, numbers, formatting
/// Based on empirical findings: "10km" is more token-efficient than "ten kilometers"
pub static STRUCTURAL_PATTERNS: &[(&str, &str, f64, &str)] = &[
    // Units - normalize to compact form
    (
        r"\b(\d+)\s*kilometers?\b",
        "${1}km",
        0.93,
        "Normalize kilometers to km (3 tokens → 2 tokens)"
    ),
    (
        r"\b(\d+)\s*meters?\b",
        "${1}m",
        0.93,
        "Normalize meters to m"
    ),
    (
        r"\b(\d+)\s*minutes?\b",
        "${1}min",
        0.92,
        "Normalize minutes to min (3 tokens → 2 tokens)"
    ),
    (
        r"\b(\d+)\s*seconds?\b",
        "${1}s",
        0.92,
        "Normalize seconds to s"
    ),
    (
        r"\b(\d+)\s*percent\b",
        "${1}%",
        0.95,
        "Normalize percent to % (3 tokens → 2 tokens)"
    ),
    (
        r"\b(\d+)\s*dollars?\b",
        "$${1}",
        0.90,
        "Normalize dollars to $ prefix"
    ),

    // Excess whitespace and formatting
    (
        r"\n\n\n+",
        "\n\n",
        0.95,
        "Collapse excessive newlines (>2 → 2)"
    ),
    (
        r"  +",
        " ",
        0.94,
        "Collapse multiple spaces to single space"
    ),
    (
        r"={3,}",
        "",
        0.88,
        "Remove decorative separators (===)"
    ),
    (
        r"-{3,}",
        "",
        0.88,
        "Remove decorative separators (---)"
    ),
    (
        r"\*{3,}",
        "",
        0.88,
        "Remove decorative separators (***)"
    ),

    // Verbose JSON/structure keywords
    (
        r#""description":\s*"#,
        r#""desc":"#,
        0.85,
        "Shorten JSON key: description → desc"
    ),
    (
        r#""configuration":\s*"#,
        r#""config":"#,
        0.85,
        "Shorten JSON key: configuration → config"
    ),
    (
        r#""parameters":\s*"#,
        r#""params":"#,
        0.85,
        "Shorten JSON key: parameters → params"
    ),

    // Excessive punctuation
    (
        r"\.{2,}",
        ".",
        0.90,
        "Normalize ellipsis to single period"
    ),
    (
        r"!{2,}",
        "!",
        0.90,
        "Collapse multiple exclamation marks"
    ),
    (
        r"\?{2,}",
        "?",
        0.90,
        "Collapse multiple question marks"
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

    /// Compiled structural optimization patterns
    pub static ref STRUCTURAL_REGEXES: Vec<Pattern> = {
        STRUCTURAL_PATTERNS
            .iter()
            .filter_map(|(pattern, replacement, confidence, reasoning)| {
                Regex::new(pattern).ok().map(|regex| Pattern {
                    pattern_type: OptimizationType::FormatConsolidation,
                    regex,
                    replacement: replacement.to_string(),
                    base_confidence: *confidence,
                    reasoning: reasoning.to_string(),
                })
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

        detected.extend(self.detect_structural(text));
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

    /// Detect structural optimization opportunities
    fn detect_structural(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for pattern in STRUCTURAL_REGEXES.iter() {
            for mat in pattern.regex.find_iter(text) {
                let optimized = pattern.regex.replace(mat.as_str(), &pattern.replacement);
                detected.push(DetectedPattern {
                    pattern_type: OptimizationType::FormatConsolidation,
                    original_text: mat.as_str().to_string(),
                    optimized_text: optimized.to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    base_confidence: pattern.base_confidence,
                    reasoning: pattern.reasoning.clone(),
                });
            }
        }

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
        // Use a word that's actually in our selective Mandarin list
        let text = "Please verify the code for bugs and issues.";

        let detected = detector.detect_mandarin(text);
        // Should detect: verify, code, bugs, issues
        assert!(detected.len() >= 3, "Should detect at least 3 Mandarin opportunities");
    }

    #[test]
    fn test_structural_detection() {
        let detector = PatternDetector::new();
        let text = "The distance is 10 kilometers and it takes 5 minutes at 50 percent speed.";

        let detected = detector.detect_structural(text);
        // Should detect: "10 kilometers" → "10km", "5 minutes" → "5min", "50 percent" → "50%"
        assert!(detected.len() >= 3, "Should detect at least 3 structural optimizations");

        // Verify one of the detections
        let km_opt = detected.iter().find(|d| d.original_text.contains("kilometer"));
        assert!(km_opt.is_some());
        assert!(km_opt.unwrap().optimized_text.contains("km"));
    }

    #[test]
    fn test_structural_formatting() {
        let detector = PatternDetector::new();
        let text = "===\nCheck this!!!\nIs this right???\nWait...\n\n\n\nNext section.";

        let detected = detector.detect_structural(text);
        // Should detect: ===, !!!, ???, ..., \n\n\n+
        assert!(detected.len() >= 4, "Should detect formatting optimizations: found {}", detected.len());
    }

    #[test]
    fn test_structural_json_keys() {
        let detector = PatternDetector::new();
        let text = r#"{"description": "test", "configuration": "prod", "parameters": {}}"#;

        let detected = detector.detect_structural(text);
        // Should detect: description → desc, configuration → config, parameters → params
        assert!(detected.len() >= 3, "Should detect JSON key shortenings");
    }
}
