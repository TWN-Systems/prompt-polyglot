#!/usr/bin/env python3
"""Build comprehensive test corpus from multiple sources.

This script combines prompts from different test suites and creates a balanced
corpus for comprehensive testing.

Usage:
    python3 scripts/build_test_corpus.py
    python3 scripts/build_test_corpus.py --samples-per-category 500
"""

import argparse
import json
import random
from pathlib import Path
from typing import List, Dict
from collections import Counter


def load_prompts(suite_dir: Path) -> List[Dict]:
    """Load prompts from a test suite directory.

    Args:
        suite_dir: Path to test suite directory

    Returns:
        List of prompt dictionaries
    """
    prompts_file = suite_dir / "prompts.jsonl"
    if not prompts_file.exists():
        return []

    prompts = []
    with open(prompts_file, encoding="utf-8") as f:
        for line in f:
            try:
                data = json.loads(line)
                data["source"] = suite_dir.name
                prompts.append(data)
            except json.JSONDecodeError as e:
                print(f"⚠️  Error parsing line in {prompts_file}: {e}")
                continue

    return prompts


def categorize_prompt(text: str) -> List[str]:
    """Categorize a prompt by its characteristics.

    Args:
        text: Prompt text

    Returns:
        List of category labels
    """
    categories = []
    length = len(text)

    # Length-based categories
    if length > 200:
        categories.append("verbose")
    elif length < 100:
        categories.append("concise")
    else:
        categories.append("medium")

    # Content-based categories
    if any(marker in text for marker in ["```", "def ", "function ", "class ", "import "]):
        categories.append("code_mixed")

    if any(ord(c) > 127 for c in text):
        categories.append("multilingual")

    # Boilerplate detection
    boilerplate_phrases = [
        "I would appreciate",
        "Please make sure",
        "Thank you in advance",
        "Could you please",
        "I would like you to",
        "Can you help me",
    ]
    if any(phrase.lower() in text.lower() for phrase in boilerplate_phrases):
        categories.append("boilerplate")

    # Question vs instruction
    if text.strip().endswith("?"):
        categories.append("question")
    elif any(text.lower().startswith(verb) for verb in ["analyze", "explain", "describe", "write", "create"]):
        categories.append("instruction")

    return categories if categories else ["uncategorized"]


def categorize_prompts(prompts: List[Dict]) -> Dict[str, List[Dict]]:
    """Categorize all prompts.

    Args:
        prompts: List of prompt dictionaries

    Returns:
        Dictionary mapping category names to lists of prompts
    """
    categories = {}

    for prompt in prompts:
        text = prompt["text"]
        prompt_categories = categorize_prompt(text)

        for category in prompt_categories:
            if category not in categories:
                categories[category] = []
            categories[category].append(prompt)

    return categories


def sample_balanced_corpus(
    categories: Dict[str, List[Dict]],
    n_per_category: int = 1000
) -> List[Dict]:
    """Sample a balanced corpus from categories.

    Args:
        categories: Dictionary of categorized prompts
        n_per_category: Number of samples per category

    Returns:
        Balanced list of prompts
    """
    corpus = []
    stats = {}

    for category, prompts in sorted(categories.items()):
        # Sample from this category
        if len(prompts) > n_per_category:
            sampled = random.sample(prompts, n_per_category)
        else:
            sampled = prompts

        # Add category label to each prompt
        for prompt in sampled:
            if "categories" not in prompt:
                prompt["categories"] = []
            if category not in prompt["categories"]:
                prompt["categories"].append(category)

        corpus.extend(sampled)
        stats[category] = len(sampled)

        print(f"✓ {category:15s}: {len(sampled):5,} prompts")

    return corpus, stats


def deduplicate_corpus(corpus: List[Dict]) -> List[Dict]:
    """Remove duplicate prompts.

    Args:
        corpus: List of prompts

    Returns:
        Deduplicated corpus
    """
    seen = set()
    deduped = []

    for prompt in corpus:
        text = prompt["text"]
        if text not in seen:
            seen.add(text)
            deduped.append(prompt)

    duplicates_removed = len(corpus) - len(deduped)
    if duplicates_removed > 0:
        print(f"\n🔄 Removed {duplicates_removed:,} duplicate prompts")

    return deduped


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Build comprehensive test corpus from multiple sources"
    )
    parser.add_argument(
        "--input-dir",
        type=Path,
        default=Path("data/test_suites"),
        help="Input directory containing test suites"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("data/test_suites/comprehensive_corpus.jsonl"),
        help="Output file for comprehensive corpus"
    )
    parser.add_argument(
        "--samples-per-category",
        type=int,
        default=1000,
        help="Number of samples per category"
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility"
    )

    args = parser.parse_args()

    # Set random seed
    random.seed(args.seed)

    print("\n" + "="*60)
    print("Building Comprehensive Test Corpus")
    print("="*60)
    print(f"Input: {args.input_dir.absolute()}")
    print(f"Output: {args.output.absolute()}")
    print(f"Samples per category: {args.samples_per_category:,}")
    print(f"Random seed: {args.seed}")
    print("="*60)

    # Load all prompts from test suites
    print("\n📂 Loading prompts from test suites...")
    all_prompts = []
    suite_counts = {}

    for suite_dir in sorted(args.input_dir.iterdir()):
        if suite_dir.is_dir() and suite_dir.name != "q1678":  # Skip Q1678
            prompts = load_prompts(suite_dir)
            if prompts:
                all_prompts.extend(prompts)
                suite_counts[suite_dir.name] = len(prompts)
                print(f"  ✓ {suite_dir.name:20s}: {len(prompts):6,} prompts")

    if not all_prompts:
        print("\n❌ No prompts found!")
        print("Run scripts/download_hf_datasets.py first")
        return

    print(f"\n📊 Total prompts loaded: {len(all_prompts):,}")

    # Categorize prompts
    print("\n🏷️  Categorizing prompts...")
    categories = categorize_prompts(all_prompts)

    print(f"\nFound {len(categories)} categories:")
    for category, prompts in sorted(categories.items(), key=lambda x: len(x[1]), reverse=True):
        print(f"  {category:15s}: {len(prompts):6,} prompts")

    # Sample balanced corpus
    print(f"\n⚖️  Building balanced corpus ({args.samples_per_category:,} per category)...")
    corpus, category_stats = sample_balanced_corpus(categories, args.samples_per_category)

    print(f"\n📝 Corpus size before deduplication: {len(corpus):,}")

    # Deduplicate
    corpus = deduplicate_corpus(corpus)

    print(f"📝 Corpus size after deduplication: {len(corpus):,}")

    # Save corpus
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        for prompt in corpus:
            f.write(json.dumps(prompt) + "\n")

    # Save metadata
    metadata_file = args.output.with_suffix(".meta.json")
    metadata = {
        "total_prompts": len(corpus),
        "sources": suite_counts,
        "categories": category_stats,
        "samples_per_category": args.samples_per_category,
        "seed": args.seed,
    }
    with open(metadata_file, "w", encoding="utf-8") as f:
        json.dump(metadata, f, indent=2)

    # Print final summary
    print("\n" + "="*60)
    print("Corpus Statistics")
    print("="*60)

    print("\n📦 By Source:")
    for source, count in sorted(suite_counts.items(), key=lambda x: x[1], reverse=True):
        print(f"  {source:20s}: {count:6,} prompts")

    print("\n🏷️  By Category:")
    for category, count in sorted(category_stats.items(), key=lambda x: x[1], reverse=True):
        print(f"  {category:15s}: {count:6,} prompts")

    print(f"\n📊 Total corpus size: {len(corpus):,} prompts")
    print(f"💾 Saved to: {args.output}")
    print(f"📋 Metadata: {metadata_file}")

    # Calculate size estimates
    total_chars = sum(len(p["text"]) for p in corpus)
    avg_chars = total_chars / len(corpus) if corpus else 0
    estimated_tokens = total_chars / 4  # Rough estimate: 1 token ≈ 4 chars

    print(f"\n📏 Size Estimates:")
    print(f"  Total characters: {total_chars:,}")
    print(f"  Average per prompt: {avg_chars:.1f} chars")
    print(f"  Estimated tokens: {int(estimated_tokens):,}")

    print("\n✅ Done!")


if __name__ == "__main__":
    main()
