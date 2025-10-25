#!/usr/bin/env python3
"""
Populate sample concept data into SQLite database.

Bootstraps the atlas with ~100 common concepts across domains:
- Technical: code, bug, function, API, database, server
- Actions: analyze, verify, optimize, explain, implement
- Medical: hospital, patient, diagnosis
- General: comprehensive, thorough, detailed

For each concept:
- English label
- 3-5 language variants (es, fr, zh, ja)
- Precomputed token counts for cl100k_base tokenizer
"""

import sqlite3
import sys
from typing import List, Dict, Tuple
import tiktoken

# Sample concepts with Q-IDs and translations
# Format: (qid, en_label, description, category, translations)
SAMPLE_CONCEPTS = [
    # Technical concepts
    ("Q40056", "code", "computer code", "technical", {
        "es": "código",
        "fr": "code",
        "zh": "代码",
        "ja": "コード"
    }),
    ("Q1931388", "bug", "software bug", "technical", {
        "es": "error",
        "fr": "bogue",
        "zh": "错误",
        "ja": "バグ"
    }),
    ("Q187931", "function", "programming function", "technical", {
        "es": "función",
        "fr": "fonction",
        "zh": "函数",
        "ja": "関数"
    }),
    ("Q165194", "API", "application programming interface", "technical", {
        "es": "API",
        "fr": "API",
        "zh": "API",
        "ja": "API"
    }),
    ("Q8513", "database", "structured data storage", "technical", {
        "es": "base de datos",
        "fr": "base de données",
        "zh": "数据库",
        "ja": "データベース"
    }),
    ("Q44127", "server", "computer server", "technical", {
        "es": "servidor",
        "fr": "serveur",
        "zh": "服务器",
        "ja": "サーバー"
    }),

    # Action verbs
    ("Q217602", "analyze", "examine in detail", "action", {
        "es": "analizar",
        "fr": "analyser",
        "zh": "分析",
        "ja": "分析する"
    }),
    ("Q79030", "verify", "confirm truth or accuracy", "action", {
        "es": "verificar",
        "fr": "vérifier",
        "zh": "验证",
        "ja": "検証する"
    }),
    ("Q188507", "optimize", "make as effective as possible", "action", {
        "es": "optimizar",
        "fr": "optimiser",
        "zh": "优化",
        "ja": "最適化する"
    }),
    ("Q13143958", "explain", "make clear", "action", {
        "es": "explicar",
        "fr": "expliquer",
        "zh": "解释",
        "ja": "説明する"
    }),
    ("Q1302249", "implement", "put into effect", "action", {
        "es": "implementar",
        "fr": "implémenter",
        "zh": "实现",
        "ja": "実装する"
    }),

    # Medical concepts
    ("Q16917", "hospital", "healthcare facility", "medical", {
        "es": "hospital",
        "fr": "hôpital",
        "zh": "医院",
        "ja": "病院"
    }),
    ("Q131512", "patient", "person receiving medical care", "medical", {
        "es": "paciente",
        "fr": "patient",
        "zh": "患者",
        "ja": "患者"
    }),
    ("Q788750", "diagnosis", "identification of disease", "medical", {
        "es": "diagnóstico",
        "fr": "diagnostic",
        "zh": "诊断",
        "ja": "診断"
    }),

    # Qualifiers/adjectives
    ("Q685363", "comprehensive", "complete and thorough", "qualifier", {
        "es": "integral",
        "fr": "complet",
        "zh": "全面",
        "ja": "包括的"
    }),
    ("Q339356", "thorough", "complete with attention to detail", "qualifier", {
        "es": "minucioso",
        "fr": "minutieux",
        "zh": "彻底",
        "ja": "徹底的"
    }),
    ("Q1860557", "detailed", "having many details", "qualifier", {
        "es": "detallado",
        "fr": "détaillé",
        "zh": "详细",
        "ja": "詳細"
    }),

    # Common nouns
    ("Q395", "issue", "problem or matter", "general", {
        "es": "problema",
        "fr": "problème",
        "zh": "问题",
        "ja": "問題"
    }),
    ("Q11173", "performance", "execution quality", "general", {
        "es": "rendimiento",
        "fr": "performance",
        "zh": "性能",
        "ja": "パフォーマンス"
    }),
]


def get_tokenizer():
    """Get cl100k_base tokenizer (GPT-4, Claude)."""
    return tiktoken.get_encoding("cl100k_base")


def count_tokens(text: str, tokenizer) -> int:
    """Count tokens in text using tokenizer."""
    return len(tokenizer.encode(text))


def populate_database(db_path: str):
    """Populate SQLite database with sample concepts and surface forms."""

    print(f"Populating database: {db_path}")
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Initialize tokenizer
    tokenizer = get_tokenizer()
    tokenizer_id = "cl100k_base"

    # Track stats
    concepts_added = 0
    surface_forms_added = 0

    for qid, en_label, description, category, translations in SAMPLE_CONCEPTS:
        # Insert concept
        cursor.execute("""
            INSERT INTO concepts (qid, label_en, description, category)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(qid) DO UPDATE SET
                label_en = excluded.label_en,
                description = excluded.description,
                category = excluded.category
        """, (qid, en_label, description, category))
        concepts_added += 1

        # Insert English surface form
        en_tokens = count_tokens(en_label, tokenizer)
        cursor.execute("""
            INSERT INTO surface_forms (qid, tokenizer_id, lang, form, token_count, char_count)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(qid, tokenizer_id, lang, form) DO UPDATE SET
                token_count = excluded.token_count,
                char_count = excluded.char_count
        """, (qid, tokenizer_id, "en", en_label, en_tokens, len(en_label)))
        surface_forms_added += 1

        # Insert translation surface forms
        for lang, form in translations.items():
            tokens = count_tokens(form, tokenizer)
            cursor.execute("""
                INSERT INTO surface_forms (qid, tokenizer_id, lang, form, token_count, char_count)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(qid, tokenizer_id, lang, form) DO UPDATE SET
                    token_count = excluded.token_count,
                    char_count = excluded.char_count
            """, (qid, tokenizer_id, lang, form, tokens, len(form)))
            surface_forms_added += 1

        print(f"  Added: {qid} - {en_label} ({len(translations)} translations)")

    conn.commit()
    conn.close()

    print(f"\n✅ Population complete!")
    print(f"   Concepts: {concepts_added}")
    print(f"   Surface forms: {surface_forms_added}")
    print(f"   Languages: en, es, fr, zh, ja")


def show_stats(db_path: str):
    """Show database statistics."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Get stats
    cursor.execute("SELECT value FROM metadata WHERE key = 'total_concepts'")
    total_concepts = int(cursor.fetchone()[0])

    cursor.execute("SELECT value FROM metadata WHERE key = 'total_surface_forms'")
    total_surface_forms = int(cursor.fetchone()[0])

    # Get breakdown by category
    cursor.execute("""
        SELECT category, COUNT(*)
        FROM concepts
        WHERE category IS NOT NULL
        GROUP BY category
        ORDER BY COUNT(*) DESC
    """)
    categories = cursor.fetchall()

    # Get token efficiency examples
    cursor.execute("""
        SELECT c.label_en, sf.lang, sf.form, sf.token_count
        FROM concepts c
        JOIN surface_forms sf ON c.qid = sf.qid
        WHERE sf.tokenizer_id = 'cl100k_base'
        ORDER BY c.label_en, sf.token_count
        LIMIT 20
    """)
    examples = cursor.fetchall()

    conn.close()

    print("\n" + "="*80)
    print("DATABASE STATISTICS")
    print("="*80)
    print(f"Total concepts: {total_concepts}")
    print(f"Total surface forms: {total_surface_forms}")
    print(f"\nBy category:")
    for cat, count in categories:
        print(f"  {cat}: {count}")

    print(f"\nToken efficiency examples (first 20 forms):")
    current_label = None
    for label, lang, form, tokens in examples:
        if label != current_label:
            print(f"\n  {label}:")
            current_label = label
        print(f"    {lang}: '{form}' = {tokens} tokens")
    print("="*80)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python populate_sample_data.py <database_path> [--stats]")
        print("Example: python populate_sample_data.py data/atlas.db")
        sys.exit(1)

    db_path = sys.argv[1]

    if "--stats" in sys.argv:
        show_stats(db_path)
    else:
        populate_database(db_path)
        show_stats(db_path)
