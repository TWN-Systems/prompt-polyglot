/// Integration Test for Consolidated Database-Backed Pattern System
///
/// Tests the full pipeline:
/// 1. Database setup
/// 2. Pattern migration
/// 3. Pattern loading
/// 4. Optimization with database patterns
/// 5. HITL feedback recording
/// 6. Confidence updating

use prompt_compress::{
    Database, DatabaseOptimizer, HitlDecision, OptimizationRequest,
    Language, DirectiveFormat, ConfidenceCalculator,
};
use std::sync::Arc;

#[test]
fn test_consolidated_system_end_to_end() {
    // Step 1: Create in-memory database
    let db = Database::in_memory().expect("Failed to create database");

    // Step 2: Apply patterns migration schema
    let migration_sql = include_str!("../migrations/002_add_patterns_table.sql");
    db.connection()
        .execute_batch(migration_sql)
        .expect("Failed to apply migration");

    // Step 3: Insert test patterns (simulating migration)
    insert_test_patterns(&db);

    // Step 4: Create database optimizer
    let tokenizer = prompt_compress::Tokenizer::new().expect("Failed to create tokenizer");
    let calculator = ConfidenceCalculator::default();

    let mut optimizer = DatabaseOptimizer::new(Arc::new(db), calculator, tokenizer)
        .expect("Failed to create optimizer");

    // Verify patterns loaded
    let pattern_count = optimizer.pattern_count();
    assert!(pattern_count > 0, "No patterns loaded");
    println!("✓ Loaded {} patterns", pattern_count);

    // Step 5: Test optimization
    let request = OptimizationRequest {
        prompt: "I would really appreciate if you could please analyze this code. I want you to provide a detailed explanation.".to_string(),
        output_language: Language::English,
        confidence_threshold: 0.85,
        aggressive_mode: false,
        directive_format: DirectiveFormat::Bracketed,
    };

    let result = optimizer.optimize(&request).expect("Optimization failed");

    // Verify optimization worked
    assert!(result.token_savings > 0, "Should have saved tokens");
    assert!(result.savings_percentage > 0.0, "Should have positive savings");
    assert!(!result.optimizations.is_empty(), "Should have applied optimizations");

    println!("✓ Optimization successful:");
    println!("  Original: {} tokens", result.original_tokens);
    println!("  Optimized: {} tokens", result.optimized_tokens);
    println!("  Savings: {:.1}%", result.savings_percentage);
    println!("  Applied {} optimizations", result.optimizations.len());

    // Step 6: Test HITL feedback
    let db = optimizer.database();

    let decision = HitlDecision {
        pattern_id: 1, // Assuming first pattern
        session_id: "test-session-001".to_string(),
        original_text: "I would really appreciate".to_string(),
        optimized_text: "".to_string(),
        decision: "accept".to_string(),
        user_alternative: None,
        context_before: "".to_string(),
        context_after: " if you could".to_string(),
    };

    db.record_hitl_decision(&decision)
        .expect("Failed to record decision");

    println!("✓ HITL decision recorded");

    // Step 7: Verify confidence updated
    let pattern = db.load_patterns()
        .expect("Failed to load patterns")
        .into_iter()
        .find(|p| p.id == 1)
        .expect("Pattern 1 not found");

    assert_eq!(pattern.accepted_count, 1, "Accepted count should be 1");
    println!("✓ Confidence updated: accepted_count = {}", pattern.accepted_count);

    // Step 8: Test pattern statistics
    let stats = db.get_pattern_stats().expect("Failed to get stats");
    assert!(!stats.is_empty(), "Should have pattern stats");

    for stat in &stats {
        println!("✓ Pattern type '{}': {} patterns, {:.1}% avg confidence",
                 stat.pattern_type,
                 stat.total_patterns,
                 stat.avg_confidence * 100.0);
    }

    // Step 9: Test pattern reloading
    optimizer.reload_patterns().expect("Failed to reload patterns");
    println!("✓ Pattern reload successful");
}

#[test]
fn test_confidence_filtering() {
    let db = Database::in_memory().expect("Failed to create database");

    // Apply migration
    let migration_sql = include_str!("../migrations/002_add_patterns_table.sql");
    db.connection()
        .execute_batch(migration_sql)
        .expect("Failed to apply migration");

    // Insert patterns with varying confidence
    db.connection()
        .execute(
            "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
             VALUES ('boilerplate', '(?i)test_high', '', 0.95, 'High confidence test')",
            [],
        )
        .expect("Failed to insert pattern");

    db.connection()
        .execute(
            "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
             VALUES ('boilerplate', '(?i)test_low', '', 0.60, 'Low confidence test')",
            [],
        )
        .expect("Failed to insert pattern");

    // Load with high threshold
    let tokenizer = prompt_compress::Tokenizer::new().unwrap();
    let calculator = ConfidenceCalculator::default();

    let optimizer = DatabaseOptimizer::with_confidence(
        Arc::new(db),
        calculator,
        tokenizer,
        0.90, // Only load patterns >= 0.90
    )
    .expect("Failed to create optimizer");

    // Should only load high-confidence pattern
    assert_eq!(optimizer.pattern_count(), 1, "Should only load high-confidence pattern");
    println!("✓ Confidence filtering works: loaded 1/2 patterns");
}

#[test]
fn test_pattern_application_tracking() {
    let db = Database::in_memory().expect("Failed to create database");

    // Apply migration
    let migration_sql = include_str!("../migrations/002_add_patterns_table.sql");
    db.connection()
        .execute_batch(migration_sql)
        .expect("Failed to apply migration");

    // Insert test pattern
    db.connection()
        .execute(
            "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
             VALUES ('boilerplate', '(?i)please', '', 0.90, 'Remove please')",
            [],
        )
        .expect("Failed to insert pattern");

    // Create optimizer
    let tokenizer = prompt_compress::Tokenizer::new().unwrap();
    let calculator = ConfidenceCalculator::default();

    let mut optimizer = DatabaseOptimizer::new(Arc::new(db), calculator, tokenizer)
        .expect("Failed to create optimizer");

    // Get initial application count
    let db_ref = optimizer.database();
    let initial_count: i64 = db_ref
        .connection()
        .query_row(
            "SELECT applied_count FROM patterns WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get count");

    println!("Initial applied_count: {}", initial_count);

    // Optimize text with "please"
    let request = OptimizationRequest {
        prompt: "Please help me with this task.".to_string(),
        output_language: Language::English,
        confidence_threshold: 0.85,
        aggressive_mode: false,
        directive_format: DirectiveFormat::Bracketed,
    };

    let _result = optimizer.optimize(&request).expect("Optimization failed");

    // Check application count increased
    let final_count: i64 = db_ref
        .connection()
        .query_row(
            "SELECT applied_count FROM patterns WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get count");

    println!("Final applied_count: {}", final_count);

    assert!(
        final_count > initial_count,
        "Application count should have increased"
    );
    println!("✓ Pattern application tracking works");
}

#[test]
fn test_hitl_confidence_bayesian_update() {
    let db = Database::in_memory().expect("Failed to create database");

    // Apply migration
    let migration_sql = include_str!("../migrations/002_add_patterns_table.sql");
    db.connection()
        .execute_batch(migration_sql)
        .expect("Failed to apply migration");

    // Insert pattern with base confidence 0.80
    db.connection()
        .execute(
            "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
             VALUES ('boilerplate', '(?i)test', '', 0.80, 'Test pattern')",
            [],
        )
        .expect("Failed to insert pattern");

    let initial_confidence: f64 = db
        .connection()
        .query_row(
            "SELECT base_confidence FROM patterns WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get confidence");

    println!("Initial confidence: {:.3}", initial_confidence);

    // Record 5 accepts and 1 reject
    for i in 0..6 {
        let decision = HitlDecision {
            pattern_id: 1,
            session_id: format!("session-{}", i),
            original_text: "test".to_string(),
            optimized_text: "".to_string(),
            decision: if i < 5 { "accept" } else { "reject" }.to_string(),
            user_alternative: None,
            context_before: "".to_string(),
            context_after: "".to_string(),
        };

        db.record_hitl_decision(&decision)
            .expect("Failed to record decision");
    }

    // Check updated confidence (should blend with feedback before 10 decisions)
    let updated_confidence: f64 = db
        .connection()
        .query_row(
            "SELECT base_confidence FROM patterns WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("Failed to get confidence");

    println!("Updated confidence after 5 accepts, 1 reject: {:.3}", updated_confidence);

    // Expected: (0.80 * 10 + 5) / (10 + 6) = 13.0 / 16 = 0.8125
    let expected = (0.80 * 10.0 + 5.0) / (10.0 + 6.0);
    let tolerance = 0.01;

    assert!(
        (updated_confidence - expected).abs() < tolerance,
        "Confidence should be updated via Bayesian formula. Expected: {:.3}, Got: {:.3}",
        expected,
        updated_confidence
    );

    println!("✓ Bayesian confidence update works correctly");
}

/// Helper function to insert test patterns
fn insert_test_patterns(db: &Database) {
    let patterns = vec![
        ("boilerplate", r"(?i)I would really appreciate", "", 0.95, "Politeness boilerplate"),
        ("boilerplate", r"(?i)please", "", 0.88, "Politeness marker"),
        ("filler", r"(?i)\breally\b", "", 0.88, "Intensity filler"),
        ("filler", r"(?i)\bvery\b", "", 0.85, "Intensity filler"),
        ("instruction", r"(?i)I want you to\s+", "", 0.92, "Verbose instruction"),
    ];

    for (pattern_type, regex, replacement, confidence, reasoning) in patterns {
        db.connection()
            .execute(
                "INSERT INTO patterns (pattern_type, regex_pattern, replacement, base_confidence, reasoning)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![pattern_type, regex, replacement, confidence, reasoning],
            )
            .expect("Failed to insert test pattern");
    }

    println!("✓ Inserted {} test patterns", 5);
}
