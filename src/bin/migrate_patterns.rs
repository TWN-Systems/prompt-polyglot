/// Pattern Migration Tool
/// Migrates hardcoded patterns from patterns.rs into the SQLite database
///
/// Usage: cargo run --bin migrate_patterns -- atlas.db

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let db_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("atlas.db")
    };

    println!("Migrating patterns to database: {:?}", db_path);

    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database at {:?}", db_path))?;

    // Apply schema migration
    apply_schema_migration(&conn)?;

    // Clear existing patterns (for fresh migration)
    conn.execute("DELETE FROM patterns", [])?;
    println!("Cleared existing patterns");

    // Migrate all pattern types
    migrate_boilerplate_patterns(&conn)?;
    migrate_filler_patterns(&conn)?;
    migrate_instruction_patterns(&conn)?;
    migrate_redundant_phrases(&conn)?;
    migrate_structural_patterns(&conn)?;

    // Print summary
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM patterns", [], |row| row.get(0))?;
    println!("\n✅ Migration complete!");
    println!("   Total patterns migrated: {}", total);

    // Print breakdown by type
    println!("\nBreakdown by type:");
    let mut stmt = conn.prepare(
        "SELECT pattern_type, COUNT(*), AVG(base_confidence)
         FROM patterns
         GROUP BY pattern_type
         ORDER BY COUNT(*) DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;

    for row in rows {
        let (pattern_type, count, avg_conf) = row?;
        println!("   {:30} {:3} patterns (avg confidence: {:.2}%)",
                 pattern_type, count, avg_conf * 100.0);
    }

    Ok(())
}

fn apply_schema_migration(conn: &Connection) -> Result<()> {
    // Check if patterns table exists
    let table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='patterns'",
        [],
        |row| row.get(0),
    )?;

    if table_exists == 0 {
        println!("Applying schema migration: 002_add_patterns_table.sql");
        let schema_sql = include_str!("../../migrations/002_add_patterns_table.sql");
        conn.execute_batch(schema_sql)
            .context("Failed to execute patterns schema migration")?;
        println!("✓ Schema migration applied");
    } else {
        println!("✓ Patterns table already exists");
    }

    Ok(())
}

fn migrate_boilerplate_patterns(conn: &Connection) -> Result<()> {
    println!("\nMigrating boilerplate patterns...");

    let patterns = prompt_compress::patterns::BOILERPLATE_PATTERNS;

    let mut stmt = conn.prepare(
        "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0;
    for (pattern, replacement, confidence, reasoning) in patterns {
        stmt.execute(rusqlite::params![
            "boilerplate",
            pattern,
            replacement,
            confidence,
            reasoning,
        ])?;
        count += 1;
    }

    println!("   ✓ Migrated {} boilerplate patterns", count);
    Ok(())
}

fn migrate_filler_patterns(conn: &Connection) -> Result<()> {
    println!("Migrating filler word patterns...");

    let patterns = prompt_compress::patterns::FILLER_WORDS;

    let mut stmt = conn.prepare(
        "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0;
    for (pattern, confidence, reasoning) in patterns {
        stmt.execute(rusqlite::params![
            "filler",
            pattern,
            "", // Fillers are always removed (empty replacement)
            confidence,
            reasoning,
        ])?;
        count += 1;
    }

    println!("   ✓ Migrated {} filler patterns", count);
    Ok(())
}

fn migrate_instruction_patterns(conn: &Connection) -> Result<()> {
    println!("Migrating instruction compression patterns...");

    let patterns = prompt_compress::patterns::INSTRUCTION_PATTERNS;

    let mut stmt = conn.prepare(
        "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0;
    for (pattern, replacement, confidence, reasoning) in patterns {
        stmt.execute(rusqlite::params![
            "instruction",
            pattern,
            replacement,
            confidence,
            reasoning,
        ])?;
        count += 1;
    }

    println!("   ✓ Migrated {} instruction patterns", count);
    Ok(())
}

fn migrate_redundant_phrases(conn: &Connection) -> Result<()> {
    println!("Migrating redundant phrase patterns...");

    let patterns = prompt_compress::patterns::REDUNDANT_PHRASES;

    let mut stmt = conn.prepare(
        "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0;
    for (pattern, replacement, confidence, reasoning) in patterns {
        stmt.execute(rusqlite::params![
            "redundant",
            pattern,
            replacement,
            confidence,
            reasoning,
        ])?;
        count += 1;
    }

    println!("   ✓ Migrated {} redundant phrase patterns", count);
    Ok(())
}

fn migrate_structural_patterns(conn: &Connection) -> Result<()> {
    println!("Migrating structural optimization patterns...");

    let patterns = prompt_compress::patterns::STRUCTURAL_PATTERNS;

    let mut stmt = conn.prepare(
        "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0;
    for (pattern, replacement, confidence, reasoning) in patterns {
        stmt.execute(rusqlite::params![
            "structural",
            pattern,
            replacement,
            confidence,
            reasoning,
        ])?;
        count += 1;
    }

    println!("   ✓ Migrated {} structural patterns", count);
    Ok(())
}
