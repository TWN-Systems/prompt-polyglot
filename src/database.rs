/// Phase 3: Database connection and schema management for Concept Atlas
///
/// Purpose: Manage SQLite database connection, migrations, and provide
/// data access layer for concepts, surface forms, and optimization cache.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

/// Database connection manager
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("Failed to open database at {:?}", path.as_ref()))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to create in-memory database")?;
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize schema from migration file
    fn initialize_schema(&self) -> Result<()> {
        // Check if metadata table exists
        let table_exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='metadata'",
                [],
                |row| row.get(0),
            )
            .context("Failed to check if metadata table exists")?;

        if table_exists > 0 {
            // Schema already initialized
            return Ok(());
        }

        // Load and execute schema migration
        let schema_sql = include_str!("../migrations/001_initial_schema.sql");
        self.conn
            .execute_batch(schema_sql)
            .context("Failed to execute schema migration")?;

        Ok(())
    }

    /// Get current schema version
    pub fn schema_version(&self) -> Result<String> {
        let version: String = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .context("Failed to get schema version")?;
        Ok(version)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let total_concepts_str: String = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'total_concepts'",
                [],
                |row| row.get(0),
            )?;
        let total_concepts = total_concepts_str.parse::<usize>().unwrap_or(0);

        let total_surface_forms_str: String = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'total_surface_forms'",
                [],
                |row| row.get(0),
            )?;
        let total_surface_forms = total_surface_forms_str.parse::<usize>().unwrap_or(0);

        let cache_size: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM optimization_cache", [], |row| {
                row.get(0)
            })?;

        Ok(DatabaseStats {
            total_concepts,
            total_surface_forms,
            cache_size: cache_size as usize,
        })
    }

    /// Insert or update concept
    pub fn upsert_concept(&self, concept: &Concept) -> Result<()> {
        self.conn.execute(
            "INSERT INTO concepts (qid, label_en, description, category)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(qid) DO UPDATE SET
                label_en = excluded.label_en,
                description = excluded.description,
                category = excluded.category,
                updated_at = strftime('%s', 'now')",
            rusqlite::params![
                &concept.qid,
                &concept.label_en,
                &concept.description,
                &concept.category,
            ],
        )?;
        Ok(())
    }

    /// Get concept by Q-ID
    pub fn get_concept(&self, qid: &str) -> Result<Option<Concept>> {
        let concept = self
            .conn
            .query_row(
                "SELECT qid, label_en, description, category FROM concepts WHERE qid = ?1",
                [qid],
                |row| {
                    Ok(Concept {
                        qid: row.get(0)?,
                        label_en: row.get(1)?,
                        description: row.get(2)?,
                        category: row.get(3)?,
                    })
                },
            )
            .optional()?;
        Ok(concept)
    }

    /// Find concept by English label (exact match)
    pub fn find_concept_by_label(&self, label: &str) -> Result<Option<Concept>> {
        let concept = self
            .conn
            .query_row(
                "SELECT qid, label_en, description, category FROM concepts
                 WHERE LOWER(label_en) = LOWER(?1)",
                [label],
                |row| {
                    Ok(Concept {
                        qid: row.get(0)?,
                        label_en: row.get(1)?,
                        description: row.get(2)?,
                        category: row.get(3)?,
                    })
                },
            )
            .optional()?;
        Ok(concept)
    }

    /// Insert surface form
    pub fn insert_surface_form(&self, form: &SurfaceForm) -> Result<()> {
        self.conn.execute(
            "INSERT INTO surface_forms (qid, tokenizer_id, lang, form, token_count, char_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(qid, tokenizer_id, lang, form) DO UPDATE SET
                token_count = excluded.token_count,
                char_count = excluded.char_count",
            rusqlite::params![
                &form.qid,
                &form.tokenizer_id,
                &form.lang,
                &form.form,
                form.token_count as i64,
                form.char_count as i64,
            ],
        )?;
        Ok(())
    }

    /// Get all surface forms for a concept
    pub fn get_surface_forms(&self, qid: &str, tokenizer_id: &str) -> Result<Vec<SurfaceForm>> {
        let mut stmt = self.conn.prepare(
            "SELECT qid, tokenizer_id, lang, form, token_count, char_count
             FROM surface_forms
             WHERE qid = ?1 AND tokenizer_id = ?2
             ORDER BY token_count ASC",
        )?;

        let forms = stmt
            .query_map([qid, tokenizer_id], |row| {
                Ok(SurfaceForm {
                    qid: row.get(0)?,
                    tokenizer_id: row.get(1)?,
                    lang: row.get(2)?,
                    form: row.get(3)?,
                    token_count: row.get::<_, i64>(4)? as usize,
                    char_count: row.get::<_, i64>(5)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(forms)
    }

    /// Get cheapest surface form for concept
    pub fn get_cheapest_form(&self, qid: &str, tokenizer_id: &str) -> Result<Option<SurfaceForm>> {
        let form = self
            .conn
            .query_row(
                "SELECT qid, tokenizer_id, lang, form, token_count, char_count
                 FROM surface_forms
                 WHERE qid = ?1 AND tokenizer_id = ?2
                 ORDER BY token_count ASC
                 LIMIT 1",
                [qid, tokenizer_id],
                |row| {
                    Ok(SurfaceForm {
                        qid: row.get(0)?,
                        tokenizer_id: row.get(1)?,
                        lang: row.get(2)?,
                        form: row.get(3)?,
                        token_count: row.get::<_, i64>(4)? as usize,
                        char_count: row.get::<_, i64>(5)? as usize,
                    })
                },
            )
            .optional()?;
        Ok(form)
    }

    /// Get reference to underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Load all active patterns from database
    pub fn load_patterns(&self) -> Result<Vec<PatternRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, regex_pattern, replacement, base_confidence, reasoning,
                    applied_count, accepted_count, rejected_count
             FROM patterns
             WHERE enabled = 1
             ORDER BY base_confidence DESC"
        )?;

        let patterns = stmt
            .query_map([], |row| {
                Ok(PatternRecord {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    regex_pattern: row.get(2)?,
                    replacement: row.get(3)?,
                    base_confidence: row.get(4)?,
                    reasoning: row.get(5)?,
                    applied_count: row.get::<_, i64>(6)? as usize,
                    accepted_count: row.get::<_, i64>(7)? as usize,
                    rejected_count: row.get::<_, i64>(8)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    /// Load patterns filtered by type
    pub fn load_patterns_by_type(&self, pattern_type: &str) -> Result<Vec<PatternRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, regex_pattern, replacement, base_confidence, reasoning,
                    applied_count, accepted_count, rejected_count
             FROM patterns
             WHERE enabled = 1 AND pattern_type = ?1
             ORDER BY base_confidence DESC"
        )?;

        let patterns = stmt
            .query_map([pattern_type], |row| {
                Ok(PatternRecord {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    regex_pattern: row.get(2)?,
                    replacement: row.get(3)?,
                    base_confidence: row.get(4)?,
                    reasoning: row.get(5)?,
                    applied_count: row.get::<_, i64>(6)? as usize,
                    accepted_count: row.get::<_, i64>(7)? as usize,
                    rejected_count: row.get::<_, i64>(8)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    /// Load patterns with minimum confidence threshold
    pub fn load_patterns_with_confidence(&self, min_confidence: f64) -> Result<Vec<PatternRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, regex_pattern, replacement, base_confidence, reasoning,
                    applied_count, accepted_count, rejected_count
             FROM patterns
             WHERE enabled = 1 AND base_confidence >= ?1
             ORDER BY base_confidence DESC"
        )?;

        let patterns = stmt
            .query_map([min_confidence], |row| {
                Ok(PatternRecord {
                    id: row.get(0)?,
                    pattern_type: row.get(1)?,
                    regex_pattern: row.get(2)?,
                    replacement: row.get(3)?,
                    base_confidence: row.get(4)?,
                    reasoning: row.get(5)?,
                    applied_count: row.get::<_, i64>(6)? as usize,
                    accepted_count: row.get::<_, i64>(7)? as usize,
                    rejected_count: row.get::<_, i64>(8)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    /// Record pattern application
    pub fn record_pattern_application(&self, pattern_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET applied_count = applied_count + 1 WHERE id = ?1",
            [pattern_id],
        )?;
        Ok(())
    }

    /// Record HITL decision
    pub fn record_hitl_decision(&self, decision: &HitlDecision) -> Result<()> {
        self.conn.execute(
            "INSERT INTO hitl_decisions
             (pattern_id, session_id, original_text, optimized_text, decision,
              user_alternative, context_before, context_after)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                decision.pattern_id,
                &decision.session_id,
                &decision.original_text,
                &decision.optimized_text,
                &decision.decision,
                &decision.user_alternative,
                &decision.context_before,
                &decision.context_after,
            ],
        )?;
        Ok(())
    }

    /// Get pattern statistics
    pub fn get_pattern_stats(&self) -> Result<Vec<PatternTypeStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT pattern_type, COUNT(*), AVG(base_confidence),
                    SUM(applied_count), SUM(accepted_count), SUM(rejected_count)
             FROM patterns
             WHERE enabled = 1
             GROUP BY pattern_type
             ORDER BY COUNT(*) DESC"
        )?;

        let stats = stmt
            .query_map([], |row| {
                let accepted = row.get::<_, i64>(4)? as usize;
                let rejected = row.get::<_, i64>(5)? as usize;
                let acceptance_rate = if accepted + rejected > 0 {
                    accepted as f64 / (accepted + rejected) as f64
                } else {
                    0.0
                };

                Ok(PatternTypeStats {
                    pattern_type: row.get(0)?,
                    total_patterns: row.get::<_, i64>(1)? as usize,
                    avg_confidence: row.get(2)?,
                    total_applications: row.get::<_, i64>(3)? as usize,
                    total_accepted: accepted,
                    total_rejected: rejected,
                    acceptance_rate,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }
}

/// Concept data structure
#[derive(Debug, Clone, PartialEq)]
pub struct Concept {
    pub qid: String,
    pub label_en: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Surface form data structure
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceForm {
    pub qid: String,
    pub tokenizer_id: String,
    pub lang: String,
    pub form: String,
    pub token_count: usize,
    pub char_count: usize,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_concepts: usize,
    pub total_surface_forms: usize,
    pub cache_size: usize,
}

/// Pattern record from database
#[derive(Debug, Clone)]
pub struct PatternRecord {
    pub id: i64,
    pub pattern_type: String,
    pub regex_pattern: String,
    pub replacement: String,
    pub base_confidence: f64,
    pub reasoning: String,
    pub applied_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
}

/// HITL decision record
#[derive(Debug, Clone)]
pub struct HitlDecision {
    pub pattern_id: i64,
    pub session_id: String,
    pub original_text: String,
    pub optimized_text: String,
    pub decision: String, // "accept", "reject", "modify"
    pub user_alternative: Option<String>,
    pub context_before: String,
    pub context_after: String,
}

/// Pattern type statistics
#[derive(Debug, Clone)]
pub struct PatternTypeStats {
    pub pattern_type: String,
    pub total_patterns: usize,
    pub avg_confidence: f64,
    pub total_applications: usize,
    pub total_accepted: usize,
    pub total_rejected: usize,
    pub acceptance_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_database() {
        let db = Database::in_memory().unwrap();
        let version = db.schema_version().unwrap();
        assert_eq!(version, "1");
    }

    #[test]
    fn test_get_stats_empty() {
        let db = Database::in_memory().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_concepts, 0);
        assert_eq!(stats.total_surface_forms, 0);
        assert_eq!(stats.cache_size, 0);
    }

    #[test]
    fn test_upsert_concept() {
        let db = Database::in_memory().unwrap();

        let concept = Concept {
            qid: "Q16917".to_string(),
            label_en: "hospital".to_string(),
            description: Some("healthcare facility".to_string()),
            category: Some("medical".to_string()),
        };

        db.upsert_concept(&concept).unwrap();

        let retrieved = db.get_concept("Q16917").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().label_en, "hospital");

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_concepts, 1);
    }

    #[test]
    fn test_find_concept_by_label() {
        let db = Database::in_memory().unwrap();

        let concept = Concept {
            qid: "Q16917".to_string(),
            label_en: "hospital".to_string(),
            description: Some("healthcare facility".to_string()),
            category: Some("medical".to_string()),
        };

        db.upsert_concept(&concept).unwrap();

        // Case-insensitive search
        let found = db.find_concept_by_label("Hospital").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().qid, "Q16917");

        // Not found
        let not_found = db.find_concept_by_label("clinic").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_surface_forms() {
        let db = Database::in_memory().unwrap();

        // Insert concept first
        let concept = Concept {
            qid: "Q16917".to_string(),
            label_en: "hospital".to_string(),
            description: None,
            category: None,
        };
        db.upsert_concept(&concept).unwrap();

        // Insert surface forms
        let forms = vec![
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "en".to_string(),
                form: "hospital".to_string(),
                token_count: 1,
                char_count: 8,
            },
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "zh".to_string(),
                form: "医院".to_string(),
                token_count: 4,
                char_count: 2,
            },
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "es".to_string(),
                form: "hospital".to_string(),
                token_count: 1,
                char_count: 8,
            },
        ];

        for form in &forms {
            db.insert_surface_form(form).unwrap();
        }

        // Get all forms
        let retrieved = db.get_surface_forms("Q16917", "cl100k_base").unwrap();
        assert_eq!(retrieved.len(), 3);

        // Get cheapest (should be one of the 1-token forms)
        let cheapest = db.get_cheapest_form("Q16917", "cl100k_base").unwrap();
        assert!(cheapest.is_some());
        assert_eq!(cheapest.unwrap().token_count, 1);

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_surface_forms, 3);
    }
}
