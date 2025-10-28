# Prompt Optimizer with Multilingual Token Compression
## Project Specification Document v2.0

---

## 1. Project Overview

**Name**: `prompt-compress`

**Purpose**: Optimize long, verbose prompts by removing boilerplate, eliminating redundancy, and strategically using token-efficient languages (Mandarin) for compression, while preserving semantic meaning and output quality.

**Core Value Proposition**: Achieve 10-15% average token savings (30-50% for boilerplate-heavy prompts) by intelligently compressing prompts without degrading LLM comprehension.

**Key Innovation**: Bayesian confidence scoring enables human-in-the-loop (HITL) review for uncertain optimizations.

---

## 2. Use Case Flow

```
User writes verbose English prompt:
┌─────────────────────────────────────────────────────────────┐
│ "I would really appreciate it if you could please take a    │
│ look at this code and analyze what it does. I want you to   │
│ provide a detailed explanation and also research potential  │
│ performance issues. Please make sure to be thorough."       │
└─────────────────────────────────────────────────────────────┘
                         ↓
              [prompt-compress analysis]
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Optimizations detected:                                      │
│ • Remove filler: "I would really appreciate it if"          │
│   Confidence: 95% ✓ Auto-applied                            │
│ • Consolidate synonyms: "look at/analyze" → "analyze"       │
│   Confidence: 92% ✓ Auto-applied                            │
│ • Consolidate: "want/provide" → "provide"                   │
│   Confidence: 88% ✓ Auto-applied                            │
│ • Consolidate: "research/analyze" → single term              │
│   Confidence: 72% ⚠ Requires review                         │
│ • Convert boilerplate to ZH: "Please make sure to be..."    │
│   Confidence: 85% ✓ Auto-applied                            │
└─────────────────────────────────────────────────────────────┘
                         ↓
              [HITL review if needed]
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ Optimized prompt (48 tokens → 35 tokens, 27% reduction):    │
│                                                              │
│ "Analyze this code: explanation + performance issues.       │
│ 要详细。[output_language: english]"                          │
│                                                              │
│ [Estimated token savings: 13 tokens]                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Core Optimization Strategies

### 3.1 Boilerplate Removal (High Confidence)

**Common patterns**:
- "I would really appreciate if you could..."
- "Please make sure to..."
- "If you don't mind, could you..."
- "Thank you in advance for..."
- "I'm looking for help with..."

**Action**: Delete entirely  
**Confidence**: 90-98%

### 3.2 Redundant Conjunction Elimination

**Pattern**: Synonym pairs that mean the same thing

Examples:
- "want and desire" → "want"
- "look into and research" → "research"
- "examine and analyze" → "analyze"
- "check and verify" → "verify"
- "improve and enhance" → "improve"

**Action**: Keep one, remove other  
**Confidence**: 85-95% (depends on context)

### 3.3 Filler Word Removal

**Common fillers**:
- "really", "very", "quite", "just"
- "actually", "basically", "essentially"
- "definitely", "absolutely", "certainly"

**Action**: Remove unless semantically critical  
**Confidence**: 80-90%

### 3.4 Instruction Compression

**Pattern**: Verbose instructions → concise imperatives

Examples:
- "I would like you to provide" → "Provide"
- "Can you please explain" → "Explain"
- "It would be helpful if you could show" → "Show"

**Action**: Convert to imperative form  
**Confidence**: 88-95%

### 3.5 Strategic Mandarin Substitution

**Use case**: Long boilerplate phrases that are semantically equivalent

Examples:
- "Be thorough and detailed in your response" → "要详细"
- "Make sure to consider all edge cases" → "考虑所有边缘情况"
- "Provide step-by-step explanations" → "逐步解释"

**Action**: Replace with Mandarin equivalent if token-efficient  
**Confidence**: 75-90% (higher for common phrases)

### 3.6 Format Consolidation

**Pattern**: Repeated formatting instructions

Examples:
- "Please use bullet points. Make sure each point is clear. Use numbered lists where appropriate."
- → "Format: bullets, numbered lists"

**Action**: Consolidate into concise format directive  
**Confidence**: 85-92%

---

## 4. Bayesian Confidence Scoring

### 4.1 Confidence Model

```rust
#[derive(Debug, Clone)]
pub struct OptimizationConfidence {
    pub base_confidence: f64,        // Pattern match confidence
    pub context_penalty: f64,        // Reduction based on ambiguity
    pub frequency_bonus: f64,        // Increase for common patterns
    pub semantic_risk: f64,          // Risk of meaning loss
    pub final_confidence: f64,       // Computed score
}

pub fn calculate_confidence(
    pattern: &Pattern,
    context: &Context,
    corpus_frequency: f64,
) -> OptimizationConfidence {
    // Bayesian update
    let base = pattern.base_confidence;
    let context_penalty = assess_context_risk(context);
    let frequency_bonus = corpus_frequency.log10() * 0.05;
    let semantic_risk = calculate_semantic_distance(pattern);
    
    let final_confidence = base 
        * (1.0 - context_penalty) 
        * (1.0 + frequency_bonus)
        * (1.0 - semantic_risk);
    
    OptimizationConfidence {
        base_confidence: base,
        context_penalty,
        frequency_bonus,
        semantic_risk,
        final_confidence: final_confidence.clamp(0.0, 1.0),
    }
}
```

### 4.2 Confidence Thresholds

| Confidence | Action | Example |
|------------|--------|---------|
| 95-100% | Auto-apply | "I would appreciate if" → DELETE |
| 85-94% | Auto-apply + log | "look into/research" → "research" |
| 70-84% | Require HITL review | Context-dependent synonym consolidation |
| 50-69% | Suggest, don't apply | Ambiguous pattern matches |
| <50% | Ignore | Low-confidence matches |

### 4.3 Bayesian Prior Training

Build priors from corpus of known-good optimizations:

```rust
pub struct OptimizationCorpus {
    pub patterns: HashMap<String, PatternStats>,
    pub total_optimizations: usize,
    pub success_rate: f64,
}

#[derive(Debug)]
pub struct PatternStats {
    pub occurrences: usize,
    pub successful_optimizations: usize,
    pub failed_optimizations: usize,
    pub avg_token_savings: f64,
}

// Update priors based on user feedback
pub fn update_priors(
    corpus: &mut OptimizationCorpus,
    pattern: &str,
    accepted: bool,
    token_savings: i64,
) {
    // Bayesian update formula
    let stats = corpus.patterns.entry(pattern.to_string())
        .or_insert_with(|| PatternStats::default());
    
    if accepted {
        stats.successful_optimizations += 1;
    } else {
        stats.failed_optimizations += 1;
    }
    
    // Recalculate confidence for this pattern
    stats.update_confidence();
}
```

---

## 5. Data Models

### 5.1 Core Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub prompt: String,
    pub output_language: Language,
    pub confidence_threshold: f64,  // Default: 0.85
    pub aggressive_mode: bool,       // Default: false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    English,
    Mandarin,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    BoilerplateRemoval,
    SynonymConsolidation,
    FillerRemoval,
    InstructionCompression,
    MandarinSubstitution,
    FormatConsolidation,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewSession {
    pub session_id: String,
    pub pending_optimizations: Vec<Optimization>,
    pub decisions: HashMap<String, ReviewDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Accept,
    Reject,
    Modify(String),  // User provides alternative
}
```

---

## 6. Architecture

### 6.1 Processing Pipeline

```
Input Prompt
     ↓
[1. Tokenize & Count]
     ↓
[2. Pattern Detection]
     ↓
[3. Confidence Scoring] ←─ Bayesian Priors
     ↓
[4. Auto-apply High-Confidence]
     ↓
[5. Queue Low-Confidence for HITL]
     ↓
[6. HITL Review (if needed)]
     ↓
[7. Apply Approved Optimizations]
     ↓
[8. Add Output Language Directive]
     ↓
[9. Tokenize & Calculate Savings]
     ↓
Output Optimized Prompt + Report
```

### 6.2 Pattern Detection Engine

```rust
pub struct PatternDetector {
    boilerplate_patterns: Vec<RegexPattern>,
    synonym_pairs: HashMap<String, Vec<String>>,
    filler_words: HashSet<String>,
    mandarin_substitutions: HashMap<String, String>,
}

impl PatternDetector {
    pub fn detect_all(&self, text: &str) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        
        patterns.extend(self.detect_boilerplate(text));
        patterns.extend(self.detect_synonyms(text));
        patterns.extend(self.detect_fillers(text));
        patterns.extend(self.detect_compressions(text));
        patterns.extend(self.detect_mandarin_opportunities(text));
        
        patterns
    }
    
    fn detect_boilerplate(&self, text: &str) -> Vec<DetectedPattern> {
        // Regex matching against known boilerplate
    }
    
    fn detect_synonyms(&self, text: &str) -> Vec<DetectedPattern> {
        // Find redundant synonym pairs within proximity
    }
    
    fn detect_mandarin_opportunities(&self, text: &str) -> Vec<DetectedPattern> {
        // Identify phrases that would be more token-efficient in Mandarin
    }
}
```

---

## 7. CLI Interface

```bash
# Basic optimization
prompt-compress optimize \
  --input prompt.txt \
  --output-lang english \
  --output optimized.txt

# With custom confidence threshold
prompt-compress optimize \
  --input prompt.txt \
  --threshold 0.90 \
  --output-lang mandarin

# Aggressive mode (lower threshold, more compression)
prompt-compress optimize \
  --input prompt.txt \
  --aggressive \
  --output optimized.txt

# Interactive HITL mode
prompt-compress optimize \
  --input prompt.txt \
  --interactive \
  --output optimized.txt

# Analyze without applying
prompt-compress analyze \
  --input prompt.txt \
  --report savings_report.json

# Update priors from feedback
prompt-compress train \
  --feedback feedback.json

# Batch processing
prompt-compress batch \
  --input prompts/ \
  --output optimized/ \
  --output-lang english
```

---

## 8. Interactive HITL Review

```bash
$ prompt-compress optimize --input prompt.txt --interactive

Analyzing prompt (156 tokens)...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Auto-applied optimizations (95%+ confidence):

✓ Removed boilerplate: "I would really appreciate if you could"
  Tokens saved: 8
  Confidence: 97%

✓ Consolidated synonyms: "look at and analyze" → "analyze"
  Tokens saved: 3
  Confidence: 92%

✓ Removed filler: "really", "very" (3 occurrences)
  Tokens saved: 3
  Confidence: 89%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Requires your review (3 optimizations):

[1/3] Consolidate: "research potential issues" + "analyze problems"
      → Suggestion: Use single term "analyze issues"
      Tokens saved: 4
      Confidence: 78%
      
      Original: "...research potential performance issues and analyze problems..."
      Optimized: "...analyze performance issues..."
      
      Accept? [Y/n/e] > y

[2/3] Replace with Mandarin: "Make sure to be thorough and detailed"
      → Suggestion: "要详细" (3 tokens vs 7 tokens)
      Tokens saved: 4
      Confidence: 82%
      
      Accept? [Y/n/e] > y

[3/3] Remove conjunction: "check and verify"
      → Suggestion: "verify"
      Tokens saved: 2
      Confidence: 71%
      
      Context: "...please check and verify the output format..."
      
      Accept? [Y/n/e] > n
      Reason: Need both actions
      
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Optimization complete!

Original: 156 tokens
Optimized: 132 tokens
Savings: 24 tokens (15.4%)

Saving to: optimized.txt
Updating confidence priors...

Would you like to see the optimized prompt? [Y/n] > y

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Optimized Prompt:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Analyze this code: explanation + performance issues. 要详细。
Check and verify output format.

[output_language: english]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 9. Output Language Directive

### 9.1 Automatic Injection

Every optimized prompt gets a language directive appended:

```
[optimized prompt content]

[output_language: english]
```

or

```
[optimized prompt content]

[output_language: mandarin]
```

### 9.2 Alternative Formats

```rust
pub enum DirectiveFormat {
    Bracketed,      // [output_language: english]
    Instructive,    // "Respond in English."
    Xml,            // <output_language>english</output_language>
    Natural,        // "Please respond to me in English."
}
```

User can configure preferred format:

```bash
prompt-compress optimize \
  --directive-format instructive \
  --output-lang english
```

---

## 10. Pattern Database

### 10.1 Boilerplate Patterns (Regex)

```rust
pub static BOILERPLATE_PATTERNS: &[(&str, &str, f64)] = &[
    // (Pattern, Replacement, Base Confidence)
    (r"I would (really )?appreciate (it )?if you could", "", 0.97),
    (r"Please make sure to", "", 0.95),
    (r"If you don't mind,?", "", 0.94),
    (r"Thank you (so much )?in advance", "", 0.96),
    (r"I('m| am) looking for help with", "", 0.93),
    (r"Could you please", "", 0.95),
    (r"Would you mind", "", 0.94),
    (r"I would like you to", "", 0.96),
    (r"It would be great if", "", 0.93),
    (r"I need you to", "", 0.92),
];
```

### 10.2 Synonym Pairs

```rust
pub static SYNONYM_PAIRS: &[(&str, &[&str], f64)] = &[
    // (Preferred term, Alternatives, Base Confidence)
    ("analyze", &["look at", "examine", "inspect"], 0.89),
    ("research", &["look into", "investigate"], 0.88),
    ("verify", &["check", "confirm"], 0.85),
    ("improve", &["enhance", "optimize"], 0.87),
    ("explain", &["describe", "clarify"], 0.84),
    ("provide", &["give", "supply"], 0.86),
    ("create", &["make", "build", "generate"], 0.83),
    ("identify", &["find", "locate", "detect"], 0.82),
];
```

### 10.3 Filler Words

```rust
pub static FILLER_WORDS: &[(&str, f64)] = &[
    // (Filler, Base Confidence for removal)
    ("really", 0.88),
    ("very", 0.85),
    ("quite", 0.87),
    ("just", 0.82),
    ("actually", 0.89),
    ("basically", 0.90),
    ("essentially", 0.89),
    ("definitely", 0.86),
    ("absolutely", 0.87),
    ("certainly", 0.85),
    ("probably", 0.80),
    ("maybe", 0.78),
];
```

### 10.4 Mandarin Substitutions

```rust
pub static MANDARIN_SUBSTITUTIONS: &[(&str, &str, usize, usize, f64)] = &[
    // (English phrase, Mandarin equivalent, EN tokens, ZH tokens, Confidence)
    ("Be thorough and detailed", "要详细", 5, 3, 0.86),
    ("Step by step", "逐步", 3, 1, 0.89),
    ("Make sure to", "确保", 3, 2, 0.84),
    ("Consider all edge cases", "考虑所有边缘情况", 4, 8, 0.82),
    ("Provide a comprehensive explanation", "提供全面解释", 4, 5, 0.85),
    ("Focus on", "专注于", 2, 3, 0.87),
    ("Pay attention to", "注意", 3, 2, 0.88),
];
```

---

## 11. Implementation Plan

### Phase 1: Core Infrastructure (Days 1-2)
- [ ] Set up Rust project
- [ ] Implement tokenizer integration
- [ ] Create data structures
- [ ] Build pattern regex engine
- [ ] Implement confidence scoring

### Phase 2: Pattern Detection (Days 3-4)
- [ ] Boilerplate detection
- [ ] Synonym detection (proximity-based)
- [ ] Filler word identification
- [ ] Mandarin substitution candidates
- [ ] Format consolidation patterns

### Phase 3: Bayesian Confidence (Day 5)
- [ ] Implement confidence calculation
- [ ] Build prior corpus structure
- [ ] Add feedback loop for updates
- [ ] Threshold-based filtering

### Phase 4: Optimization Engine (Days 6-7)
- [ ] Apply optimizations
- [ ] Handle conflicts (overlapping patterns)
- [ ] Calculate token savings
- [ ] Generate optimization report

### Phase 5: HITL Interface (Days 8-9)
- [ ] Interactive review CLI
- [ ] Accept/reject/modify flow
- [ ] Update priors from feedback
- [ ] Session persistence

### Phase 6: Output & Testing (Day 10)
- [ ] Add language directive injection
- [ ] Export formats (JSON, TXT)
- [ ] Write unit tests
- [ ] Integration testing
- [ ] Documentation

---

## 12. Example Transformations

### 12.1 Light Optimization (10-15% savings)

**Before (52 tokens)**:
```
I would really appreciate it if you could please analyze this Python 
function and explain what it does. I want you to provide a detailed 
explanation of the algorithm and also look into potential performance 
issues. Thank you!
```

**After (44 tokens, 15.4% savings)**:
```
Analyze this Python function: algorithm explanation + performance issues. 
要详细。

[output_language: english]
```

### 12.2 Heavy Optimization (30-50% savings)

**Before (128 tokens)**:
```
I would really appreciate it if you could please take the time to 
carefully review and analyze this code snippet. I want you to provide 
a very thorough and detailed explanation of what it does, how it works, 
and why it was implemented this way. Please make sure to look into any 
potential bugs, performance issues, or areas for improvement. I would 
also like you to research best practices and explain whether this code 
follows them. If you find any problems, please provide suggestions for 
how to fix them. Thank you so much in advance for your help with this!
```

**After (76 tokens, 40.6% savings)**:
```
Analyze code: functionality, implementation rationale. Identify: bugs, 
performance issues, improvements. Research best practices compliance. 
Provide fix suggestions. 要详细和全面。

[output_language: english]
```

### 12.3 Boilerplate-Heavy (50%+ savings)

**Before (94 tokens)**:
```
Hello! I hope you're doing well. I was wondering if you wouldn't mind 
helping me out with something. I would really appreciate it if you could 
please take a look at this problem I'm working on. If you don't mind, 
could you please provide some guidance? I would be very grateful for any 
help you could offer. Thank you so much in advance!
```

**After (12 tokens, 87.2% savings)**:
```
Help with this problem. Provide guidance.

[output_language: english]
```

---

## 13. Success Metrics

### 13.1 Target Benchmarks

| Prompt Type | Target Savings | Acceptable Range |
|-------------|----------------|------------------|
| Casual/polite | 10-15% | 8-20% |
| Technical/direct | 12-18% | 10-22% |
| Boilerplate-heavy | 30-50% | 25-60% |
| Already concise | 5-8% | 0-12% |

### 13.2 Quality Metrics

**Measured by HITL feedback**:
- Accept rate for auto-applied optimizations: >90%
- Accept rate for reviewed optimizations: >75%
- False positive rate: <5%
- User satisfaction: >4.5/5.0

### 13.3 Confidence Calibration

**Goal**: Confidence scores should match actual success rates

- 95% confidence → 95% accept rate
- 85% confidence → 85% accept rate
- 75% confidence → 75% accept rate

Track and recalibrate using Bayesian updates.

---

## 14. Configuration File

```toml
# prompt-compress.toml

[optimization]
confidence_threshold = 0.85
aggressive_mode = false
output_language = "english"
directive_format = "bracketed"

[hitl]
enabled = true
auto_accept_threshold = 0.95
batch_review = false

[patterns]
boilerplate_enabled = true
synonym_consolidation = true
filler_removal = true
mandarin_substitution = true
format_consolidation = true

[bayesian]
prior_corpus_path = "data/priors.json"
update_priors_on_feedback = true
min_confidence = 0.50

[output]
save_report = true
report_format = "json"
show_diff = true
```

---

## 15. Future Enhancements (Out of Scope v1.0)

**Phase 2: Advanced Features**
- Semantic similarity scoring (using embeddings)
- Context-aware optimization (understand prompt intent)
- Multi-language support (Japanese, Korean, etc.)
- LLM-assisted pattern discovery
- A/B testing framework

**Phase 3: Platform Integration**
- Browser extension
- API server
- Web UI for HITL
- IDE plugins (VSCode, etc.)

---

**Document Version**: 3.0 (HITL + Bayesian)  
**Last Updated**: 2025-10-25  
**Status**: Ready for implementation  
**Target**: 10-15% avg savings, 30-50% for boilerplate-heavy
