/// Database-backed Pattern Detector
/// Loads regex patterns from SQLite instead of hardcoded constants

use crate::database::{Database, PatternRecord};
use crate::models::OptimizationType;
use crate::patterns::DetectedPattern;
use anyhow::Result;
use regex::Regex;
use std::sync::Arc;

/// Pattern detector that loads patterns from database
pub struct DatabasePatternDetector {
    db: Arc<Database>,
    patterns: Vec<CompiledPattern>,
}

struct CompiledPattern {
    id: i64,
    pattern_type: OptimizationType,
    regex: Regex,
    replacement: String,
    base_confidence: f64,
    reasoning: String,
}

impl DatabasePatternDetector {
    /// Create new detector and load patterns from database
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let pattern_records = db.load_patterns()?;
        let patterns = Self::compile_patterns(pattern_records)?;

        Ok(Self { db, patterns })
    }

    /// Create new detector with minimum confidence threshold
    pub fn with_confidence(db: Arc<Database>, min_confidence: f64) -> Result<Self> {
        let pattern_records = db.load_patterns_with_confidence(min_confidence)?;
        let patterns = Self::compile_patterns(pattern_records)?;

        Ok(Self { db, patterns })
    }

    /// Compile pattern records into regex patterns
    fn compile_patterns(records: Vec<PatternRecord>) -> Result<Vec<CompiledPattern>> {
        let mut compiled = Vec::new();

        for record in records {
            // Try to compile regex
            match Regex::new(&record.regex_pattern) {
                Ok(regex) => {
                    let pattern_type = Self::parse_pattern_type(&record.pattern_type);

                    compiled.push(CompiledPattern {
                        id: record.id,
                        pattern_type,
                        regex,
                        replacement: record.replacement,
                        base_confidence: record.base_confidence,
                        reasoning: record.reasoning,
                    });
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to compile pattern {}: {} - {}",
                        record.id, record.regex_pattern, e
                    );
                    // Continue with other patterns
                }
            }
        }

        Ok(compiled)
    }

    /// Parse pattern type string into enum
    fn parse_pattern_type(type_str: &str) -> OptimizationType {
        match type_str {
            "boilerplate" => OptimizationType::BoilerplateRemoval,
            "filler" => OptimizationType::FillerRemoval,
            "instruction" => OptimizationType::InstructionCompression,
            "redundant" => OptimizationType::FormatConsolidation,
            "structural" => OptimizationType::FormatConsolidation,
            "synonym" => OptimizationType::SynonymConsolidation,
            "mandarin" => OptimizationType::MandarinSubstitution,
            _ => OptimizationType::FormatConsolidation, // Default
        }
    }

    /// Detect all patterns in text (similar to PatternDetector::detect_all)
    pub fn detect_all(&self, text: &str) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();

        for pattern in &self.patterns {
            for mat in pattern.regex.find_iter(text) {
                let optimized = pattern.regex.replace(mat.as_str(), &pattern.replacement);

                detected.push(DetectedPattern {
                    pattern_type: pattern.pattern_type.clone(),
                    original_text: mat.as_str().to_string(),
                    optimized_text: optimized.to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    base_confidence: pattern.base_confidence,
                    reasoning: pattern.reasoning.clone(),
                });

                // Record pattern application in database
                if let Err(e) = self.db.record_pattern_application(pattern.id) {
                    eprintln!("Warning: Failed to record pattern application: {}", e);
                }
            }
        }

        // Sort by position to handle overlaps later
        detected.sort_by_key(|d| d.start_pos);
        detected
    }

    /// Reload patterns from database
    pub fn reload_patterns(&mut self) -> Result<()> {
        let pattern_records = self.db.load_patterns()?;
        self.patterns = Self::compile_patterns(pattern_records)?;
        Ok(())
    }

    /// Get number of loaded patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get database reference
    pub fn database(&self) -> &Arc<Database> {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_database_pattern_detector() {
        // Create in-memory database
        let db = Database::in_memory().unwrap();

        // Insert a test pattern
        db.connection()
            .execute(
                "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    "boilerplate",
                    r"(?i)I would really appreciate",
                    "",
                    0.95,
                    "Test pattern"
                ],
            )
            .unwrap();

        // Create detector
        let detector = DatabasePatternDetector::new(Arc::new(db)).unwrap();

        assert_eq!(detector.pattern_count(), 1);

        // Test detection
        let text = "I would really appreciate your help with this.";
        let detected = detector.detect_all(text);

        assert!(!detected.is_empty());
        assert_eq!(detected[0].base_confidence, 0.95);
    }

    #[test]
    fn test_confidence_filtering() {
        let db = Database::in_memory().unwrap();

        // Insert patterns with different confidences
        db.connection()
            .execute(
                "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
                 VALUES ('boilerplate', 'test1', '', 0.95, 'High confidence')",
                [],
            )
            .unwrap();

        db.connection()
            .execute(
                "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
                 VALUES ('boilerplate', 'test2', '', 0.50, 'Low confidence')",
                [],
            )
            .unwrap();

        // Load with high confidence threshold
        let detector = DatabasePatternDetector::with_confidence(Arc::new(db), 0.90).unwrap();

        // Should only load the high-confidence pattern
        assert_eq!(detector.pattern_count(), 1);
    }
}
