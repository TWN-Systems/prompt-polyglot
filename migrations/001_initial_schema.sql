-- Phase 3: Concept Atlas Database Schema
-- Purpose: Store Wikidata concepts, surface forms, and token costs per tokenizer

-- ==============================================================================
-- CONCEPTS TABLE
-- ==============================================================================
-- Core concept metadata from Wikidata
CREATE TABLE IF NOT EXISTS concepts (
    qid TEXT PRIMARY KEY NOT NULL,           -- e.g., "Q16917" for hospital
    label_en TEXT NOT NULL,                   -- English label: "hospital"
    description TEXT,                         -- Short description from Wikidata
    category TEXT,                            -- "medical", "technical", "general", etc.
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_concepts_category ON concepts(category);
CREATE INDEX idx_concepts_label ON concepts(label_en);

-- ==============================================================================
-- SURFACE_FORMS TABLE
-- ==============================================================================
-- All language variants and token costs for each concept, per tokenizer
CREATE TABLE IF NOT EXISTS surface_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    qid TEXT NOT NULL,                        -- Links to concepts.qid
    tokenizer_id TEXT NOT NULL,               -- "cl100k_base", "llama3", "claude"
    lang TEXT NOT NULL,                       -- ISO code: "en", "zh", "es", "ja"
    form TEXT NOT NULL,                       -- Actual text: "hospital", "医院", etc.
    token_count INTEGER NOT NULL,             -- Precomputed token count
    char_count INTEGER NOT NULL,              -- Character length (for metrics)
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    FOREIGN KEY (qid) REFERENCES concepts(qid) ON DELETE CASCADE,
    UNIQUE(qid, tokenizer_id, lang, form)     -- No duplicate forms per concept/tokenizer
);

CREATE INDEX idx_surface_forms_qid ON surface_forms(qid);
CREATE INDEX idx_surface_forms_tokenizer ON surface_forms(tokenizer_id);
CREATE INDEX idx_surface_forms_lang ON surface_forms(lang);
CREATE INDEX idx_surface_forms_tokens ON surface_forms(token_count);
CREATE INDEX idx_surface_forms_lookup ON surface_forms(qid, tokenizer_id);

-- ==============================================================================
-- OPTIMIZATION_CACHE TABLE
-- ==============================================================================
-- Cache optimized forms to avoid recomputation
CREATE TABLE IF NOT EXISTS optimization_cache (
    cache_key TEXT PRIMARY KEY NOT NULL,      -- SHA-256(original_text + tokenizer_id + policy)
    original_text TEXT NOT NULL,              -- Original phrase/word
    qid TEXT,                                 -- Resolved concept (NULL if no match)
    selected_form TEXT,                       -- Chosen optimized form (NULL if no optimization)
    token_count INTEGER,                      -- Token count of selected form
    confidence REAL,                          -- Optimization confidence score
    policy TEXT NOT NULL,                     -- "min_tokens", "cross_lingual", "same_lang"
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    hits INTEGER NOT NULL DEFAULT 0,          -- Cache hit counter

    FOREIGN KEY (qid) REFERENCES concepts(qid) ON DELETE SET NULL
);

CREATE INDEX idx_cache_original ON optimization_cache(original_text);
CREATE INDEX idx_cache_qid ON optimization_cache(qid);
CREATE INDEX idx_cache_hits ON optimization_cache(hits DESC);

-- ==============================================================================
-- CONCEPT_EMBEDDINGS TABLE
-- ==============================================================================
-- Optional: Store embeddings for fuzzy concept matching
CREATE TABLE IF NOT EXISTS concept_embeddings (
    qid TEXT PRIMARY KEY NOT NULL,
    embedding BLOB NOT NULL,                  -- Serialized float array
    dimension INTEGER NOT NULL,               -- 384, 768, 1536, etc.
    model_id TEXT NOT NULL,                   -- "all-MiniLM-L6-v2", etc.
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    FOREIGN KEY (qid) REFERENCES concepts(qid) ON DELETE CASCADE
);

CREATE INDEX idx_embeddings_model ON concept_embeddings(model_id);

-- ==============================================================================
-- METADATA TABLE
-- ==============================================================================
-- Schema version and statistics
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Initialize metadata
INSERT INTO metadata (key, value) VALUES
    ('schema_version', '1'),
    ('created_at', strftime('%s', 'now')),
    ('total_concepts', '0'),
    ('total_surface_forms', '0');

-- ==============================================================================
-- TRIGGERS
-- ==============================================================================
-- Update timestamps automatically
CREATE TRIGGER IF NOT EXISTS update_concepts_timestamp
AFTER UPDATE ON concepts
BEGIN
    UPDATE concepts SET updated_at = strftime('%s', 'now') WHERE qid = NEW.qid;
END;

-- Update metadata counts
CREATE TRIGGER IF NOT EXISTS increment_concepts_count
AFTER INSERT ON concepts
BEGIN
    UPDATE metadata SET value = CAST(CAST(value AS INTEGER) + 1 AS TEXT)
    WHERE key = 'total_concepts';
END;

CREATE TRIGGER IF NOT EXISTS increment_surface_forms_count
AFTER INSERT ON surface_forms
BEGIN
    UPDATE metadata SET value = CAST(CAST(value AS INTEGER) + 1 AS TEXT)
    WHERE key = 'total_surface_forms';
END;

CREATE TRIGGER IF NOT EXISTS decrement_concepts_count
AFTER DELETE ON concepts
BEGIN
    UPDATE metadata SET value = CAST(CAST(value AS INTEGER) - 1 AS TEXT)
    WHERE key = 'total_concepts';
END;

CREATE TRIGGER IF NOT EXISTS decrement_surface_forms_count
AFTER DELETE ON surface_forms
BEGIN
    UPDATE metadata SET value = CAST(CAST(value AS INTEGER) - 1 AS TEXT)
    WHERE key = 'total_surface_forms';
END;
