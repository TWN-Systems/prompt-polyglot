# Prompt Compress

> Optimize prompts with multilingual token compression and Bayesian confidence scoring

## Overview

`prompt-compress` is a Rust-based tool that optimizes verbose prompts by:
- Removing boilerplate and filler words
- Consolidating redundant synonyms
- Strategically using token-efficient languages (Mandarin)
- Maintaining semantic meaning with Bayesian confidence scoring

**Key Features:**
- 10-15% average token savings (30-50% for boilerplate-heavy prompts)
- Bayesian confidence scoring with human-in-the-loop (HITL) review
- REST API with webhook support for automated parsing
- CLI for batch processing and analysis

## Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Build from source

```bash
git clone https://github.com/your-org/prompt-polyglot.git
cd prompt-polyglot
cargo build --release
```

Binaries will be available in `target/release/`:
- `prompt-compress` - CLI tool
- `prompt-compress-server` - API server

## Quick Start

### CLI Usage

#### Basic Optimization

```bash
# Optimize a prompt
prompt-compress optimize \
  --input prompt.txt \
  --output optimized.txt \
  --output-lang english

# With custom confidence threshold
prompt-compress optimize \
  --input prompt.txt \
  --threshold 0.90 \
  --output-lang mandarin

# Aggressive mode (lower threshold, more compression)
prompt-compress optimize \
  --input prompt.txt \
  --aggressive
```

#### Analyze Without Optimizing

```bash
prompt-compress analyze \
  --input prompt.txt \
  --report savings_report.json
```

#### Batch Processing

```bash
prompt-compress batch \
  --input prompts/ \
  --output optimized/ \
  --output-lang english
```

### API Server

#### Start the Server

```bash
prompt-compress-server
```

The server will start on `http://0.0.0.0:8080`

#### API Endpoints

**Health Check**
```bash
curl http://localhost:8080/api/v1/health
```

**Optimize Prompt**
```bash
curl -X POST http://localhost:8080/api/v1/optimize \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I would really appreciate it if you could please help me with this task.",
    "output_language": "english",
    "confidence_threshold": 0.85,
    "aggressive_mode": false
  }'
```

**Response:**
```json
{
  "result": {
    "original_prompt": "I would really appreciate it if you could please help me with this task.",
    "optimized_prompt": "Help me with this task.\n\n[output_language: english]",
    "original_tokens": 18,
    "optimized_tokens": 12,
    "token_savings": 6,
    "savings_percentage": 33.3,
    "optimizations": [...],
    "requires_review": [],
    "output_language": "english"
  },
  "review_session_id": null
}
```

**Webhook for Automated Parsing**
```bash
curl -X POST http://localhost:8080/api/v1/webhook/optimize \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Please analyze this code carefully and provide detailed feedback.",
    "output_language": "english",
    "callback_url": "https://your-service.com/webhook/callback"
  }'
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "optimized_prompt": "Analyze this code: detailed feedback.\n\n[output_language: english]",
  "original_tokens": 12,
  "optimized_tokens": 9,
  "token_savings": 3,
  "savings_percentage": 25.0,
  "status": "completed"
}
```

If `callback_url` is provided, the same response will be POSTed to that URL asynchronously.

**Analyze Prompt**
```bash
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Your prompt here...",
    "output_language": "english"
  }'
```

## Optimization Strategies

### 1. Boilerplate Removal (High Confidence: 90-98%)

Common patterns removed:
- "I would really appreciate if you could..."
- "Please make sure to..."
- "Thank you in advance for..."

### 2. Filler Word Removal (80-90%)

Removes:
- "really", "very", "quite", "just"
- "actually", "basically", "essentially"

### 3. Synonym Consolidation (85-95%)

Examples:
- "analyze and examine" → "analyze"
- "check and verify" → "verify"
- "improve and enhance" → "improve"

### 4. Mandarin Substitution (75-90%)

Token-efficient replacements:
- "Be thorough and detailed" → "要详细" (5 tokens → 3 tokens)
- "Step by step" → "逐步" (3 tokens → 1 token)

### 5. Instruction Compression (88-95%)

- "I would like you to provide" → "Provide"
- "Can you please explain" → "Explain"

## Confidence Scoring

Uses Bayesian inference to calculate confidence:

| Confidence | Action | Example |
|------------|--------|---------|
| 95-100% | Auto-apply | "I would appreciate if" → DELETE |
| 85-94% | Auto-apply + log | "look into/research" → "research" |
| 70-84% | Require HITL review | Context-dependent synonym consolidation |
| 50-69% | Suggest, don't apply | Ambiguous pattern matches |
| <50% | Ignore | Low-confidence matches |

## Example Transformations

### Light Optimization (15% savings)

**Before (52 tokens):**
```
I would really appreciate it if you could please analyze this Python
function and explain what it does. I want you to provide a detailed
explanation of the algorithm and also look into potential performance
issues. Thank you!
```

**After (44 tokens, 15.4% savings):**
```
Analyze this Python function: algorithm explanation + performance issues.
要详细。

[output_language: english]
```

### Heavy Optimization (40% savings)

**Before (128 tokens):**
```
I would really appreciate it if you could please take the time to
carefully review and analyze this code snippet. I want you to provide
a very thorough and detailed explanation of what it does, how it works,
and why it was implemented this way. Please make sure to look into any
potential bugs, performance issues, or areas for improvement.
```

**After (76 tokens, 40.6% savings):**
```
Analyze code: functionality, implementation rationale. Identify: bugs,
performance issues, improvements. Research best practices compliance.
Provide fix suggestions. 要详细和全面。

[output_language: english]
```

## Webhook Integration

The webhook endpoint allows seamless integration with other systems:

### Use Cases

1. **CI/CD Pipeline**: Automatically optimize prompts in your test suite
2. **Content Management**: Optimize user-submitted prompts before processing
3. **Analytics**: Track token savings across your organization

### Integration Example

```python
import requests

# Optimize a prompt via webhook
response = requests.post(
    'http://localhost:8080/api/v1/webhook/optimize',
    json={
        'prompt': 'Your verbose prompt here...',
        'output_language': 'english',
        'confidence_threshold': 0.85,
        'callback_url': 'https://your-app.com/webhook/receive'
    }
)

result = response.json()
print(f"Saved {result['token_savings']} tokens ({result['savings_percentage']:.1f}%)")
```

## Architecture

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
[6. Apply Approved Optimizations]
     ↓
[7. Add Output Language Directive]
     ↓
Output Optimized Prompt
```

## Configuration

Create a `prompt-compress.toml` file:

```toml
[optimization]
confidence_threshold = 0.85
aggressive_mode = false
output_language = "english"
directive_format = "bracketed"

[hitl]
enabled = true
auto_accept_threshold = 0.95

[patterns]
boilerplate_enabled = true
synonym_consolidation = true
filler_removal = true
mandarin_substitution = true

[bayesian]
prior_corpus_path = "data/priors.json"
update_priors_on_feedback = true
min_confidence = 0.50
```

## Development

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
RUST_LOG=debug cargo run -- optimize --input test.txt
```

### API Development

```bash
RUST_LOG=info cargo run --bin prompt-compress-server
```

## License

MIT

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- Issues: [GitHub Issues](https://github.com/your-org/prompt-polyglot/issues)
- Documentation: See [CLAUDE.md](CLAUDE.md) for detailed specification
