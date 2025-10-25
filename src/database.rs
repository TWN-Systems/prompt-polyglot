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
