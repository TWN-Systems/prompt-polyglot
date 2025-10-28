# Testing Strategy with Hugging Face Datasets

> Comprehensive testing strategy using real-world prompts from Hugging Face

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Two-Tier Strategy](#two-tier-strategy)
3. [Hugging Face Datasets](#hugging-face-datasets)
4. [Test Corpus Structure](#test-corpus-structure)
5. [Implementation](#implementation)
6. [CI/CD Integration](#cicd-integration)
7. [Benchmarking](#benchmarking)
8. [Atlas Training](#atlas-training)

---

## Testing Philosophy

### Why Multiple Test Tiers?

**Q1678 (Curated)**: Like unit tests in software - fast, targeted, regression protection
**HF Datasets**: Like integration tests - comprehensive, statistical, real-world validation

Just as you wouldn't delete unit tests when adding integration tests, both tiers serve different purposes:

| Aspect | Q1678 (Unit Tests) | HF Datasets (Integration Tests) |
|--------|-------------------|----------------------------------|
| **Purpose** | Regression testing, development validation | Statistical validation, real-world testing |
| **When to run** | Every commit, every PR | Nightly builds, pre-release |
| **Speed** | <1 minute for 1,678 cases | 10-30 minutes for 20K+ samples |
| **What it proves** | Core patterns work, no regressions | Works at scale on diverse data |
| **Value** | Fast feedback loop | Statistical significance |

---

## Two-Tier Strategy

### Tier 1: Q1678 - Fast Regression Tests

**Use for:**
- Development velocity (fast feedback)
- Pattern-specific validation
- Edge case detection
- Regression prevention
- Pre-commit checks

**Characteristics:**
- Hand-curated test cases
- Known expected outputs
- Covers specific compression patterns
- Runs in <1 minute
- Perfect for TDD workflow

```bash
# Run on every commit
prompt-compress test --suite q1678

# Development watch mode
prompt-compress test --suite q1678 --watch
```

### Tier 2: HF Datasets - Comprehensive Integration Tests

**Use for:**
- Statistical validation
- Real-world diversity testing
- Unknown edge case discovery
- Benchmarking against baselines
- Atlas training data
- Release validation

**Characteristics:**
- 20K+ real user prompts
- Diverse contexts (chat, code, instructions)
- Statistical significance
- Runs in 10-30 minutes
- Perfect for release validation

```bash
# Run before releases
prompt-compress benchmark --suite full --report release_v0.4.json

# Weekly validation
prompt-compress benchmark --suite wildchat,ultrachat
```

---

## Hugging Face Datasets

### Top Priority Datasets

#### 1. **[allenai/WildChat-1M](https://huggingface.co/datasets/allenai/WildChat-1M)** ⭐⭐⭐

**Stats:** 13.6K downloads | 1M real user conversations

**Why use it:**
- Real-world user behavior
- Highly verbose, boilerplate-heavy prompts
- Conversational patterns
- Perfect for testing boilerplate removal

**Example prompts:**
```
"I would really appreciate it if you could please help me understand..."
"Could you please take the time to carefully analyze..."
"Thank you so much in advance for your detailed explanation..."
```

**Load it:**
```python
from datasets import load_dataset
ds = load_dataset("allenai/WildChat-1M", split="train[:10000]")
prompts = [conv[0]["content"] for conv in ds["conversation"]]
```

#### 2. **[HuggingFaceH4/ultrachat_200k](https://huggingface.co/datasets/HuggingFaceH4/ultrachat_200k)** ⭐⭐⭐

**Stats:** 21.9K downloads | High-quality filtered conversations

**Why use it:**
- Multi-turn compression patterns
- Varied instruction types (coding, writing, analysis)
- Quality-filtered dataset
- Good for context-aware optimization testing

**Example patterns:**
- Complex multi-step instructions
- Technical explanations
- Creative writing prompts

**Load it:**
```python
ds = load_dataset("HuggingFaceH4/ultrachat_200k", split="train_sft[:5000]")
prompts = [msg[0]["content"] for msg in ds["messages"]]
```

#### 3. **[tatsu-lab/alpaca](https://huggingface.co/datasets/tatsu-lab/alpaca)** ⭐⭐

**Stats:** 39.1K downloads | 52K instruction-response pairs

**Why use it:**
- Standardized instruction format
- Easy to process
- Benchmark baseline
- Diverse task types

**Example instructions:**
```
"Give three tips for staying healthy."
"Arrange the words in the given sentence to form a grammatically correct sentence."
"Classify the given input as either a poem, story, or song."
```

**Load it:**
```python
ds = load_dataset("tatsu-lab/alpaca", split="train[:5000]")
prompts = ds["instruction"]
```

### Specialized Testing Datasets

#### 4. **[iamtarun/python_code_instructions_18k_alpaca](https://huggingface.co/datasets/iamtarun/python_code_instructions_18k_alpaca)** ⭐

**Stats:** 1.4K downloads | Code-focused instructions

**Why use it:**
- Test protected regions (code blocks)
- Mix of natural language + code
- Verify code never gets corrupted

**Critical test:**
```python
# Input:
"Please analyze this function and explain what it does:
```python
def foo(x):
    return x * 2
```"

# Expected: Code block MUST be preserved exactly
```

#### 5. **[rubend18/DALL-E-Prompts-OpenAI-ChatGPT](https://huggingface.co/datasets/rubend18/DALL-E-Prompts-OpenAI-ChatGPT)** ⭐

**Stats:** 29 downloads | 1M+ image generation prompts

**Why use it:**
- Highly verbose, descriptive text
- Creative/artistic language
- Tests aggressive compression limits

**Example:**
```
"A highly detailed, photorealistic image of a beautiful sunset over the ocean,
with vibrant orange and pink colors reflecting on the calm water, dramatic
clouds in the sky, and a silhouette of a lone sailboat in the distance..."
```

### Benchmarking Datasets

#### 6. **[HuggingFaceFW/fineweb-edu](https://huggingface.co/datasets/HuggingFaceFW/fineweb-edu)**

**Stats:** 360K downloads | 1.3T tokens

**Why use it:**
- Educational content (documentation style)
- Diverse text styles
- Multilingual
- Test on explanation-heavy text

#### 7. **[bigcode/the-stack](https://huggingface.co/datasets/bigcode/the-stack)** (Gated)

**Stats:** 35.6K downloads | Code in 30+ languages

**Why use it:**
- Code comments compression
- Multi-language code protection
- Verify protected regions across languages

---

## Test Corpus Structure

### Directory Layout

```
data/
├── test_suites/
│   ├── q1678/
│   │   ├── cases.jsonl                    # Curated test cases
│   │   ├── expected_outputs.jsonl         # Ground truth
│   │   ├── patterns.json                  # Which patterns each tests
│   │   └── README.md                      # Why each case exists
│   │
│   ├── wildchat_10k/
│   │   ├── raw.jsonl                      # Downloaded from HF
│   │   ├── prompts.jsonl                  # Extracted prompts
│   │   └── metadata.json                  # Source info
│   │
│   ├── ultrachat_5k/
│   ├── alpaca_5k/
│   └── code_mixed_3k/
│
├── benchmarks/
│   ├── baselines/
│   │   ├── q1678_v0.3_baseline.json
│   │   └── wildchat_v0.3_baseline.json
│   │
│   ├── current/
│   │   └── $(date)_results.json
│   │
│   └── reports/
│       └── comparison_v0.3_vs_v0.4.md
│
└── atlas_training/
    ├── compression_decisions.db           # HITL feedback
    └── learned_patterns.json              # Patterns learned from data
```

---

## Implementation

### Script: Download Datasets

`scripts/download_hf_datasets.py`:

```python
#!/usr/bin/env python3
"""Download and prepare Hugging Face datasets for testing."""

import json
from pathlib import Path
from datasets import load_dataset
from typing import List, Tuple, Dict

# Configuration
DATASETS = {
    "wildchat": {
        "repo": "allenai/WildChat-1M",
        "samples": 10000,
        "extract": lambda ex: ex["conversation"][0]["content"] if ex["conversation"] else None,
    },
    "ultrachat": {
        "repo": "HuggingFaceH4/ultrachat_200k",
        "samples": 5000,
        "split": "train_sft",
        "extract": lambda ex: ex["messages"][0]["content"] if ex["messages"] else None,
    },
    "alpaca": {
        "repo": "tatsu-lab/alpaca",
        "samples": 5000,
        "extract": lambda ex: ex["instruction"],
    },
    "code_mixed": {
        "repo": "iamtarun/python_code_instructions_18k_alpaca",
        "samples": 3000,
        "extract": lambda ex: ex["prompt"],
    },
}

def download_dataset(name: str, config: Dict) -> List[str]:
    """Download and extract prompts from a dataset."""
    print(f"📥 Downloading {name}...")

    split = config.get("split", "train")
    n_samples = config["samples"]

    # Load dataset
    ds = load_dataset(
        config["repo"],
        split=f"{split}[:{n_samples}]",
        trust_remote_code=True
    )

    # Extract prompts
    print(f"📝 Extracting prompts from {len(ds)} examples...")
    prompts = []
    for ex in ds:
        try:
            prompt = config["extract"](ex)
            if prompt and len(prompt) > 10:  # Filter too short
                prompts.append(prompt)
        except Exception as e:
            print(f"⚠️  Error extracting: {e}")
            continue

    print(f"✅ Extracted {len(prompts)} prompts from {name}")
    return prompts

def save_prompts(prompts: List[str], output_dir: Path, name: str):
    """Save prompts to JSONL file."""
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save raw prompts
    prompts_file = output_dir / "prompts.jsonl"
    with open(prompts_file, "w") as f:
        for prompt in prompts:
            f.write(json.dumps({"text": prompt}) + "\n")

    # Save metadata
    metadata = {
        "source": name,
        "count": len(prompts),
        "avg_length": sum(len(p) for p in prompts) / len(prompts),
    }
    with open(output_dir / "metadata.json", "w") as f:
        json.dump(metadata, f, indent=2)

    print(f"💾 Saved to {prompts_file}")

def main():
    """Download all datasets."""
    base_dir = Path("data/test_suites")

    for name, config in DATASETS.items():
        print(f"\n{'='*60}")
        print(f"Processing: {name}")
        print(f"{'='*60}")

        try:
            prompts = download_dataset(name, config)
            save_prompts(prompts, base_dir / f"{name}_{config['samples']//1000}k", name)
        except Exception as e:
            print(f"❌ Failed to process {name}: {e}")
            continue

    print(f"\n{'='*60}")
    print("✅ All datasets downloaded!")
    print(f"{'='*60}")
    print(f"\nTotal test cases: ~23,000")
    print(f"Location: {base_dir.absolute()}")

if __name__ == "__main__":
    main()
```

### Script: Build Test Corpus

`scripts/build_test_corpus.py`:

```python
#!/usr/bin/env python3
"""Build comprehensive test corpus from multiple sources."""

import json
from pathlib import Path
from typing import List, Dict
import random

def load_prompts(suite_dir: Path) -> List[Dict]:
    """Load prompts from a test suite directory."""
    prompts_file = suite_dir / "prompts.jsonl"
    if not prompts_file.exists():
        return []

    prompts = []
    with open(prompts_file) as f:
        for line in f:
            data = json.loads(line)
            data["source"] = suite_dir.name
            prompts.append(data)

    return prompts

def categorize_prompts(prompts: List[Dict]) -> Dict[str, List[Dict]]:
    """Categorize prompts by characteristics."""
    categories = {
        "verbose": [],      # >200 chars
        "concise": [],      # <100 chars
        "code_mixed": [],   # Contains code blocks
        "multilingual": [], # Non-ASCII chars
        "boilerplate": [],  # Contains common boilerplate
    }

    for prompt in prompts:
        text = prompt["text"]
        length = len(text)

        # Categorize
        if length > 200:
            categories["verbose"].append(prompt)
        elif length < 100:
            categories["concise"].append(prompt)

        if "```" in text or "def " in text or "function " in text:
            categories["code_mixed"].append(prompt)

        if any(ord(c) > 127 for c in text):
            categories["multilingual"].append(prompt)

        boilerplate_phrases = [
            "I would appreciate",
            "Please make sure",
            "Thank you in advance",
            "Could you please",
        ]
        if any(phrase in text for phrase in boilerplate_phrases):
            categories["boilerplate"].append(prompt)

    return categories

def sample_balanced_corpus(categories: Dict[str, List[Dict]], n_per_category: int = 1000) -> List[Dict]:
    """Sample a balanced corpus from categories."""
    corpus = []

    for category, prompts in categories.items():
        if len(prompts) > n_per_category:
            sampled = random.sample(prompts, n_per_category)
        else:
            sampled = prompts

        for prompt in sampled:
            prompt["category"] = category

        corpus.extend(sampled)
        print(f"✓ {category}: {len(sampled)} prompts")

    return corpus

def main():
    """Build test corpus."""
    test_suites_dir = Path("data/test_suites")

    # Load all prompts
    all_prompts = []
    for suite_dir in test_suites_dir.iterdir():
        if suite_dir.is_dir() and suite_dir.name != "q1678":
            prompts = load_prompts(suite_dir)
            all_prompts.extend(prompts)
            print(f"Loaded {len(prompts)} from {suite_dir.name}")

    print(f"\nTotal prompts: {len(all_prompts)}")

    # Categorize
    print("\nCategorizing prompts...")
    categories = categorize_prompts(all_prompts)

    # Sample balanced corpus
    print("\nBuilding balanced corpus...")
    corpus = sample_balanced_corpus(categories, n_per_category=1000)

    # Save
    output_file = test_suites_dir / "comprehensive_corpus.jsonl"
    with open(output_file, "w") as f:
        for prompt in corpus:
            f.write(json.dumps(prompt) + "\n")

    print(f"\n✅ Saved {len(corpus)} prompts to {output_file}")

    # Stats
    print("\n" + "="*60)
    print("Corpus Statistics:")
    print("="*60)
    for category in categories.keys():
        count = sum(1 for p in corpus if p.get("category") == category)
        print(f"  {category:15s}: {count:5d} prompts")
    print(f"  {'TOTAL':15s}: {len(corpus):5d} prompts")

if __name__ == "__main__":
    main()
```

### Rust Test Suite Implementation

`src/testing/test_suites.rs`:

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum TestSuite {
    Q1678,           // Curated regression tests
    WildChat10K,     // Real user prompts
    UltraChat5K,     // Multi-turn conversations
    Alpaca5K,        // Instructions
    CodeMixed3K,     // Code protection tests
    Comprehensive,   // Balanced corpus
    Full,            // All of the above
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestCase {
    pub text: String,
    pub source: Option<String>,
    pub category: Option<String>,
    pub expected_savings_min: Option<f64>,
    pub expected_preserved: Option<Vec<String>>,
}

impl TestSuite {
    pub fn path(&self) -> PathBuf {
        let base = PathBuf::from("data/test_suites");
        match self {
            Self::Q1678 => base.join("q1678/cases.jsonl"),
            Self::WildChat10K => base.join("wildchat_10k/prompts.jsonl"),
            Self::UltraChat5K => base.join("ultrachat_5k/prompts.jsonl"),
            Self::Alpaca5K => base.join("alpaca_5k/prompts.jsonl"),
            Self::CodeMixed3K => base.join("code_mixed_3k/prompts.jsonl"),
            Self::Comprehensive => base.join("comprehensive_corpus.jsonl"),
            Self::Full => base.join("comprehensive_corpus.jsonl"),
        }
    }

    pub fn load(&self) -> Result<Vec<TestCase>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let path = self.path();
        let file = File::open(&path)
            .map_err(|e| anyhow::anyhow!("Failed to open {:?}: {}", path, e))?;

        let reader = BufReader::new(file);
        let mut cases = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let case: TestCase = serde_json::from_str(&line)?;
            cases.push(case);
        }

        Ok(cases)
    }

    pub fn is_fast(&self) -> bool {
        matches!(self, Self::Q1678)
    }

    pub fn expected_duration(&self) -> std::time::Duration {
        match self {
            Self::Q1678 => std::time::Duration::from_secs(60),
            Self::WildChat10K | Self::UltraChat5K | Self::Alpaca5K => {
                std::time::Duration::from_secs(300)
            }
            Self::CodeMixed3K => std::time::Duration::from_secs(180),
            Self::Comprehensive => std::time::Duration::from_secs(600),
            Self::Full => std::time::Duration::from_secs(1800),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_q1678() {
        let suite = TestSuite::Q1678;
        let cases = suite.load().expect("Failed to load Q1678");
        assert_eq!(cases.len(), 1678, "Q1678 should have exactly 1678 cases");
    }

    #[test]
    fn test_suite_is_fast() {
        assert!(TestSuite::Q1678.is_fast());
        assert!(!TestSuite::WildChat10K.is_fast());
        assert!(!TestSuite::Full.is_fast());
    }
}
```

---

## CI/CD Integration

### GitHub Actions Workflow

`.github/workflows/test.yml`:

```yaml
name: Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    # Run nightly at 2 AM UTC
    - cron: '0 2 * * *'

jobs:
  quick-test:
    name: Quick Tests (Q1678)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Run Unit Tests (Q1678)
        run: |
          cargo test --suite q1678 --verbose
        timeout-minutes: 5
        # ✓ Fast: completes in <2 minutes

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: q1678-results
          path: test-results/q1678_*.json

  integration-test:
    name: Integration Tests (HF Datasets)
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' || github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.10'

      - name: Install dependencies
        run: |
          pip install datasets tiktoken

      - name: Download HF datasets
        run: |
          python3 scripts/download_hf_datasets.py
        timeout-minutes: 30

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run integration tests
        run: |
          cargo test --suite comprehensive --report integration_$(date +%Y%m%d).json
        timeout-minutes: 45

      - name: Generate benchmark report
        run: |
          cargo run --bin generate-report -- \
            --baseline benchmarks/baselines/v0.3.json \
            --current integration_$(date +%Y%m%d).json \
            --output reports/benchmark_$(date +%Y%m%d).md

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: integration-results
          path: |
            integration_*.json
            reports/benchmark_*.md

  pre-release:
    name: Pre-Release Validation
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    steps:
      - uses: actions/checkout@v3

      - name: Full Test Suite
        run: |
          cargo test --suite full --report release_${{ github.ref_name }}.json
        timeout-minutes: 60

      - name: Compare against previous release
        run: |
          cargo run --bin compare-releases -- \
            --baseline benchmarks/baselines/latest.json \
            --current release_${{ github.ref_name }}.json \
            --fail-on-regression

      - name: Upload release validation
        uses: actions/upload-artifact@v3
        with:
          name: release-validation
          path: release_${{ github.ref_name }}.json
```

---

## Benchmarking

### Running Benchmarks

```bash
# Baseline (establish reference)
prompt-compress benchmark \
  --suite comprehensive \
  --tokenizers cl100k,claude,llama3,deepseek \
  --output benchmarks/baselines/v0.4_baseline.json

# Compare against baseline
prompt-compress benchmark \
  --suite comprehensive \
  --baseline benchmarks/baselines/v0.4_baseline.json \
  --output benchmarks/current/$(date +%Y%m%d).json \
  --generate-report

# Multi-tokenizer comparison
prompt-compress benchmark \
  --suite wildchat \
  --tokenizers cl100k,claude,llama3 \
  --compare-tokenizers \
  --output reports/tokenizer_comparison.json
```

### Benchmark Report Format

`benchmarks/current/20241028.json`:

```json
{
  "metadata": {
    "version": "0.4.0",
    "date": "2024-10-28",
    "suite": "comprehensive",
    "total_cases": 5000
  },
  "overall": {
    "total_tokens_saved": 125340,
    "avg_savings_percent": 32.5,
    "median_savings_percent": 28.1,
    "p95_savings_percent": 62.3,
    "avg_confidence": 0.91
  },
  "by_category": {
    "verbose": {
      "count": 1000,
      "avg_savings": 45.2,
      "confidence": 0.94
    },
    "code_mixed": {
      "count": 1000,
      "avg_savings": 18.3,
      "confidence": 0.88,
      "code_corruption_rate": 0.0
    }
  },
  "by_tokenizer": {
    "cl100k_base": { "avg_savings": 32.5 },
    "claude": { "avg_savings": 31.8 },
    "llama3": { "avg_savings": 33.2 }
  },
  "regressions": [],
  "improvements": [
    {
      "pattern": "boilerplate_removal",
      "improvement": "+5.2% vs baseline"
    }
  ]
}
```

---

## Atlas Training

### Training from HF Data

```python
# scripts/train_atlas_from_hf.py

from datasets import load_dataset
import sqlite3

def train_from_wildchat(db_path: str, n_samples: int = 10000):
    """Train pattern atlas from WildChat dataset."""

    # Load dataset
    ds = load_dataset("allenai/WildChat-1M", split=f"train[:{n_samples}]")

    # Connect to database
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Process each conversation
    for example in ds:
        for message in example["conversation"]:
            if message["role"] == "user":
                prompt = message["content"]

                # Optimize with HITL simulation
                result = optimize_prompt(prompt)

                # Record decision (simulated accept if high confidence)
                if result["confidence"] > 0.90:
                    decision = "accept"
                elif result["confidence"] < 0.70:
                    decision = "reject"
                else:
                    decision = "review"

                # Update database
                cursor.execute("""
                    INSERT INTO hitl_decisions (pattern_id, original_text, optimized_text, decision)
                    VALUES (?, ?, ?, ?)
                """, (result["pattern_id"], prompt, result["optimized"], decision))

    conn.commit()
    conn.close()

    print(f"✅ Trained atlas from {n_samples} WildChat samples")
```

---

## Development Workflow

### Daily Development

```bash
# Morning: Run fast tests
cargo test --suite q1678

# Development loop (watch mode)
cargo watch -x "test --suite q1678"

# Before committing
cargo test --suite q1678 && git commit
```

### Weekly Validation

```bash
# Download latest HF data
python3 scripts/download_hf_datasets.py

# Run comprehensive tests
cargo test --suite comprehensive

# Compare against last week's baseline
cargo run --bin compare-results -- \
  --baseline benchmarks/week_$(date -d "7 days ago" +%Y%m%d).json \
  --current benchmarks/week_$(date +%Y%m%d).json
```

### Pre-Release

```bash
# Full validation
cargo test --suite full --verbose

# Generate release report
cargo run --bin generate-release-report -- \
  --version 0.4.0 \
  --baseline benchmarks/baselines/v0.3.json \
  --output RELEASE_NOTES_v0.4.md

# Validate no regressions
cargo run --bin validate-release -- \
  --fail-on-regression \
  --min-avg-savings 30.0 \
  --max-corruption-rate 0.001
```

---

## Metrics to Track

### Core Metrics

1. **Token Savings**
   - Average savings percentage
   - Median savings
   - P95/P99 savings (for aggressive mode)
   - Total tokens saved

2. **Confidence Scores**
   - Average confidence
   - Distribution by confidence bucket
   - HITL acceptance rate

3. **Correctness**
   - Code corruption rate (should be 0%)
   - URL preservation rate (should be 100%)
   - Semantic preservation (manual review)

4. **Performance**
   - Throughput (prompts/second)
   - Latency (p50, p95, p99)
   - Cache hit rate

### Quality Metrics

1. **By Category**
   - Verbose prompts: >40% savings expected
   - Concise prompts: <20% savings expected
   - Code-mixed: 15-30% savings, 0% corruption
   - Boilerplate-heavy: >60% savings expected

2. **By Pattern Type**
   - Boilerplate removal: >90% confidence
   - Filler words: >85% confidence
   - Synonym consolidation: >85% confidence
   - Mandarin substitution: >90% confidence

3. **Regression Detection**
   - No pattern should degrade >5% vs baseline
   - Overall savings should not drop >2%
   - Confidence should not drop >3%

---

## Summary

### Quick Reference

| Task | Command | Duration |
|------|---------|----------|
| Development testing | `cargo test --suite q1678` | <1 min |
| Pre-commit check | `cargo test --suite q1678` | <1 min |
| Weekly validation | `cargo test --suite comprehensive` | ~10 min |
| Pre-release | `cargo test --suite full` | ~30 min |
| Benchmarking | `cargo benchmark --suite wildchat` | ~15 min |
| Atlas training | `python train_atlas.py --dataset wildchat` | ~1 hour |

### Best Practices

1. ✅ **Always** run Q1678 before committing
2. ✅ **Weekly** run comprehensive tests
3. ✅ **Pre-release** run full suite
4. ✅ **Monthly** update HF datasets
5. ✅ **Quarterly** retrain atlas with new data

---

**Next Steps:**
- [Download datasets](../../scripts/download_hf_datasets.py)
- [Build test corpus](../../scripts/build_test_corpus.py)
- [Configure CI/CD](./.github/workflows/test.yml)
- [Run benchmarks](#benchmarking)
