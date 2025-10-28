# Quick Start Guide - v0.3

## 5-Minute Setup

```bash
# 1. Build the project
cargo build --release

# 2. Populate the concept atlas database
cargo run --example populate_atlas

# 3. Run the demonstration
cargo run --example end_to_end_demo

# 4. Verify all tests pass (62 tests)
cargo test
```

## See It In Action

```bash
cargo run --example end_to_end_demo
```

**Expected output:**
```
Original (40 tokens):
I would really appreciate it if you could please help me analyze this code.
I want you to verify the function and explain what it does.
Thank you so much in advance for your help with this!

Optimized (26 tokens):
Please help me analyze this code. I want you to verify the function
and explain what it does.

Savings: 14 tokens (35.0%)
```

## What You Get

✅ **35% token savings** on boilerplate-heavy prompts
✅ **Zero code corruption** (protected regions)
✅ **Multi-tokenizer support** (GPT-4, Claude, Llama3)
✅ **Production-ready** (62/62 tests passing)

## Next Steps

- Read [PHASE3-COMPLETE.md](PHASE3-COMPLETE.md) for implementation details
- Read [FINAL-SUMMARY.md](FINAL-SUMMARY.md) for project evolution
- Read [README.md](README.md) for full documentation
- Extend `examples/populate_atlas.rs` to add more concepts

## Key Directories

```
├── src/               # Core implementation (9 modules)
├── examples/          # populate_atlas.rs, end_to_end_demo.rs
├── tests/             # mandarin_efficiency_test.rs
├── migrations/        # 001_initial_schema.sql
├── data/              # atlas.db (generated)
└── docs/              # PHASE3-COMPLETE.md, FINAL-SUMMARY.md
```

## Common Commands

```bash
# Run tests
cargo test

# Run examples
cargo run --example populate_atlas
cargo run --example end_to_end_demo

# CLI (v0.2)
cargo run --bin prompt-compress -- optimize --input examples/verbose_prompt.txt

# API server
cargo run --bin prompt-compress-server
```

## What's New in v0.3

- **Protected Regions:** Never corrupts code, templates, URLs, identifiers
- **Structural Patterns:** "10 kilometers" → "10km", JSON key shortening
- **Concept Atlas:** Wikidata Q-IDs with multi-lingual surface forms
- **Multi-Tokenizer:** Support for cl100k_base, llama3, claude

## Quick API Usage

```rust
use prompt_compress::{ConceptOptimizer, Database, OptimizationRequest, Language};
use std::sync::Arc;

let db = Database::open("data/atlas.db")?;
let mut optimizer = ConceptOptimizer::new(Arc::new(db))?;

let request = OptimizationRequest {
    prompt: "I would really appreciate if you could help.".to_string(),
    output_language: Language::English,
    confidence_threshold: 0.85,
    aggressive_mode: false,
    directive_format: DirectiveFormat::Bracketed,
};

let result = optimizer.optimize(&request)?;
println!("Saved {} tokens ({}%)", result.token_savings, result.savings_percentage);
```

That's it! You're ready to compress prompts and save energy. 🚀
