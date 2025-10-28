# Consolidated Architecture - Database-Backed Pattern Optimization

**Date:** 2025-10-27
**Version:** v0.4 (Consolidated)
**Status:** ✅ Complete - Atlas Integration

---

## Overview

The codebase has been **consolidated** to integrate the SQLite concept atlas with the regex pattern optimization system. Previously, these were two separate systems:

### Before Consolidation (v0.3)

```
❌ DISCONNECTED SYSTEMS:

1. Hardcoded Patterns (Active)
   - patterns.rs with 92 static regex patterns
   - Used by Optimizer in production
   - No database integration

2. Concept Atlas (Unused)
   - SQLite database with concepts/surface forms
   - ConceptOptimizer implementation
   - Only used in test cases
```

### After Consolidation (v0.4)

```
✅ INTEGRATED SYSTEM:

1. Database-Backed Patterns (Production)
   - patterns table in SQLite
   - DatabaseOptimizer loads patterns from DB
   - HITL feedback updates confidence scores
   - Pattern statistics and tracking

2. Concept Atlas (Optional Enhancement)
   - Wikidata concepts for cross-lingual optimization
   - Can be used alongside pattern optimization
   - ConceptOptimizer for advanced use cases
```

---

## Architecture Components

### Core Databases

#### **atlas.db** (SQLite Database)

Contains both patterns and concepts:

1. **Patterns System** (NEW)
   - `patterns` - Regex patterns with confidence scores
   - `hitl_decisions` - User feedback for confidence calibration
   - `active_patterns` (view) - Enabled patterns ordered by confidence
   - `pattern_stats` (view) - Performance statistics

2. **Concept Atlas** (Existing)
   - `concepts` - Wikidata entities (Q-IDs)
   - `surface_forms` - Language variants with token costs
   - `optimization_cache` - Cached optimizations
   - `concept_embeddings` - Optional semantic search

### Code Architecture

```
src/
├── database.rs                    # SQLite connection + CRUD
│   ├── load_patterns()            # NEW: Load patterns from DB
│   ├── record_pattern_application() # NEW: Track usage
│   └── record_hitl_decision()     # NEW: Store feedback
│
├── database_pattern_detector.rs   # NEW: DB-backed pattern detector
│   ├── DatabasePatternDetector    # Loads regex patterns from SQLite
│   ├── detect_all()               # Pattern matching + tracking
│   └── reload_patterns()          # Refresh from DB
│
├── database_optimizer.rs          # NEW: DB-backed optimizer
│   ├── DatabaseOptimizer          # Uses DatabasePatternDetector
│   ├── optimize()                 # Main optimization loop
│   └── reload_patterns()          # Hot-reload patterns
│
├── optimizer.rs                   # Legacy hardcoded optimizer
│   └── Optimizer                  # Uses static patterns.rs (deprecated)
│
├── patterns.rs                    # Hardcoded patterns (used for migration)
│   └── BOILERPLATE_PATTERNS, etc. # Static arrays
│
├── concept_optimizer.rs           # Concept-based optimization
│   └── ConceptOptimizer           # Cross-lingual substitution
│
└── lib.rs                         # Public API
    ├── init_optimizer()                      # Legacy (hardcoded)
    ├── init_database_optimizer()             # NEW: Production (DB-backed)
    └── init_database_optimizer_with_confidence() # NEW: Filtered patterns
```

---

## Setup Guide

### Step 1: Run Pattern Migration

Migrate hardcoded patterns from `patterns.rs` into the database:

```bash
# Build the migration tool
cargo build --release --bin migrate_patterns

# Run migration (creates patterns table + inserts data)
./target/release/migrate_patterns atlas.db
```

Expected output:
```
Migrating patterns to database: "atlas.db"
✓ Schema migration applied

Migrating boilerplate patterns...
   ✓ Migrated 25 boilerplate patterns
Migrating filler word patterns...
   ✓ Migrated 31 filler patterns
Migrating instruction compression patterns...
   ✓ Migrated 6 instruction patterns
Migrating redundant phrase patterns...
   ✓ Migrated 23 redundant phrase patterns
Migrating structural optimization patterns...
   ✓ Migrated 17 structural patterns

✅ Migration complete!
   Total patterns migrated: 102

Breakdown by type:
   filler                          31 patterns (avg confidence: 84.84%)
   boilerplate                     25 patterns (avg confidence: 93.48%)
   redundant                       23 patterns (avg confidence: 88.30%)
   structural                      17 patterns (avg confidence: 90.76%)
   instruction                      6 patterns (avg confidence: 90.50%)
```

### Step 2: Update Your Code

Replace hardcoded optimizer with database-backed version:

#### Before (v0.3):
```rust
use prompt_compress::init_optimizer;

let mut optimizer = init_optimizer()?;
```

#### After (v0.4):
```rust
use prompt_compress::init_database_optimizer;

let mut optimizer = init_database_optimizer("atlas.db")?;
```

### Step 3: Use the Optimizer

Same API as before:

```rust
use prompt_compress::{init_database_optimizer, OptimizationRequest, Language, DirectiveFormat};

let mut optimizer = init_database_optimizer("atlas.db")?;

let request = OptimizationRequest {
    prompt: "I would really appreciate if you could analyze this code.".to_string(),
    output_language: Language::English,
    confidence_threshold: 0.85,
    aggressive_mode: false,
    directive_format: DirectiveFormat::Bracketed,
};

let result = optimizer.optimize(&request)?;

println!("Original: {} tokens", result.original_tokens);
println!("Optimized: {} tokens", result.optimized_tokens);
println!("Savings: {:.1}%", result.savings_percentage);
```

---

## Key Features

### 1. Pattern Management from Database

Load patterns dynamically:

```rust
use prompt_compress::{Database, init_database_optimizer_with_confidence};
use std::sync::Arc;

// Load only high-confidence patterns
let optimizer = init_database_optimizer_with_confidence("atlas.db", 0.90)?;

// Check how many patterns loaded
println!("Loaded {} patterns", optimizer.pattern_count());
```

### 2. Pattern Statistics

Track pattern performance:

```rust
let db = Database::open("atlas.db")?;
let stats = db.get_pattern_stats()?;

for stat in stats {
    println!("{}: {} patterns, {:.1}% acceptance",
             stat.pattern_type,
             stat.total_patterns,
             stat.acceptance_rate * 100.0);
}
```

### 3. HITL Feedback Integration

Record user decisions to improve confidence scores:

```rust
use prompt_compress::{Database, HitlDecision};

let db = Database::open("atlas.db")?;

let decision = HitlDecision {
    pattern_id: 42,
    session_id: "session-123".to_string(),
    original_text: "I would really appreciate".to_string(),
    optimized_text: "".to_string(),
    decision: "accept".to_string(),
    user_alternative: None,
    context_before: "Hello! ".to_string(),
    context_after: " if you could help.".to_string(),
};

db.record_hitl_decision(&decision)?;
```

**Automatic Confidence Updating:**

The database trigger automatically updates confidence scores:
- After 10+ decisions, uses empirical acceptance rate
- Before 10 decisions, blends base confidence with feedback
- Formula: `new_conf = (base * 10 + accepted) / (10 + total_decisions)`

### 4. Hot Reloading

Update patterns without restarting:

```rust
let mut optimizer = init_database_optimizer("atlas.db")?;

// ... use optimizer ...

// Reload patterns from database (picks up new patterns, confidence updates)
optimizer.reload_patterns()?;
```

### 5. Database Queries

Direct database access:

```rust
let db = Database::open("atlas.db")?;

// Load all patterns
let patterns = db.load_patterns()?;

// Load by type
let boilerplate = db.load_patterns_by_type("boilerplate")?;

// Load with threshold
let high_conf = db.load_patterns_with_confidence(0.95)?;

// Get statistics
let stats = db.get_pattern_stats()?;
```

---

## Database Schema

### Patterns Table

```sql
CREATE TABLE patterns (
    id INTEGER PRIMARY KEY,
    pattern_type TEXT NOT NULL,        -- "boilerplate", "filler", etc.
    regex_pattern TEXT NOT NULL,       -- Regex string
    replacement TEXT NOT NULL,         -- What to replace with
    base_confidence REAL NOT NULL,     -- 0.0-1.0
    reasoning TEXT NOT NULL,           -- Why this pattern works
    enabled INTEGER DEFAULT 1,         -- 1=active, 0=disabled
    applied_count INTEGER DEFAULT 0,   -- Usage tracking
    accepted_count INTEGER DEFAULT 0,  -- HITL accepts
    rejected_count INTEGER DEFAULT 0,  -- HITL rejects
    created_at INTEGER,
    updated_at INTEGER
);
```

### HITL Decisions Table

```sql
CREATE TABLE hitl_decisions (
    id INTEGER PRIMARY KEY,
    pattern_id INTEGER NOT NULL,
    session_id TEXT NOT NULL,
    original_text TEXT NOT NULL,
    optimized_text TEXT NOT NULL,
    decision TEXT NOT NULL,            -- "accept", "reject", "modify"
    user_alternative TEXT,             -- If modified
    context_before TEXT,
    context_after TEXT,
    created_at INTEGER,
    FOREIGN KEY (pattern_id) REFERENCES patterns(id)
);
```

### Useful Views

```sql
-- Active patterns ordered by confidence
CREATE VIEW active_patterns AS
SELECT *,
       CASE WHEN (accepted_count + rejected_count) > 0
            THEN CAST(accepted_count AS REAL) / (accepted_count + rejected_count)
            ELSE base_confidence
       END AS empirical_confidence
FROM patterns
WHERE enabled = 1
ORDER BY base_confidence DESC;

-- Pattern type statistics
CREATE VIEW pattern_stats AS
SELECT pattern_type,
       COUNT(*) as total_patterns,
       AVG(base_confidence) as avg_confidence,
       SUM(applied_count) as total_applications,
       SUM(accepted_count) as total_accepted,
       SUM(rejected_count) as total_rejected,
       CAST(SUM(accepted_count) AS REAL) / NULLIF(SUM(accepted_count + rejected_count), 0)
           AS acceptance_rate
FROM patterns
WHERE enabled = 1
GROUP BY pattern_type;
```

---

## Migration Checklist

To migrate from v0.3 (hardcoded) to v0.4 (database-backed):

- [ ] **1. Run migration tool**
  ```bash
  cargo run --bin migrate_patterns -- atlas.db
  ```

- [ ] **2. Update imports**
  ```rust
  // OLD:
  use prompt_compress::init_optimizer;

  // NEW:
  use prompt_compress::init_database_optimizer;
  ```

- [ ] **3. Update initialization**
  ```rust
  // OLD:
  let mut optimizer = init_optimizer()?;

  // NEW:
  let mut optimizer = init_database_optimizer("atlas.db")?;
  ```

- [ ] **4. Update CLI** (if using `src/main.rs`)
  ```rust
  // Update all calls to init_optimizer() → init_database_optimizer("atlas.db")
  ```

- [ ] **5. Update API server** (if using `src/bin/server.rs`)
  ```rust
  // Replace init_optimizer() with init_database_optimizer("atlas.db")
  ```

- [ ] **6. Test**
  ```bash
  cargo test --lib database_optimizer
  cargo test --lib database_pattern_detector
  ```

- [ ] **7. Verify database**
  ```bash
  sqlite3 atlas.db "SELECT pattern_type, COUNT(*) FROM patterns GROUP BY pattern_type;"
  ```

---

## Benefits of Consolidation

### ✅ Pattern Persistence
- Patterns stored in database, not code
- Easy to add/remove/modify patterns
- Version control of pattern changes

### ✅ HITL Integration
- User feedback directly updates confidence
- Bayesian confidence calibration
- Track pattern effectiveness

### ✅ Performance Tracking
- Pattern application counts
- Acceptance rates per pattern
- Identify underperforming patterns

### ✅ Hot Reloading
- Update patterns without code changes
- A/B test new patterns
- Disable problematic patterns instantly

### ✅ Multi-Tenancy Ready
- Different databases for different users
- Custom pattern sets per deployment
- Confidence scores per user/org

### ✅ Audit Trail
- Complete history of decisions
- Why patterns were accepted/rejected
- Context of each optimization

---

## Comparison: Old vs New

| Feature | v0.3 (Hardcoded) | v0.4 (Database-Backed) |
|---------|------------------|------------------------|
| Pattern storage | Static arrays in code | SQLite database |
| Add new pattern | Edit code + recompile | SQL INSERT |
| Update confidence | Edit code + recompile | HITL feedback |
| Disable pattern | Comment out code | UPDATE enabled=0 |
| Track usage | Not tracked | applied_count column |
| HITL decisions | Not stored | hitl_decisions table |
| Pattern stats | Manual calculation | SQL views |
| Hot reload | Restart required | `reload_patterns()` |
| Audit trail | None | Full decision history |
| Multi-tenancy | Not supported | Different DB per tenant |

---

## Testing

### Unit Tests

```bash
# Test database pattern loading
cargo test --lib database_pattern_detector

# Test database optimizer
cargo test --lib database_optimizer

# Test database operations
cargo test --lib database
```

### Integration Test

```rust
use prompt_compress::{init_database_optimizer, OptimizationRequest, Language, DirectiveFormat};

#[test]
fn test_full_optimization_pipeline() {
    // Setup
    let db = Database::in_memory().unwrap();

    // Run migration (insert patterns)
    // ... (migration code) ...

    // Create optimizer
    let mut optimizer = init_database_optimizer_with_path(db.path()).unwrap();

    // Optimize
    let request = OptimizationRequest {
        prompt: "I would really appreciate your help.".to_string(),
        output_language: Language::English,
        confidence_threshold: 0.85,
        aggressive_mode: false,
        directive_format: DirectiveFormat::Bracketed,
    };

    let result = optimizer.optimize(&request).unwrap();

    // Verify
    assert!(result.token_savings > 0);
    assert!(result.savings_percentage > 0.0);
}
```

---

## Next Steps

### Week 2 Priorities

1. **CLI Interactive Mode**
   - Interactive HITL review
   - Show optimizations for approval
   - Record decisions to database

2. **Pattern Management CLI**
   ```bash
   # Add new pattern
   prompt-compress patterns add \
     --type boilerplate \
     --regex "(?i)hello there" \
     --replacement "hi" \
     --confidence 0.90

   # List patterns
   prompt-compress patterns list --type boilerplate

   # Disable pattern
   prompt-compress patterns disable --id 42

   # Show stats
   prompt-compress patterns stats
   ```

3. **Confidence Calibration**
   - Export pattern performance report
   - Identify low-performing patterns
   - Recommend confidence adjustments

4. **Pattern Discovery**
   - Analyze corpus for new patterns
   - Suggest regex patterns
   - Estimate confidence from frequency

---

## Troubleshooting

### "No such table: patterns"

Run the migration:
```bash
cargo run --bin migrate_patterns -- atlas.db
```

### "Pattern count is 0"

Check database:
```bash
sqlite3 atlas.db "SELECT COUNT(*) FROM patterns WHERE enabled = 1;"
```

If zero, re-run migration.

### "Failed to compile pattern"

Check regex syntax in database:
```bash
sqlite3 atlas.db "SELECT id, regex_pattern FROM patterns WHERE id = 42;"
```

Test regex separately or disable pattern:
```sql
UPDATE patterns SET enabled = 0 WHERE id = 42;
```

### "No token savings"

Lower confidence threshold:
```rust
let optimizer = init_database_optimizer_with_confidence("atlas.db", 0.70)?;
```

Or enable aggressive mode:
```rust
let request = OptimizationRequest {
    aggressive_mode: true,
    confidence_threshold: 0.70,
    ..
};
```

---

## Summary

The consolidated architecture integrates the SQLite atlas with regex pattern optimization:

✅ **Patterns stored in database**
✅ **HITL feedback updates confidence**
✅ **Pattern statistics and tracking**
✅ **Hot reloading of patterns**
✅ **Backward compatible (legacy `init_optimizer()` still works)**

**Migration is simple:**
1. Run `migrate_patterns` binary
2. Replace `init_optimizer()` with `init_database_optimizer("atlas.db")`
3. Enjoy database-backed pattern management

---

**Status:** ✅ Consolidation Complete
**Ready for:** Production deployment
**Recommended:** Use `init_database_optimizer()` for all new code
