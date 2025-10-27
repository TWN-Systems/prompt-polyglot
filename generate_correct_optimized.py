#!/usr/bin/env python3
"""
Generate the CORRECT optimized output that v0.2+ should produce
"""

import re

def apply_optimizations(text):
    """Apply v0.2 optimizations with proper capitalization"""
    result = text

    # Step 1: Apply boilerplate removal
    patterns_applied = []

    # Complete sentence boilerplate (must match complete sentence to avoid orphans)
    complete_sentence_patterns = [
        (r"(?i)Thank you (so much )?in advance for [^.!?]+[.!?]\s*", "", "Complete gratitude sentence"),
        (r"(?i)I hope you('re| are) doing well\.\s*", "", "Greeting"),
    ]

    for pattern, repl, desc in complete_sentence_patterns:
        if re.search(pattern, result):
            result = re.sub(pattern, repl, result)
            patterns_applied.append(desc)

    # Partial boilerplate (safe to remove without creating orphans)
    partial_patterns = [
        (r"(?i)I would (really )?appreciate (it )?if you could\s+", "", "Polite prefix"),
        (r"(?i)Please make sure to\s+", "", "Redundant instruction"),
        (r"(?i)If you don't mind,?\s+", "", "Politeness"),
        (r"(?i)Could you please\s+", "", "Polite request"),
        (r"(?i)Would you mind\s+", "", "Polite request"),
    ]

    for pattern, repl, desc in partial_patterns:
        if re.search(pattern, result):
            result = re.sub(pattern, repl, result)
            patterns_applied.append(desc)

    # Step 2: Instruction compression
    instruction_patterns = [
        (r"(?i)I want you to\s+", ""),
        (r"(?i)I would like you to\s+", ""),
        (r"(?i)I would also like you to\s+", ""),
        (r"(?i)I need you to\s+", ""),
        (r"(?i)take the time to\s+", ""),
    ]

    for pattern, repl in instruction_patterns:
        result = re.sub(pattern, repl, result)

    # Step 3: Filler words
    fillers = ['really', 'very', 'quite', 'carefully', 'also']
    for filler in fillers:
        result = re.sub(r'(?i)\b' + filler + r'\b\s*', '', result)

    # Step 4: Redundant phrases
    result = re.sub(r"(?i)very\s+detailed\s+and\s+thorough", "detailed", result)
    result = re.sub(r"(?i)detailed\s+and\s+thorough", "detailed", result)
    result = re.sub(r"(?i)that\s+I'?m\s+working\s+on", "", result)
    result = re.sub(r"(?i)this\s+code\s+snippet", "this code", result)
    result = re.sub(r"(?i)any\s+potential\s+", "any ", result)

    # Step 5: Clean whitespace
    result = re.sub(r'  +', ' ', result)
    result = re.sub(r' ([.,!?])', r'\1', result)
    result = result.strip()

    # Step 6: Fix sentence capitalization
    # This is the KEY step that v0.2 includes
    lines = result.split('\n')
    fixed_lines = []

    for line in lines:
        line = line.strip()
        if not line:
            fixed_lines.append(line)
            continue

        # Capitalize first letter of line
        if line and line[0].isalpha() and line[0].islower():
            line = line[0].upper() + line[1:]

        # Capitalize after sentence-ending punctuation
        fixed = []
        i = 0
        while i < len(line):
            fixed.append(line[i])

            # If we hit sentence-ending punctuation followed by space
            if line[i] in '.!?' and i + 1 < len(line):
                # Skip whitespace
                j = i + 1
                while j < len(line) and line[j].isspace():
                    fixed.append(line[j])
                    j += 1

                # Capitalize next letter
                if j < len(line) and line[j].isalpha():
                    fixed.append(line[j].upper())
                    i = j
                else:
                    i = j - 1

            i += 1

        fixed_lines.append(''.join(fixed))

    result = '\n'.join(fixed_lines)

    # Step 7: Add language directive
    result += "\n\n[output_language: english]"

    return result, patterns_applied

# Read the verbose prompt
with open('examples/verbose_prompt.txt', 'r') as f:
    original = f.read().strip()

print("=" * 100)
print("GENERATING CORRECT OPTIMIZED OUTPUT (v0.2+)")
print("=" * 100)

optimized, patterns = apply_optimizations(original)

# Save to new file
with open('examples/optimized_v02.txt', 'w') as f:
    f.write(optimized)

print("\n✓ Saved to: examples/optimized_v02.txt")
print("\nOptimizations applied:")
for p in patterns:
    print(f"  - {p}")

print("\n" + "-" * 100)
print("CORRECTED OUTPUT:")
print("-" * 100)
print(optimized)
print("-" * 100)

# Check for issues
issues = []
if "for your help" in optimized.lower():
    issues.append("❌ Orphaned phrase found")
if optimized[0].islower():
    issues.append("❌ Starts with lowercase")

sentences = re.split(r'[.!?]\s+', optimized)
for sent in sentences[1:]:
    if sent and sent[0].isalpha() and sent[0].islower():
        issues.append(f"❌ Uncapitalized sentence: '{sent[:40]}'")
        break

print("\nQuality Check:")
if issues:
    for issue in issues:
        print(f"  {issue}")
else:
    print("  ✓ No issues found - proper capitalization, no orphans")

# Compare with old version
print("\n" + "=" * 100)
print("COMPARISON WITH OLD optimized.txt:")
print("=" * 100)

with open('examples/optimized.txt', 'r') as f:
    old = f.read().strip()

old_issues = []
if "for your help" in old.lower():
    old_issues.append("❌ Contains orphaned phrase")
if old[0].islower():
    old_issues.append("❌ Starts with lowercase")
if "carefully" in old or "very detailed" in old or "I want you to" in old:
    old_issues.append("❌ Didn't remove all fillers/instructions")

print("\nOld version issues:")
for issue in old_issues:
    print(f"  {issue}")

print("\nRecommendation: Replace optimized.txt with optimized_v02.txt")
