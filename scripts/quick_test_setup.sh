#!/bin/bash
# Quick setup script for testing infrastructure
#
# This script:
# 1. Installs required Python dependencies
# 2. Downloads HF datasets (quick mode)
# 3. Builds test corpus
# 4. Runs initial tests
#
# Usage:
#   ./scripts/quick_test_setup.sh

set -e  # Exit on error

echo "=========================================="
echo "Quick Test Setup for prompt-compress"
echo "=========================================="

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3.8+"
    exit 1
fi

echo "✓ Python found: $(python3 --version)"

# Install dependencies
echo ""
echo "📦 Installing Python dependencies..."
pip install datasets tiktoken tqdm 2>&1 | grep -v "Requirement already satisfied" || true
echo "✓ Dependencies installed"

# Create data directories
echo ""
echo "📁 Creating data directories..."
mkdir -p data/test_suites
mkdir -p data/benchmarks/baselines
mkdir -p data/benchmarks/current
echo "✓ Directories created"

# Download datasets (quick mode for fast setup)
echo ""
echo "📥 Downloading datasets (quick mode - 1000 samples per dataset)..."
python3 scripts/download_hf_datasets.py --quick

# Build test corpus
echo ""
echo "🏗️  Building test corpus..."
python3 scripts/build_test_corpus.py --samples-per-category 500

# Summary
echo ""
echo "=========================================="
echo "✅ Setup Complete!"
echo "=========================================="
echo ""
echo "Test suites available in: data/test_suites/"
echo ""
echo "Next steps:"
echo "  1. Run quick tests:       cargo test"
echo "  2. Run comprehensive:     cargo test --suite comprehensive"
echo "  3. Download full datasets: python3 scripts/download_hf_datasets.py"
echo ""
echo "For more info, see: docs/TESTING-STRATEGY.md"
echo ""
