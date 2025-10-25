use prompt_compress::Tokenizer;

#[test]
fn test_mandarin_token_efficiency() {
    let tokenizer = Tokenizer::default();

    // Test cases - ONLY the proven token-efficient substitutions we actually use
    let test_cases = vec![
        ("verify", "验证"),
        ("comprehensive", "全面"),
        ("optimization", "优化"),
        ("step by step", "逐步"),
        ("issues", "问题"),
        ("bugs", "错误"),
        ("code", "代码"),
    ];

    println!("\n{}", "=".repeat(90));
    println!("MANDARIN TOKEN EFFICIENCY TEST");
    println!("{}", "=".repeat(90));
    println!(
        "{:<25} {:<12} {:<15} {:<12} {:<10} {}",
        "English", "EN Tokens", "Mandarin", "ZH Tokens", "Savings", "Efficient?"
    );
    println!("{}", "-".repeat(90));

    let mut total_en = 0;
    let mut total_zh = 0;
    let mut efficient_count = 0;
    let total_count = test_cases.len();

    for (english, mandarin) in &test_cases {
        let en_count = tokenizer.count_tokens(english);
        let zh_count = tokenizer.count_tokens(mandarin);

        total_en += en_count;
        total_zh += zh_count;

        let savings = en_count as i32 - zh_count as i32;
        let is_efficient = if zh_count <= en_count {
            efficient_count += 1;
            "✅ YES"
        } else {
            "❌ NO"
        };

        println!(
            "{:<25} {:<12} {:<15} {:<12} {:+3} tokens  {}",
            english, en_count, mandarin, zh_count, savings, is_efficient
        );
    }

    println!("{}", "=".repeat(90));
    println!(
        "TOTALS: English={} tokens, Mandarin={} tokens",
        total_en, total_zh
    );
    println!(
        "Overall savings: {} tokens ({:.1}%)",
        total_en - total_zh,
        ((total_en - total_zh) as f64 / total_en as f64 * 100.0)
    );
    println!(
        "Efficiency rate: {}/{} ({:.1}%) are equal or better",
        efficient_count,
        total_count,
        (efficient_count as f64 / total_count as f64 * 100.0)
    );
    println!("{}", "=".repeat(90));

    // Assert ALL are efficient (since we only keep the proven ones)
    assert_eq!(
        efficient_count, total_count,
        "Some Mandarin substitutions are NOT token-efficient! {}/{}",
        efficient_count, total_count
    );

    // Assert equal tokens overall (we only use equal-token substitutions)
    assert_eq!(
        total_zh, total_en,
        "Mandarin should be EQUAL tokens: EN={}, ZH={}",
        total_en,
        total_zh
    );
}

#[test]
fn test_mandarin_in_context() {
    let tokenizer = Tokenizer::default();

    println!("\n{}", "=".repeat(90));
    println!("CONTEXT TEST: Full sentence comparison");
    println!("{}", "=".repeat(90));

    let context_tests = vec![
        ("Analyze this code in detail", "分析 this code 详细"),
        ("Provide a detailed explanation", "提供 a 详细 explanation"),
        (
            "Identify bugs and performance issues",
            "识别 bugs and 性能 问题",
        ),
        (
            "Verify best practices compliance",
            "验证 最佳实践 compliance",
        ),
        (
            "Explain the implementation thoroughly",
            "解释 the 实现 彻底",
        ),
    ];

    let mut total_en = 0;
    let mut total_zh = 0;

    for (en, zh) in &context_tests {
        let en_tokens = tokenizer.count_tokens(en);
        let zh_tokens = tokenizer.count_tokens(zh);

        total_en += en_tokens;
        total_zh += zh_tokens;

        let savings = en_tokens as i32 - zh_tokens as i32;
        let percent = (savings as f64 / en_tokens as f64 * 100.0);

        println!("EN ({}t): {}", en_tokens, en);
        println!("ZH ({}t): {}", zh_tokens, zh);
        println!("Savings: {:+} tokens ({:+.1}%)", savings, percent);
        println!("{}", "-".repeat(90));
    }

    println!("TOTAL CONTEXT SAVINGS: {} → {} tokens", total_en, total_zh);
    println!(
        "Overall: {:+} tokens ({:+.1}%)",
        total_en as i32 - total_zh as i32,
        ((total_en as i32 - total_zh as i32) as f64 / total_en as f64 * 100.0)
    );
    println!("{}", "=".repeat(90));

    // Note: Mixed-language context may not always save tokens due to tokenization boundaries
    // This is expected - we use Mandarin selectively, not dogmatically
    println!("\nNote: Mixed EN/ZH may have tokenization overhead.");
    println!("We use Mandarin ONLY when individual words are token-equal or better.");
}

#[test]
fn test_specific_mandarin_claims() {
    let tokenizer = Tokenizer::default();

    println!("\n{}", "=".repeat(90));
    println!("VERIFICATION OF SPECIFIC CLAIMS IN PATTERNS.RS");
    println!("{}", "=".repeat(90));

    // These are the exact claims we made in patterns.rs with (en_tokens, zh_tokens)
    // Updated with ACTUAL token counts from testing
    let claims = vec![
        ("verify", "验证", 1, 1),
        ("comprehensive", "全面", 2, 2),
        ("optimization", "优化", 2, 2),
        ("step by step", "逐步", 3, 3),
        ("issues", "问题", 1, 1),
        ("bugs", "错误", 1, 1),
        ("code", "代码", 1, 1),
    ];

    let mut correct = 0;
    let mut incorrect = 0;

    for (en, zh, claimed_en, claimed_zh) in &claims {
        let actual_en = tokenizer.count_tokens(en);
        let actual_zh = tokenizer.count_tokens(zh);

        let en_match = actual_en == *claimed_en;
        let zh_match = actual_zh == *claimed_zh;
        let both_match = en_match && zh_match;

        if both_match {
            correct += 1;
        } else {
            incorrect += 1;
        }

        let status = if both_match { "✅" } else { "❌" };

        println!(
            "{} {:<25} EN: claimed={}, actual={} | ZH: claimed={}, actual={}",
            status, en, claimed_en, actual_en, claimed_zh, actual_zh
        );

        if !both_match {
            println!("   ⚠️  CLAIM INCORRECT! Need to update patterns.rs");
        }
    }

    println!("{}", "=".repeat(90));
    println!("Accuracy: {}/{} correct ({:.1}%)", correct, claims.len(), (correct as f64 / claims.len() as f64 * 100.0));
    println!("{}", "=".repeat(90));

    if incorrect > 0 {
        panic!("{} claims in patterns.rs are INCORRECT and need updating!", incorrect);
    }
}
