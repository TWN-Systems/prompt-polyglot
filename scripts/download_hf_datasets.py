#!/usr/bin/env python3
"""Download and prepare Hugging Face datasets for testing.

This script downloads real-world prompt datasets from Hugging Face and prepares
them for comprehensive testing of the prompt-compress optimizer.

Usage:
    python3 scripts/download_hf_datasets.py
    python3 scripts/download_hf_datasets.py --quick  # Download fewer samples
"""

import argparse
import json
from pathlib import Path
from typing import List, Dict, Optional, Callable
import sys

try:
    from datasets import load_dataset
except ImportError:
    print("❌ Error: 'datasets' package not installed")
    print("Run: pip install datasets")
    sys.exit(1)


# Dataset configurations
DATASETS = {
    "wildchat": {
        "repo": "allenai/WildChat-1M",
        "samples": 10000,
        "samples_quick": 1000,
        "description": "Real user conversations with ChatGPT",
        "extract": lambda ex: ex["conversation"][0]["content"] if ex.get("conversation") and len(ex["conversation"]) > 0 else None,
    },
    "ultrachat": {
        "repo": "HuggingFaceH4/ultrachat_200k",
        "samples": 5000,
        "samples_quick": 500,
        "split": "train_sft",
        "description": "High-quality filtered conversations",
        "extract": lambda ex: ex["messages"][0]["content"] if ex.get("messages") and len(ex["messages"]) > 0 else None,
    },
    "alpaca": {
        "repo": "tatsu-lab/alpaca",
        "samples": 5000,
        "samples_quick": 500,
        "description": "Instruction-response pairs",
        "extract": lambda ex: ex.get("instruction"),
    },
    "code_mixed": {
        "repo": "iamtarun/python_code_instructions_18k_alpaca",
        "samples": 3000,
        "samples_quick": 300,
        "description": "Code-focused instructions",
        "extract": lambda ex: ex.get("prompt"),
    },
}


def download_dataset(name: str, config: Dict, quick: bool = False) -> List[str]:
    """Download and extract prompts from a dataset.

    Args:
        name: Dataset name
        config: Dataset configuration
        quick: If True, download fewer samples for quick testing

    Returns:
        List of extracted prompts
    """
    print(f"\n{'='*60}")
    print(f"📥 Downloading: {name}")
    print(f"Description: {config['description']}")
    print(f"{'='*60}")

    split = config.get("split", "train")
    n_samples = config["samples_quick"] if quick else config["samples"]

    try:
        # Load dataset
        print(f"Loading {n_samples} samples from {config['repo']}...")
        ds = load_dataset(
            config["repo"],
            split=f"{split}[:{n_samples}]",
            trust_remote_code=True
        )
        print(f"✓ Downloaded {len(ds)} examples")

    except Exception as e:
        print(f"❌ Error downloading {name}: {e}")
        return []

    # Extract prompts
    print(f"📝 Extracting prompts...")
    prompts = []
    skipped = 0

    for i, ex in enumerate(ds):
        if i % 1000 == 0 and i > 0:
            print(f"  Processed {i}/{len(ds)} examples...")

        try:
            prompt = config["extract"](ex)
            if prompt and isinstance(prompt, str) and len(prompt) > 10:
                prompts.append(prompt)
            else:
                skipped += 1
        except Exception as e:
            skipped += 1
            if skipped < 10:  # Only show first few errors
                print(f"⚠️  Error extracting from example {i}: {e}")

    print(f"✅ Extracted {len(prompts)} valid prompts")
    if skipped > 0:
        print(f"⚠️  Skipped {skipped} invalid examples")

    return prompts


def save_prompts(prompts: List[str], output_dir: Path, name: str, config: Dict):
    """Save prompts to JSONL file with metadata.

    Args:
        prompts: List of prompts to save
        output_dir: Output directory
        name: Dataset name
        config: Dataset configuration
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save prompts as JSONL
    prompts_file = output_dir / "prompts.jsonl"
    with open(prompts_file, "w", encoding="utf-8") as f:
        for prompt in prompts:
            f.write(json.dumps({"text": prompt}) + "\n")

    # Calculate statistics
    total_chars = sum(len(p) for p in prompts)
    avg_length = total_chars / len(prompts) if prompts else 0

    # Save metadata
    metadata = {
        "name": name,
        "source": config["repo"],
        "description": config["description"],
        "count": len(prompts),
        "total_chars": total_chars,
        "avg_length": avg_length,
        "min_length": min(len(p) for p in prompts) if prompts else 0,
        "max_length": max(len(p) for p in prompts) if prompts else 0,
    }
    with open(output_dir / "metadata.json", "w", encoding="utf-8") as f:
        json.dump(metadata, f, indent=2)

    # Save README
    readme_content = f"""# {name} Test Suite

**Source:** {config['repo']}
**Description:** {config['description']}
**Samples:** {len(prompts)}

## Statistics

- Total prompts: {len(prompts):,}
- Average length: {avg_length:.1f} characters
- Min length: {metadata['min_length']} characters
- Max length: {metadata['max_length']} characters
- Total size: {total_chars:,} characters

## Usage

```bash
# Run tests with this suite
prompt-compress test --suite {name}

# Benchmark
prompt-compress benchmark --input prompts.jsonl --output results.json
```

## Files

- `prompts.jsonl`: Extracted prompts (one per line)
- `metadata.json`: Dataset statistics
- `README.md`: This file

Generated: {Path(__file__).name}
"""
    with open(output_dir / "README.md", "w", encoding="utf-8") as f:
        f.write(readme_content)

    print(f"💾 Saved to: {prompts_file}")
    print(f"   Metadata: {output_dir / 'metadata.json'}")
    print(f"   README: {output_dir / 'README.md'}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Download Hugging Face datasets for prompt-compress testing"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Download fewer samples for quick testing"
    )
    parser.add_argument(
        "--datasets",
        nargs="+",
        choices=list(DATASETS.keys()) + ["all"],
        default=["all"],
        help="Which datasets to download (default: all)"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("data/test_suites"),
        help="Output directory for test suites"
    )

    args = parser.parse_args()

    # Determine which datasets to download
    if "all" in args.datasets:
        datasets_to_download = list(DATASETS.keys())
    else:
        datasets_to_download = args.datasets

    print("\n" + "="*60)
    print("Hugging Face Dataset Downloader")
    print("="*60)
    print(f"Mode: {'Quick' if args.quick else 'Full'}")
    print(f"Datasets: {', '.join(datasets_to_download)}")
    print(f"Output: {args.output_dir.absolute()}")
    print("="*60)

    # Download each dataset
    total_prompts = 0
    successful = []
    failed = []

    for name in datasets_to_download:
        config = DATASETS[name]

        try:
            prompts = download_dataset(name, config, args.quick)

            if prompts:
                n_samples = config["samples_quick"] if args.quick else config["samples"]
                output_dir = args.output_dir / f"{name}_{n_samples//1000}k"
                save_prompts(prompts, output_dir, name, config)
                total_prompts += len(prompts)
                successful.append((name, len(prompts)))
            else:
                failed.append(name)

        except Exception as e:
            print(f"\n❌ Fatal error processing {name}: {e}")
            failed.append(name)
            continue

    # Print summary
    print("\n" + "="*60)
    print("Download Summary")
    print("="*60)

    if successful:
        print("\n✅ Successfully downloaded:")
        for name, count in successful:
            print(f"   {name:15s}: {count:6,} prompts")

    if failed:
        print("\n❌ Failed to download:")
        for name in failed:
            print(f"   {name}")

    print(f"\n📊 Total prompts: {total_prompts:,}")
    print(f"📁 Output directory: {args.output_dir.absolute()}")

    if args.quick:
        print("\n💡 Tip: Run without --quick for full datasets")

    print("\n✅ Done!")


if __name__ == "__main__":
    main()
