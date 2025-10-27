#!/usr/bin/env python3
"""
Manual simulation of prompt optimization to verify behavior
"""

import re

# Read the verbose prompt
with open("examples/verbose_prompt.txt", "r") as f:
    original = f.read().strip()

print("=" * 100)
print("MANUAL OPTIMIZATION SIMULATION")
print("=" * 100)
print(f"\nORIGINAL PROMPT ({len(original)} chars):")
print(original)
print()

# Simulate the boilerplate patterns from patterns.rs
boilerplate_patterns = [
    (r"(?i)I would (really )?appreciate (it )?if you could\s*", "", "Boilerplate removal"),
    (r"(?i)Please make sure to\s*", "", "Redundant instruction"),
    (r"(?i)Thank you (so much )?in advance for .+?[.!]", "", "Complete gratitude sentence"),
    (r"(?i)Thank you (so much )?in advance\s*", "", "Partial gratitude"),
]

# Apply boilerplate removals
result = original
optimizations = []

for pattern, replacement, reasoning in boilerplate_patterns:
    matches = list(re.finditer(pattern, result))
    for match in matches:
        old_text = match.group()
        result = result[:match.start()] + replacement + result[match.end():]
        optimizations.append({
            'type': 'Boilerplate',
            'original': old_text,
            'replacement': replacement,
            'reasoning': reasoning,
            'position': match.start()
        })
        print(f"✓ Removed: '{old_text.strip()}' ({reasoning})")
        break  # Re-search after each replacement

# Filler words
filler_patterns = [
    (r"(?i)\breally\b", ""),
    (r"(?i)\bvery\b", ""),
    (r"(?i)\bcarefully\b", ""),
    (r"(?i)\balso\b", ""),
]

for pattern, replacement in filler_patterns:
    matches = list(re.finditer(pattern, result))
    if matches:
        for match in matches:
            old_text = match.group()
            result = result[:match.start()] + replacement + result[match.end():]
            optimizations.append({
                'type': 'Filler',
                'original': old_text,
                'replacement': replacement
            })
            print(f"✓ Removed filler: '{old_text}'")
            break

# Instruction compression
instruction_patterns = [
    (r"(?i)I want you to\s+", ""),
    (r"(?i)I would like you to\s+", ""),
    (r"(?i)I would also like you to\s+", ""),
    (r"(?i)take the time to\s+", ""),
]

for pattern, replacement in instruction_patterns:
    matches = list(re.finditer(pattern, result))
    if matches:
        for match in matches:
            old_text = match.group()
            result = result[:match.start()] + replacement + result[match.end():]
            optimizations.append({
                'type': 'Instruction',
                'original': old_text,
                'replacement': replacement
            })
            print(f"✓ Compressed instruction: '{old_text.strip()}'")
            break

# Clean whitespace
result = re.sub(r'  +', ' ', result)
result = re.sub(r' ([.,!?])', r'\1', result)
result = result.strip()

# Capitalize sentences
def capitalize_sentences(text):
    # Simple implementation
    sentences = re.split(r'([.!?]\s+)', text)
    capitalized = []
    for i, part in enumerate(sentences):
        if i % 2 == 0 and part:  # Sentence content, not separator
            part = part[0].upper() + part[1:] if part else part
        capitalized.append(part)
    return ''.join(capitalized)

result = capitalize_sentences(result)

# Add language directive
result += "\n\n[output_language: english]"

print()
print("=" * 100)
print(f"OPTIMIZED PROMPT ({len(result)} chars):")
print("=" * 100)
print(result)
print()
print("=" * 100)
print("ISSUES TO CHECK:")
print("=" * 100)

issues = []

# Check for orphaned phrases
orphaned_phrases = ["for your help with this", "for your", "for this"]
for phrase in orphaned_phrases:
    if phrase in result.lower():
        issues.append(f"❌ Orphaned phrase found: '{phrase}'")

# Check for lowercase sentence starts
lines = result.split('\n')
for i, line in enumerate(lines):
    line = line.strip()
    if line and line[0].islower() and line[0].isalpha():
        issues.append(f"❌ Line {i+1} starts with lowercase: '{line[:30]}...'")

# Check for proper capitalization after periods
sentences = re.split(r'[.!?]\s+', result)
for i, sent in enumerate(sentences[1:], 1):  # Skip first
    if sent and sent[0].islower() and sent[0].isalpha():
        issues.append(f"❌ Sentence {i+1} not capitalized: '{sent[:30]}...'")

if issues:
    for issue in issues:
        print(issue)
else:
    print("✓ No obvious issues found")

print("=" * 100)

# Compare with existing optimized.txt
try:
    with open("examples/optimized.txt", "r") as f:
        existing_optimized = f.read().strip()

    print("\nEXISTING optimized.txt file:")
    print("-" * 100)
    print(existing_optimized)
    print("-" * 100)

    if "for your help" in existing_optimized.lower():
        print("\n⚠️  WARNING: Existing optimized.txt contains orphaned phrase!")
    if existing_optimized.split('\n')[0][0].islower():
        print("⚠️  WARNING: Existing optimized.txt starts with lowercase!")

except FileNotFoundError:
    print("\nNo existing optimized.txt found for comparison")
