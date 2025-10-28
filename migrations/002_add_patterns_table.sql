-- Phase 3.5: Add Patterns Table for Regex-Based Optimizations
-- Purpose: Store regex patterns for boilerplate removal, filler words, etc.
-- This complements the concept atlas with pattern-based optimizations

-- ==============================================================================
-- PATTERNS TABLE
-- ==============================================================================
-- Store all regex-based optimization patterns
CREATE TABLE IF NOT EXISTS patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,               -- "boilerplate", "filler", "instruction", etc.
    regex_pattern TEXT NOT NULL,              -- The regex pattern
    replacement TEXT NOT NULL,                -- What to replace with (empty string for removal)
    base_confidence REAL NOT NULL,            -- Base confidence score (0.0-1.0)
    reasoning TEXT NOT NULL,                  -- Why this pattern is applied
    enabled INTEGER NOT NULL DEFAULT 1,       -- 1 = enabled, 0 = disabled
    applied_count INTEGER NOT NULL DEFAULT 0, -- How many times applied
    accepted_count INTEGER NOT NULL DEFAULT 0,-- How many times user accepted (HITL)
    rejected_count INTEGER NOT NULL DEFAULT 0,-- How many times user rejected (HITL)
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    UNIQUE(pattern_type, regex_pattern)       -- No duplicate patterns
);

CREATE INDEX idx_patterns_type ON patterns(pattern_type);
CREATE INDEX idx_patterns_confidence ON patterns(base_confidence DESC);
CREATE INDEX idx_patterns_enabled ON patterns(enabled);

-- ==============================================================================
-- HITL_DECISIONS TABLE
-- ==============================================================================
-- Track user decisions for confidence calibration
CREATE TABLE IF NOT EXISTS hitl_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id INTEGER NOT NULL,
    session_id TEXT NOT NULL,                 -- UUID of review session
    original_text TEXT NOT NULL,              -- The actual text matched
    optimized_text TEXT NOT NULL,             -- The proposed replacement
    decision TEXT NOT NULL,                   -- "accept", "reject", "modify"
    user_alternative TEXT,                    -- If modified, what user provided
    context_before TEXT,                      -- 50 chars before match
    context_after TEXT,                       -- 50 chars after match
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
);

CREATE INDEX idx_decisions_pattern ON hitl_decisions(pattern_id);
CREATE INDEX idx_decisions_session ON hitl_decisions(session_id);
CREATE INDEX idx_decisions_decision ON hitl_decisions(decision);

-- ==============================================================================
-- TRIGGERS
-- ==============================================================================
-- Update timestamps
CREATE TRIGGER IF NOT EXISTS update_patterns_timestamp
AFTER UPDATE ON patterns
BEGIN
    UPDATE patterns SET updated_at = strftime('%s', 'now') WHERE id = NEW.id;
END;

-- Update confidence scores based on HITL feedback
-- This implements Bayesian updating of confidence scores
CREATE TRIGGER IF NOT EXISTS update_pattern_confidence_on_decision
AFTER INSERT ON hitl_decisions
BEGIN
    -- Update accept/reject counts
    UPDATE patterns
    SET
        accepted_count = accepted_count + (CASE WHEN NEW.decision = 'accept' THEN 1 ELSE 0 END),
        rejected_count = rejected_count + (CASE WHEN NEW.decision = 'reject' THEN 1 ELSE 0 END),
        -- Bayesian update: adjust confidence based on feedback
        -- Formula: new_conf = (base_conf * prior_weight + success_rate * feedback_weight) / total_weight
        base_confidence = CASE
            WHEN (accepted_count + rejected_count + 1) >= 10 THEN
                -- After 10+ feedbacks, use empirical success rate
                CAST(accepted_count + (CASE WHEN NEW.decision = 'accept' THEN 1 ELSE 0 END) AS REAL) /
                CAST(accepted_count + rejected_count + 1 AS REAL)
            ELSE
                -- Before 10 feedbacks, blend base confidence with early feedback
                (base_confidence * 10.0 +
                 CAST(accepted_count + (CASE WHEN NEW.decision = 'accept' THEN 1 ELSE 0 END) AS REAL)) /
                (10.0 + CAST(accepted_count + rejected_count + 1 AS REAL))
            END
    WHERE id = NEW.pattern_id;
END;

-- ==============================================================================
-- VIEWS
-- ==============================================================================
-- View: Active patterns ordered by confidence
CREATE VIEW IF NOT EXISTS active_patterns AS
SELECT
    id,
    pattern_type,
    regex_pattern,
    replacement,
    base_confidence,
    reasoning,
    applied_count,
    accepted_count,
    rejected_count,
    CASE
        WHEN (accepted_count + rejected_count) > 0
        THEN CAST(accepted_count AS REAL) / CAST(accepted_count + rejected_count AS REAL)
        ELSE base_confidence
    END AS empirical_confidence
FROM patterns
WHERE enabled = 1
ORDER BY base_confidence DESC;

-- View: Pattern performance statistics
CREATE VIEW IF NOT EXISTS pattern_stats AS
SELECT
    pattern_type,
    COUNT(*) as total_patterns,
    AVG(base_confidence) as avg_confidence,
    SUM(applied_count) as total_applications,
    SUM(accepted_count) as total_accepted,
    SUM(rejected_count) as total_rejected,
    CASE
        WHEN SUM(accepted_count + rejected_count) > 0
        THEN CAST(SUM(accepted_count) AS REAL) / CAST(SUM(accepted_count + rejected_count) AS REAL)
        ELSE 0.0
    END AS overall_acceptance_rate
FROM patterns
WHERE enabled = 1
GROUP BY pattern_type;

-- Update metadata
INSERT INTO metadata (key, value) VALUES ('patterns_schema_version', '2');
