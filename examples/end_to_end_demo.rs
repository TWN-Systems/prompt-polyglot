/// End-to-end demonstration of the prompt compression system
///
/// Shows the complete optimization pipeline with real-world examples:
/// 1. v0.2 pattern-based optimization (boilerplate, fillers, structural)
/// 2. v0.3 concept-based optimization (Q-ID resolution + surface form selection)
/// 3. Protected region detection (never optimize code/instructions)
/// 4. Token savings analysis

use prompt_compress::{
    ConceptOptimizer, Database, DirectiveFormat, Language, OptimizationRequest,
};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("\n{}", "=".repeat(100));
    println!("PROMPT COMPRESSION SYSTEM - END-TO-END DEMONSTRATION");
    println!("{}\n", "=".repeat(100));

    // Load the concept atlas database
    let db = Database::open("data/atlas.db")?;
    let db = Arc::new(db);

    // Create v0.3 concept optimizer
    let mut optimizer = ConceptOptimizer::new(db)?;

    // Test cases covering different optimization layers
    let test_cases = vec![
        (
            "Boilerplate-Heavy Prompt",
            "I would really appreciate it if you could please help me analyze this code. \
             I want you to verify the function and explain what it does. \
             Thank you so much in advance for your help with this!",
        ),
        (
            "Structural Optimizations",
            "The distance is 10 kilometers and it takes 5 minutes. \
             That's about 50 percent faster than expected!!!\n\n\n\
             ===\nNext section\n===",
        ),
        (
            "Protected Code Block",
            "Please analyze this function:\n\n\
             ```python\n\
             def hospital_distance(km):\n\
                 return km * 0.621371\n\
             ```\n\n\
             Verify the code works correctly.",
        ),
        (
            "JSON Structure",
            r#"{"description": "test server", "configuration": "prod", "parameters": {"timeout": 30}}"#,
        ),
        (
            "Mixed Optimizations",
            "I would really appreciate if you could analyze the hospital database server. \
             It's about 10 kilometers away and takes 5 minutes to access. \
             Please verify the API function works and explain the implementation thoroughly. \
             Make sure to check for bugs and issues!!!",
        ),
    ];

    for (name, prompt) in test_cases {
        println!("📝 Test Case: {}", name);
        println!("{}", "-".repeat(100));
        println!("Original ({} chars):", prompt.len());
        println!("{}\n", prompt);

        let request = OptimizationRequest {
            prompt: prompt.to_string(),
            output_language: Language::English,
            confidence_threshold: 0.85,
            aggressive_mode: false,
            directive_format: DirectiveFormat::Bracketed,
        };

        match optimizer.optimize(&request) {
            Ok(result) => {
                println!("Optimized ({} chars):", result.optimized_prompt.len());
                println!("{}\n", result.optimized_prompt);

                println!("📊 Savings:");
                println!("  Original tokens:   {}", result.original_tokens);
                println!("  Optimized tokens:  {}", result.optimized_tokens);
                println!("  Tokens saved:      {} ({:.1}%)",
                    result.token_savings,
                    result.savings_percentage
                );
                println!("  Characters saved:  {}",
                    prompt.len() as i64 - result.optimized_prompt.len() as i64
                );

                if !result.optimizations.is_empty() {
                    println!("\n  Applied optimizations:");
                    for opt in &result.optimizations {
                        println!("    • {:?}: \"{}\" → \"{}\" ({} tokens saved)",
                            opt.optimization_type,
                            opt.original_text.chars().take(40).collect::<String>(),
                            opt.optimized_text.chars().take(40).collect::<String>(),
                            opt.token_savings
                        );
                    }
                }

                if !result.requires_review.is_empty() {
                    println!("\n  ⚠️  Requires review ({} optimizations):", result.requires_review.len());
                    for opt in &result.requires_review {
                        println!("    • {:?}: \"{}\" (confidence: {:.0}%)",
                            opt.optimization_type,
                            opt.original_text.chars().take(40).collect::<String>(),
                            opt.confidence.final_confidence * 100.0
                        );
                    }
                }
            }
            Err(e) => {
                println!("❌ Optimization failed: {}", e);
            }
        }

        println!("\n{}\n", "=".repeat(100));
    }

    // Show system statistics
    let stats = optimizer.get_stats();
    println!("📊 System Statistics:");
    println!("  Concepts in atlas:     {}", stats.db_stats.total_concepts);
    println!("  Surface forms:         {}", stats.db_stats.total_surface_forms);
    println!("  Cache entries:         {}", stats.cache_stats.size);
    println!("  Cache capacity:        {}", stats.cache_stats.capacity);

    println!("\n✅ Demonstration complete!\n");
    Ok(())
}
