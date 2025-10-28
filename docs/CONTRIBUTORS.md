# Contributors Guide

Welcome to the **prompt-compress** project! This guide will help you understand the codebase, contribute effectively, and extend the project.

## Table of Contents

1. [Project Architecture](#project-architecture)
2. [Codebase Structure](#codebase-structure)
3. [Database Schema](#database-schema)
4. [Wikidata Integration](#wikidata-integration)
5. [Adding New Language Pairs (Q16917)](#adding-new-language-pairs-q16917)
6. [CLI Tools](#cli-tools)
7. [REST API](#rest-api)
8. [Pattern System](#pattern-system)
9. [Creating MCP Servers](#creating-mcp-servers)
10. [Creating Claude Skills](#creating-claude-skills)
11. [Testing](#testing)
12. [Development Workflow](#development-workflow)

---

## Project Architecture

`prompt-compress` is a Rust-based multilingual prompt optimization tool with three core components:

### 1. Pattern-Based Optimizer
- Regex-based pattern matching for boilerplate, filler words, synonyms
- Stored in SQLite database with Bayesian confidence scoring
- HITL (Human-in-the-Loop) feedback integration

### 2. Concept Atlas System
- Wikidata-backed multilingual concept resolution
- Cross-language token optimization using QIDs (e.g., Q16917 = "hospital")
- Surface form selection based on token efficiency

### 3. REST API Server
- Actix-web based HTTP server
- Webhook support for async optimization
- Health checks and metrics

### High-Level Flow

```
User Input Prompt
       ↓
[1. Protected Region Detection] → Preserve code, URLs, templates
       ↓
[2. Pattern Detection] → Database-backed regex patterns
       ↓
[3. Concept Resolution] → Wikidata QID mapping
       ↓
[4. Surface Selection] → Choose optimal language form
       ↓
[5. Confidence Scoring] → Bayesian scoring + HITL feedback
       ↓
[6. Apply Optimizations] → Auto-apply high confidence (≥85%)
       ↓
[7. Output Language Directive] → Append [output_language: ...]
       ↓
Optimized Prompt
```

---

## Codebase Structure

```
prompt-polyglot/
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── main.rs                   # CLI entry point
│   ├── api.rs                    # REST API handlers (Actix-web)
│   ├── models.rs                 # Data structures (Optimization, Config, etc.)
│   ├── optimizer.rs              # Legacy hardcoded pattern optimizer
│   ├── patterns.rs               # Pattern definitions (deprecated)
│   ├── confidence.rs             # Bayesian confidence calculation
│   ├── tokenizer.rs              # OpenAI tokenizer (tiktoken)
│   │
│   ├── tokenizer_registry.rs    # Multi-tokenizer support (GPT, Claude, Llama)
│   ├── database.rs               # SQLite connection and queries
│   ├── concept_resolver.rs      # Wikidata concept resolution
│   ├── surface_selector.rs      # Optimal surface form selection
│   ├── protected_regions.rs     # Code/URL protection
│   ├── concept_optimizer.rs     # Concept-based optimization
│   ├── database_pattern_detector.rs  # Database pattern matching
│   ├── database_optimizer.rs    # Database-backed optimizer (RECOMMENDED)
│   │
│   └── bin/
│       ├── server.rs             # API server binary
│       └── migrate_patterns.rs   # Migrate patterns to DB
│
├── migrations/
│   ├── 001_initial_schema.sql   # Concept atlas schema
│   └── 002_add_patterns_table.sql # Patterns + HITL tables
│
├── data/
│   └── priors.json               # Bayesian priors (optional)
│
├── docs/
│   ├── CONTRIBUTORS.md           # This file
│   ├── CLAUDE.md                 # Full project specification
│   ├── CONSOLIDATED-ARCHITECTURE.md # Architecture details
│   ├── QUICKSTART.md             # Quick start guide
│   └── ... (other docs)
│
├── tests/                        # Integration tests
├── examples/                     # Example prompts
├── scripts/                      # Utility scripts
│   └── populate_sample_data.py  # Populate DB with sample data
│
├── Cargo.toml                    # Rust dependencies
└── README.md                     # Project overview
```

### Key Modules

| Module | Purpose | Status |
|--------|---------|--------|
| `database_optimizer.rs` | **RECOMMENDED** - Database-backed pattern optimizer | Production |
| `optimizer.rs` | Legacy hardcoded patterns | Deprecated |
| `database.rs` | SQLite connection, migrations, queries | Production |
| `concept_resolver.rs` | Wikidata concept resolution with caching | Production |
| `surface_selector.rs` | Token-efficient surface form selection | Production |
| `database_pattern_detector.rs` | Load patterns from DB, apply regex | Production |
| `api.rs` | REST API endpoints (Actix-web) | Production |

---

## Database Schema

### 1. Concepts Table (`concepts`)

Stores Wikidata concepts (QIDs).

```sql
CREATE TABLE concepts (
    qid TEXT PRIMARY KEY NOT NULL,           -- e.g., "Q16917"
    label_en TEXT NOT NULL,                   -- English label: "hospital"
    description TEXT,                         -- "healthcare institution"
    category TEXT,                            -- "medical", "technical", etc.
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**Example Row:**
```
qid: "Q16917"
label_en: "hospital"
description: "healthcare institution providing patient treatment"
category: "medical"
```

### 2. Surface Forms Table (`surface_forms`)

Stores language variants and token counts for each concept.

```sql
CREATE TABLE surface_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    qid TEXT NOT NULL,                        -- Links to concepts.qid
    tokenizer_id TEXT NOT NULL,               -- "cl100k_base", "llama3", "claude"
    lang TEXT NOT NULL,                       -- ISO code: "en", "zh", "es", "ja"
    form TEXT NOT NULL,                       -- Actual text: "hospital", "医院"
    token_count INTEGER NOT NULL,             -- Precomputed token count
    char_count INTEGER NOT NULL,              -- Character length
    created_at INTEGER NOT NULL,
    FOREIGN KEY (qid) REFERENCES concepts(qid) ON DELETE CASCADE,
    UNIQUE(qid, tokenizer_id, lang, form)
);
```

**Example Rows:**
```
qid: "Q16917", tokenizer: "cl100k_base", lang: "en", form: "hospital", tokens: 1
qid: "Q16917", tokenizer: "cl100k_base", lang: "zh", form: "医院", tokens: 1
qid: "Q16917", tokenizer: "cl100k_base", lang: "es", form: "hospital", tokens: 1
```

### 3. Patterns Table (`patterns`)

Stores regex-based optimization patterns.

```sql
CREATE TABLE patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,               -- "boilerplate", "filler", etc.
    regex_pattern TEXT NOT NULL,              -- Regex pattern
    replacement TEXT NOT NULL,                -- Replacement text (empty = remove)
    base_confidence REAL NOT NULL,            -- Base confidence (0.0-1.0)
    reasoning TEXT NOT NULL,                  -- Explanation
    enabled INTEGER NOT NULL DEFAULT 1,       -- 1 = enabled, 0 = disabled
    applied_count INTEGER NOT NULL DEFAULT 0, -- Usage count
    accepted_count INTEGER NOT NULL DEFAULT 0,-- HITL accepts
    rejected_count INTEGER NOT NULL DEFAULT 0,-- HITL rejects
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(pattern_type, regex_pattern)
);
```

**Example Row:**
```
pattern_type: "boilerplate"
regex_pattern: "I would (really )?appreciate (it )?if you could"
replacement: ""
base_confidence: 0.97
reasoning: "Polite boilerplate with no semantic value"
enabled: 1
```

### 4. HITL Decisions Table (`hitl_decisions`)

Tracks user feedback for Bayesian confidence updates.

```sql
CREATE TABLE hitl_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id INTEGER NOT NULL,
    session_id TEXT NOT NULL,                 -- UUID
    original_text TEXT NOT NULL,              -- Matched text
    optimized_text TEXT NOT NULL,             -- Proposed replacement
    decision TEXT NOT NULL,                   -- "accept", "reject", "modify"
    user_alternative TEXT,                    -- If modified
    context_before TEXT,                      -- 50 chars before
    context_after TEXT,                       -- 50 chars after
    created_at INTEGER NOT NULL,
    FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
);
```

### 5. Optimization Cache (`optimization_cache`)

Caches concept resolutions to avoid recomputation.

```sql
CREATE TABLE optimization_cache (
    cache_key TEXT PRIMARY KEY NOT NULL,      -- SHA-256(text + tokenizer + policy)
    original_text TEXT NOT NULL,
    qid TEXT,                                 -- Resolved concept (NULL if no match)
    selected_form TEXT,                       -- Optimized form
    token_count INTEGER,                      -- Token count
    confidence REAL,                          -- Confidence score
    policy TEXT NOT NULL,                     -- "min_tokens", "cross_lingual", etc.
    created_at INTEGER NOT NULL,
    hits INTEGER NOT NULL DEFAULT 0,          -- Cache hit counter
    FOREIGN KEY (qid) REFERENCES concepts(qid) ON DELETE SET NULL
);
```

---

## Wikidata Integration

### What is Wikidata?

[Wikidata](https://www.wikidata.org/) is a free, collaborative knowledge base with structured data about millions of concepts. Each concept has a unique **QID** (e.g., `Q16917` = "hospital") and labels in 300+ languages.

### Why Wikidata?

- **Multilingual**: Get equivalent terms in any language (e.g., "hospital" → "医院" → "hospital")
- **Canonical**: One QID = one concept, regardless of language
- **Token-efficient**: Choose the shortest form across languages

### Wikidata API Usage

**Endpoint:** `https://www.wikidata.org/w/api.php`

**Get Concept by QID:**
```bash
curl "https://www.wikidata.org/w/api.php?action=wbgetentities&ids=Q16917&format=json&languages=en|zh|es|ja"
```

**Response:**
```json
{
  "entities": {
    "Q16917": {
      "labels": {
        "en": { "value": "hospital" },
        "zh": { "value": "医院" },
        "es": { "value": "hospital" },
        "ja": { "value": "病院" }
      },
      "descriptions": {
        "en": { "value": "healthcare institution providing patient treatment" }
      }
    }
  }
}
```

**Search for Concepts:**
```bash
curl "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=hospital&language=en&format=json"
```

### Integration in Code

See `src/concept_resolver.rs`:

```rust
pub struct ConceptResolver {
    db: Arc<Database>,
    tokenizer: TokenizerRegistry,
    cache: LruCache<String, Option<String>>, // Text → QID
}

impl ConceptResolver {
    /// Resolve text to Wikidata QID
    pub fn resolve(&mut self, text: &str) -> Result<Option<String>> {
        // 1. Check cache
        if let Some(qid) = self.cache.get(text) {
            return Ok(qid.clone());
        }

        // 2. Check database
        if let Some(qid) = self.db.find_qid_by_label(text)? {
            self.cache.put(text.to_string(), Some(qid.clone()));
            return Ok(Some(qid));
        }

        // 3. Query Wikidata API (not implemented in v1.0)
        Ok(None)
    }
}
```

---

## Adding New Language Pairs (Q16917)

### Step 1: Fetch Wikidata Concept

Use the Wikidata API to get labels for a concept:

```python
import requests

def fetch_concept(qid, languages=["en", "zh", "es", "ja", "fr", "de"]):
    url = "https://www.wikidata.org/w/api.php"
    params = {
        "action": "wbgetentities",
        "ids": qid,
        "format": "json",
        "languages": "|".join(languages)
    }
    response = requests.get(url, params=params)
    data = response.json()

    entity = data["entities"][qid]
    labels = {lang: label["value"] for lang, label in entity["labels"].items()}
    description = entity["descriptions"].get("en", {}).get("value", "")

    return labels, description

# Example: Q16917 = hospital
labels, desc = fetch_concept("Q16917")
print(labels)  # {'en': 'hospital', 'zh': '医院', 'es': 'hospital', ...}
```

### Step 2: Calculate Token Counts

Use the tokenizer to count tokens for each language variant:

```python
import tiktoken

tokenizer = tiktoken.get_encoding("cl100k_base")

def count_tokens(text):
    return len(tokenizer.encode(text))

for lang, form in labels.items():
    tokens = count_tokens(form)
    print(f"{lang}: {form} → {tokens} tokens")
```

Output:
```
en: hospital → 1 token
zh: 医院 → 1 token
es: hospital → 1 token
ja: 病院 → 1 token
```

### Step 3: Insert into Database

```sql
-- Insert concept
INSERT INTO concepts (qid, label_en, description, category)
VALUES ('Q16917', 'hospital', 'healthcare institution', 'medical');

-- Insert surface forms
INSERT INTO surface_forms (qid, tokenizer_id, lang, form, token_count, char_count)
VALUES
    ('Q16917', 'cl100k_base', 'en', 'hospital', 1, 8),
    ('Q16917', 'cl100k_base', 'zh', '医院', 1, 2),
    ('Q16917', 'cl100k_base', 'es', 'hospital', 1, 8),
    ('Q16917', 'cl100k_base', 'ja', '病院', 1, 2);
```

### Step 4: Use Python Script

Use `scripts/populate_sample_data.py` as a template:

```python
import sqlite3
import tiktoken
import requests

def add_concept(qid, category, tokenizer_name="cl100k_base"):
    """Add a Wikidata concept to the database"""
    tokenizer = tiktoken.get_encoding(tokenizer_name)

    # Fetch from Wikidata
    labels, description = fetch_concept(qid)

    # Insert into DB
    conn = sqlite3.connect("atlas.db")
    cursor = conn.cursor()

    # Insert concept
    cursor.execute(
        "INSERT OR IGNORE INTO concepts (qid, label_en, description, category) VALUES (?, ?, ?, ?)",
        (qid, labels.get("en", ""), description, category)
    )

    # Insert surface forms
    for lang, form in labels.items():
        token_count = len(tokenizer.encode(form))
        char_count = len(form)
        cursor.execute(
            "INSERT OR IGNORE INTO surface_forms (qid, tokenizer_id, lang, form, token_count, char_count) VALUES (?, ?, ?, ?, ?, ?)",
            (qid, tokenizer_name, lang, form, token_count, char_count)
        )

    conn.commit()
    conn.close()
    print(f"Added {qid}: {labels.get('en')} with {len(labels)} language variants")

# Example usage
add_concept("Q16917", "medical")  # hospital
add_concept("Q9842", "technical")  # computer
add_concept("Q11424", "technical")  # software
```

### Step 5: Verify

```bash
sqlite3 atlas.db "SELECT * FROM concepts WHERE qid = 'Q16917';"
sqlite3 atlas.db "SELECT * FROM surface_forms WHERE qid = 'Q16917';"
```

---

## CLI Tools

### Binary: `prompt-compress`

Located in `src/main.rs`.

**Usage:**

```bash
# Optimize a prompt
prompt-compress optimize \
  --input prompt.txt \
  --output optimized.txt \
  --output-lang english \
  --threshold 0.85

# Aggressive mode (lower threshold)
prompt-compress optimize \
  --input prompt.txt \
  --aggressive

# Analyze without optimizing
prompt-compress analyze \
  --input prompt.txt \
  --report report.json

# Batch processing
prompt-compress batch \
  --input prompts/ \
  --output optimized/
```

### Binary: `prompt-compress-server`

Located in `src/bin/server.rs`.

**Usage:**

```bash
# Start API server (default: 0.0.0.0:8080)
prompt-compress-server

# With custom port
PORT=3000 prompt-compress-server

# With logging
RUST_LOG=info prompt-compress-server
```

### Binary: `migrate_patterns`

Located in `src/bin/migrate_patterns.rs`.

**Usage:**

```bash
# Migrate hardcoded patterns to database
cargo run --bin migrate_patterns -- atlas.db
```

This migrates 102 patterns from `src/patterns.rs` to the `patterns` table.

---

## REST API

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/health` | Health check |
| POST | `/api/v1/optimize` | Optimize prompt |
| POST | `/api/v1/webhook/optimize` | Optimize with callback |
| POST | `/api/v1/analyze` | Analyze without optimizing |

### Example: Optimize Prompt

**Request:**
```bash
curl -X POST http://localhost:8080/api/v1/optimize \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I would really appreciate it if you could help me with this task.",
    "output_language": "english",
    "confidence_threshold": 0.85,
    "aggressive_mode": false
  }'
```

**Response:**
```json
{
  "result": {
    "original_prompt": "I would really appreciate it if you could help me with this task.",
    "optimized_prompt": "Help me with this task.\n\n[output_language: english]",
    "original_tokens": 18,
    "optimized_tokens": 12,
    "token_savings": 6,
    "savings_percentage": 33.3,
    "optimizations": [
      {
        "id": "opt_12345",
        "optimization_type": "BoilerplateRemoval",
        "original_text": "I would really appreciate it if you could",
        "optimized_text": "",
        "token_savings": 8,
        "confidence": 0.97,
        "requires_review": false,
        "reasoning": "Polite boilerplate with no semantic value"
      }
    ],
    "requires_review": [],
    "output_language": "english"
  },
  "review_session_id": null
}
```

### Example: Webhook with Callback

**Request:**
```bash
curl -X POST http://localhost:8080/api/v1/webhook/optimize \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Please analyze this code carefully.",
    "output_language": "english",
    "callback_url": "https://your-app.com/webhook/receive"
  }'
```

**Response (immediate):**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "optimized_prompt": "Analyze this code.\n\n[output_language: english]",
  "original_tokens": 6,
  "optimized_tokens": 5,
  "token_savings": 1,
  "savings_percentage": 16.7,
  "status": "completed"
}
```

**Callback (POSTed to your URL):**
Same JSON as immediate response.

---

## Pattern System

### Pattern Types

| Type | Description | Example |
|------|-------------|---------|
| `boilerplate` | Polite filler phrases | "I would appreciate if..." |
| `filler_word` | Unnecessary modifiers | "really", "very", "quite" |
| `synonym` | Redundant pairs | "analyze and examine" |
| `instruction` | Verbose commands | "I want you to provide" → "Provide" |
| `mandarin` | Token-efficient Chinese | "verify" → "验证" |

### Adding New Patterns

**Manual SQL:**

```sql
INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
VALUES (
    'boilerplate',
    'if you could please',
    '',
    0.95,
    'Polite request with no semantic value'
);
```

**Using Rust API:**

```rust
use prompt_compress::Database;

let db = Database::open("atlas.db")?;
db.add_pattern(
    "boilerplate",
    "if you could please",
    "",
    0.95,
    "Polite request with no semantic value"
)?;
```

### HITL Feedback

When a user accepts/rejects an optimization, update the confidence:

```rust
db.record_hitl_decision(
    pattern_id,
    session_id,
    original_text,
    optimized_text,
    "accept",  // or "reject", "modify"
    None,      // user_alternative
    context_before,
    context_after,
)?;
```

The trigger `update_pattern_confidence_on_decision` automatically updates the pattern's `base_confidence` using Bayesian inference.

---

## Creating MCP Servers

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/) is Anthropic's standard for connecting AI assistants to external tools and data sources.

### What is an MCP Server?

An MCP server exposes **tools** and **resources** that Claude can use during conversations. Tools are like function calls; resources are like files or databases.

### prompt-compress as an MCP Server

You can wrap the `prompt-compress` API as an MCP server to let Claude optimize prompts on-the-fly.

#### Step 1: Install MCP SDK

```bash
npm install @anthropic-ai/sdk @modelcontextprotocol/sdk
```

#### Step 2: Create MCP Server

`mcp-server.js`:

```javascript
import { MCPServer } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import axios from 'axios';

const API_BASE = process.env.PROMPT_COMPRESS_API || 'http://localhost:8080/api/v1';

// Create MCP server
const server = new MCPServer({
  name: 'prompt-compress',
  version: '1.0.0',
});

// Register tool: optimize_prompt
server.tool(
  'optimize_prompt',
  'Optimize a verbose prompt by removing boilerplate and compressing',
  {
    prompt: {
      type: 'string',
      description: 'The prompt to optimize',
      required: true,
    },
    output_language: {
      type: 'string',
      description: 'Output language (english or mandarin)',
      default: 'english',
    },
    aggressive_mode: {
      type: 'boolean',
      description: 'Use aggressive compression (lower threshold)',
      default: false,
    },
  },
  async ({ prompt, output_language, aggressive_mode }) => {
    const response = await axios.post(`${API_BASE}/optimize`, {
      prompt,
      output_language: output_language || 'english',
      aggressive_mode: aggressive_mode || false,
      confidence_threshold: 0.85,
    });

    const result = response.data.result;
    return {
      content: [
        {
          type: 'text',
          text: `Optimized Prompt:\n${result.optimized_prompt}\n\nSavings: ${result.token_savings} tokens (${result.savings_percentage.toFixed(1)}%)`,
        },
      ],
    };
  }
);

// Register tool: analyze_prompt
server.tool(
  'analyze_prompt',
  'Analyze a prompt and show optimization opportunities without applying them',
  {
    prompt: {
      type: 'string',
      description: 'The prompt to analyze',
      required: true,
    },
  },
  async ({ prompt }) => {
    const response = await axios.post(`${API_BASE}/analyze`, {
      prompt,
      output_language: 'english',
    });

    const result = response.data.result;
    const optimizations = result.optimizations
      .map((opt) => `- ${opt.optimization_type}: "${opt.original_text}" → "${opt.optimized_text}" (${opt.token_savings} tokens, confidence: ${opt.confidence})`)
      .join('\n');

    return {
      content: [
        {
          type: 'text',
          text: `Optimization Opportunities:\n${optimizations}\n\nTotal Savings: ${result.token_savings} tokens (${result.savings_percentage.toFixed(1)}%)`,
        },
      ],
    };
  }
);

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
console.error('MCP server running on stdio');
```

#### Step 3: Register with Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "prompt-compress": {
      "command": "node",
      "args": ["/path/to/mcp-server.js"],
      "env": {
        "PROMPT_COMPRESS_API": "http://localhost:8080/api/v1"
      }
    }
  }
}
```

#### Step 4: Use in Claude

Restart Claude Desktop. You can now use the tools:

```
User: Optimize this prompt: "I would really appreciate if you could please analyze this code and provide a detailed explanation."

Claude: I'll use the optimize_prompt tool to compress this prompt.
[Uses optimize_prompt tool]
Result: "Analyze this code: detailed explanation."
Savings: 12 tokens (60.0%)
```

### MCP Resources

You can also expose the database as a resource:

```javascript
server.resource(
  'pattern://boilerplate',
  'List of boilerplate removal patterns',
  'text/plain',
  async () => {
    const response = await axios.get(`${API_BASE}/patterns?type=boilerplate`);
    return response.data.patterns.map(p => p.regex_pattern).join('\n');
  }
);
```

---

## Creating Claude Skills

**Claude Skills** are reusable capabilities that can be invoked in Claude conversations.

### What is a Claude Skill?

A skill is a YAML file that defines:
1. **Trigger**: When to activate (e.g., user mentions "optimize prompt")
2. **Instructions**: What Claude should do
3. **Tools**: External tools to use (like MCP tools)

### Example: Prompt Optimizer Skill

`prompt-optimizer.skill.yaml`:

```yaml
name: prompt_optimizer
description: Optimize verbose prompts by removing boilerplate and compressing text
version: 1.0.0

triggers:
  - pattern: "optimize (this|my|the) prompt"
  - pattern: "compress (this|my|the) prompt"
  - pattern: "remove boilerplate from"

instructions: |
  When the user asks you to optimize a prompt:

  1. Identify the prompt to optimize (usually in quotes or code blocks)
  2. Use the `optimize_prompt` tool from the prompt-compress MCP server
  3. Show the optimized version and token savings
  4. Ask if the user wants to use the optimized version or make adjustments

tools:
  - name: optimize_prompt
    server: prompt-compress
    description: Optimize a verbose prompt

  - name: analyze_prompt
    server: prompt-compress
    description: Analyze optimization opportunities

examples:
  - input: 'Optimize this prompt: "I would really appreciate if you could please help me with this task."'
    output: |
      I'll optimize that prompt for you.

      Original: "I would really appreciate if you could please help me with this task."
      Optimized: "Help me with this task."

      Savings: 8 tokens (44.4%)

      Would you like to use this optimized version?

  - input: "Analyze this prompt for optimization opportunities: [long prompt]"
    output: |
      I've analyzed the prompt. Here are the optimization opportunities:

      - Remove boilerplate: "I would appreciate if you could" (8 tokens)
      - Remove filler: "really", "very" (2 tokens)
      - Consolidate synonyms: "analyze and examine" → "analyze" (2 tokens)

      Total potential savings: 12 tokens (20.5%)

      Would you like me to apply these optimizations?
```

### Using the Skill

In a Claude conversation:

```
User: Optimize this prompt: "I would really appreciate if you could please take the time to analyze this code carefully."

Claude: [Activates prompt_optimizer skill]
I'll optimize that prompt for you using the prompt-compress tool.

[Uses optimize_prompt tool]

Original: "I would really appreciate if you could please take the time to analyze this code carefully."
Optimized: "Analyze this code."

Savings: 14 tokens (63.6%)

The optimized version is much more concise while preserving the core instruction. Would you like to use this version?
```

### Advanced: Context-Aware Optimization

```yaml
name: context_aware_optimizer
description: Optimize prompts based on the conversation context

instructions: |
  Analyze the conversation context to determine:

  1. Output language: If the user has been speaking in Mandarin, use mandarin mode
  2. Aggression level: For casual conversations, use aggressive mode; for formal, use standard
  3. Preservation: If code or technical terms are present, preserve protected regions

  Then use the appropriate optimization settings.

tools:
  - name: optimize_prompt
    server: prompt-compress
```

---

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module
cargo test patterns
cargo test database_optimizer

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Integration Tests

Located in `tests/`:

```rust
// tests/integration_test.rs
use prompt_compress::{init_database_optimizer, OptimizationRequest, Language};

#[test]
fn test_optimize_boilerplate() {
    let optimizer = init_database_optimizer("test_atlas.db").unwrap();
    let request = OptimizationRequest {
        prompt: "I would really appreciate if you could help me.".to_string(),
        output_language: Language::English,
        confidence_threshold: 0.85,
        aggressive_mode: false,
    };

    let result = optimizer.optimize(&request).unwrap();
    assert!(result.token_savings > 0);
    assert!(result.savings_percentage > 10.0);
}
```

### Python Simulation Tests

If you can't build Rust (dependency issues), use Python simulations:

```bash
python3 manual_test.py
python3 test_optimization_goals.py
```

These simulate the optimization logic in Python.

### API Tests

```bash
# Start server
cargo run --bin prompt-compress-server &

# Test health
curl http://localhost:8080/api/v1/health

# Test optimization
curl -X POST http://localhost:8080/api/v1/optimize \
  -H "Content-Type: application/json" \
  -d '{"prompt": "I would really appreciate if you could help me.", "output_language": "english"}'
```

---

## Development Workflow

### 1. Clone and Build

```bash
git clone https://github.com/your-org/prompt-polyglot.git
cd prompt-polyglot
cargo build --release
```

### 2. Set Up Database

```bash
# Migrate patterns to database
cargo run --bin migrate_patterns -- atlas.db

# Populate sample data
python3 scripts/populate_sample_data.py
```

### 3. Run Tests

```bash
cargo test
```

### 4. Start Development Server

```bash
RUST_LOG=info cargo run --bin prompt-compress-server
```

### 5. Make Changes

- **Add patterns**: Insert into `patterns` table
- **Add concepts**: Fetch from Wikidata, insert into `concepts` and `surface_forms`
- **Add optimizations**: Modify `database_optimizer.rs` or `concept_optimizer.rs`

### 6. Run Tests Again

```bash
cargo test
```

### 7. Commit and Push

```bash
git add .
git commit -m "feat: Add new boilerplate patterns"
git push origin your-branch
```

### 8. Submit PR

Submit a pull request with:
- **Description**: What you changed and why
- **Tests**: Include test results
- **Examples**: Show before/after optimization examples

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/your-org/prompt-polyglot/issues)
- **Docs**: See `/docs` folder for more documentation
- **Specification**: [CLAUDE.md](./CLAUDE.md) for full project spec

---

**Thank you for contributing to prompt-compress!**
