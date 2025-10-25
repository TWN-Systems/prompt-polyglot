/// Populate sample concept data into SQLite database using Rust
///
/// Bootstraps the atlas with ~17 common concepts across domains

use prompt_compress::{Concept, Database, SurfaceForm, TokenizerId, TokenizerRegistry};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("Populating Concept Atlas database...\n");

    // Create or open database
    let db = Database::open("data/atlas.db")?;
    let db = Arc::new(db);

    // Get tokenizer for counting tokens
    let registry = TokenizerRegistry::new()?;
    let tokenizer = registry
        .get(TokenizerId::Cl100kBase)
        .expect("cl100k_base tokenizer not available");

    // Sample concepts with Q-IDs and translations
    let concepts = vec![
        // Technical concepts
        ("Q40056", "code", "computer code", "technical", vec![
            ("es", "código"),
            ("fr", "code"),
            ("zh", "代码"),
            ("ja", "コード"),
        ]),
        ("Q1931388", "bug", "software bug", "technical", vec![
            ("es", "error"),
            ("fr", "bogue"),
            ("zh", "错误"),
            ("ja", "バグ"),
        ]),
        ("Q187931", "function", "programming function", "technical", vec![
            ("es", "función"),
            ("fr", "fonction"),
            ("zh", "函数"),
            ("ja", "関数"),
        ]),
        ("Q165194", "API", "application programming interface", "technical", vec![
            ("es", "API"),
            ("fr", "API"),
            ("zh", "API"),
            ("ja", "API"),
        ]),
        ("Q8513", "database", "structured data storage", "technical", vec![
            ("es", "base de datos"),
            ("fr", "base de données"),
            ("zh", "数据库"),
            ("ja", "データベース"),
        ]),
        ("Q44127", "server", "computer server", "technical", vec![
            ("es", "servidor"),
            ("fr", "serveur"),
            ("zh", "服务器"),
            ("ja", "サーバー"),
        ]),
        // Action verbs
        ("Q217602", "analyze", "examine in detail", "action", vec![
            ("es", "analizar"),
            ("fr", "analyser"),
            ("zh", "分析"),
            ("ja", "分析する"),
        ]),
        ("Q79030", "verify", "confirm truth or accuracy", "action", vec![
            ("es", "verificar"),
            ("fr", "vérifier"),
            ("zh", "验证"),
            ("ja", "検証する"),
        ]),
        ("Q188507", "optimize", "make as effective as possible", "action", vec![
            ("es", "optimizar"),
            ("fr", "optimiser"),
            ("zh", "优化"),
            ("ja", "最適化する"),
        ]),
        ("Q13143958", "explain", "make clear", "action", vec![
            ("es", "explicar"),
            ("fr", "expliquer"),
            ("zh", "解释"),
            ("ja", "説明する"),
        ]),
        ("Q1302249", "implement", "put into effect", "action", vec![
            ("es", "implementar"),
            ("fr", "implémenter"),
            ("zh", "实现"),
            ("ja", "実装する"),
        ]),
        // Medical concepts
        ("Q16917", "hospital", "healthcare facility", "medical", vec![
            ("es", "hospital"),
            ("fr", "hôpital"),
            ("zh", "医院"),
            ("ja", "病院"),
        ]),
        ("Q131512", "patient", "person receiving medical care", "medical", vec![
            ("es", "paciente"),
            ("fr", "patient"),
            ("zh", "患者"),
            ("ja", "患者"),
        ]),
        ("Q788750", "diagnosis", "identification of disease", "medical", vec![
            ("es", "diagnóstico"),
            ("fr", "diagnostic"),
            ("zh", "诊断"),
            ("ja", "診断"),
        ]),
        // Qualifiers/adjectives
        ("Q685363", "comprehensive", "complete and thorough", "qualifier", vec![
            ("es", "integral"),
            ("fr", "complet"),
            ("zh", "全面"),
            ("ja", "包括的"),
        ]),
        ("Q339356", "thorough", "complete with attention to detail", "qualifier", vec![
            ("es", "minucioso"),
            ("fr", "minutieux"),
            ("zh", "彻底"),
            ("ja", "徹底的"),
        ]),
        ("Q1860557", "detailed", "having many details", "qualifier", vec![
            ("es", "detallado"),
            ("fr", "détaillé"),
            ("zh", "详细"),
            ("ja", "詳細"),
        ]),
        // Common nouns
        ("Q395", "issue", "problem or matter", "general", vec![
            ("es", "problema"),
            ("fr", "problème"),
            ("zh", "问题"),
            ("ja", "問題"),
        ]),
    ];

    let mut concepts_added = 0;
    let mut surface_forms_added = 0;

    for (qid, en_label, description, category, translations) in concepts {
        // Insert concept
        db.upsert_concept(&Concept {
            qid: qid.to_string(),
            label_en: en_label.to_string(),
            description: Some(description.to_string()),
            category: Some(category.to_string()),
        })?;
        concepts_added += 1;

        // Insert English surface form
        let en_tokens = tokenizer.count_tokens(en_label);
        db.insert_surface_form(&SurfaceForm {
            qid: qid.to_string(),
            tokenizer_id: TokenizerId::Cl100kBase.as_str().to_string(),
            lang: "en".to_string(),
            form: en_label.to_string(),
            token_count: en_tokens,
            char_count: en_label.len(),
        })?;
        surface_forms_added += 1;

        // Insert translation surface forms
        let translation_count = translations.len();
        for (lang, form) in &translations {
            let tokens = tokenizer.count_tokens(form);
            db.insert_surface_form(&SurfaceForm {
                qid: qid.to_string(),
                tokenizer_id: TokenizerId::Cl100kBase.as_str().to_string(),
                lang: lang.to_string(),
                form: form.to_string(),
                token_count: tokens,
                char_count: form.len(),
            })?;
            surface_forms_added += 1;
        }

        println!("  ✓ Added: {} - {} ({} translations)", qid, en_label, translation_count);
    }

    println!("\n✅ Population complete!");
    println!("   Concepts: {}", concepts_added);
    println!("   Surface forms: {}", surface_forms_added);
    println!("   Languages: en, es, fr, zh, ja");

    // Show stats
    let stats = db.get_stats()?;
    println!("\n📊 Database Statistics:");
    println!("   Total concepts: {}", stats.total_concepts);
    println!("   Total surface forms: {}", stats.total_surface_forms);
    println!("   Cache size: {}", stats.cache_size);

    Ok(())
}
