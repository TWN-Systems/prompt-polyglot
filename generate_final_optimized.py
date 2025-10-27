#!/usr/bin/env python3
"""
Generate final optimized output using NEW v0.3 aggressive patterns
Matches Rust pattern order exactly
"""

import re

def optimize_aggressive_v03(text):
    """Apply v0.3 aggressive patterns in correct order"""
    result = text

    # Phase 1: Remove standalone politeness FIRST (before other patterns)
    result = re.sub(r'(?i)\bplease\b\s+', '', result)
    result = re.sub(r'(?i)\bkindly\b\s+', '', result)

    # Phase 2: Boilerplate removal (high-level phrases)
    result = re.sub(r"(?i)Thank you (so much )?in advance for .+?[.!]", "", result)
    result = re.sub(r"(?i)I would (really )?appreciate (it )?if you could\s*", "", result)
    result = re.sub(r"(?i)Could you\s+", "", result)
    result = re.sub(r"(?i)I want you to\s+", "", result)
    result = re.sub(r"(?i)I would (also )?like you to\s+", "", result)
    result = re.sub(r"(?i)\bmake sure to\s+", "", result)
    result = re.sub(r"(?i)take the time to\s+", "", result)

    # Phase 3: AGGRESSIVE v0.3 complete sentence compressions (MOST SPECIFIC FIRST)
    # These must run BEFORE filler word removal

    # Complete explanation pattern
    result = re.sub(
        r"(?i)Provide\s+a\s+(?:very\s+)?detailed\s+(?:and\s+thorough\s+)?explanation\s+of\s+what\s+(?:the\s+)?code\s+does,?\s+how\s+it\s+works,?\s+and\s+why\s+it\s+was\s+implemented(?:\s+in\s+this\s+particular\s+way)?\.?",
        "Explain: functionality, implementation, rationale.",
        result
    )

    # Combined bugs + performance check
    result = re.sub(
        r"(?i)look\s+into\s+(?:any\s+)?(?:potential\s+)?bugs?\s+or\s+issues\s+(?:that\s+you\s+might\s+find)?,?\s+and\s+(?:also\s+)?check\s+for\s+(?:any\s+)?performance\s+problems?\s+or\s+areas\s+where\s+(?:the\s+)?code\s+could\s+be\s+improved\s+or\s+optimized\.?",
        "Identify: bugs, performance issues, improvements.",
        result
    )

    # Research and explain best practices
    result = re.sub(
        r"(?i)Research\s+and\s+explain\s+whether\s+(?:this\s+)?code\s+follows\s+best\s+practices\s+and\s+coding\s+standards\.?",
        "Verify best practices.",
        result
    )

    # Final suggestion sentence
    result = re.sub(
        r"(?i)If\s+you\s+find\s+(?:any\s+)?problems?\s+or\s+issues?,?\s+(?:please\s+)?provide\s+detailed\s+suggestions\s+on\s+how\s+to\s+fix\s+them\.?",
        "Suggest fixes.",
        result
    )

    # Phase 4: Filler word removal (after big compressions)
    fillers = ['really', 'very', 'quite', 'carefully', 'also', 'any', 'potential']
    for filler in fillers:
        result = re.sub(r'(?i)\b' + filler + r'\b\s*', '', result)

    # Phase 5: Context removals
    result = re.sub(r"(?i)that\s+I'?m\s+working\s+on", "", result)
    result = re.sub(r"(?i)this\s+code\s+snippet", "this code", result)

    # Phase 6: Clean whitespace
    result = re.sub(r'  +', ' ', result)
    result = re.sub(r' ([.,;:])', r'\1', result)
    result = re.sub(r'\s+,', ',', result)
    result = result.strip()

    # Phase 7: Capitalize
    def capitalize_sentences(text):
        if not text or not text[0].isalpha():
            if text:
                text = text[0].upper() + text[1:] if len(text) > 1 else text
        elif text[0].islower():
            text = text[0].upper() + text[1:]

        result = []
        i = 0
        while i < len(text):
            result.append(text[i])

            if text[i] in '.!?' and i + 1 < len(text):
                j = i + 1
                while j < len(text) and text[j].isspace():
                    result.append(text[j])
                    j += 1

                if j < len(text) and text[j].isalpha() and text[j].islower():
                    result.append(text[j].upper())
                    i = j
                elif j < len(text):
                    result.append(text[j])
                    i = j
                else:
                    i = j - 1

            i += 1

        return ''.join(result)

    result = capitalize_sentences(result)
    result += "\n\n[output_language: english]"

    return result

# Test
with open('examples/verbose_prompt.txt', 'r') as f:
    original = f.read().strip()

print("=" * 100)
print("FINAL v0.3 AGGRESSIVE OPTIMIZATION")
print("=" * 100)

optimized = optimize_aggressive_v03(original)

# Save
with open('examples/optimized.txt', 'w') as f:
    f.write(optimized)

print("\n✓ Saved to: examples/optimized.txt")

# Stats
def count_words(text):
    return len(text.split())

def estimate_tokens(text):
    words = count_words(text)
    punct = len(re.findall(r'[.,!?;:]', text))
    return int(words * 0.75 + punct * 0.3)

orig_words = count_words(original)
opt_content = optimized.split('\n\n[output_language')[0]
opt_words = count_words(opt_content)

orig_tokens = estimate_tokens(original)
opt_tokens = estimate_tokens(opt_content)

print(f"\nOriginal: {orig_words} words (~{orig_tokens} tokens)")
print(f"Optimized: {opt_words} words (~{opt_tokens} tokens)")
print(f"Savings: {orig_words - opt_words} words ({(orig_words - opt_words) / orig_words * 100:.1f}%)")
print(f"         ~{orig_tokens - opt_tokens} tokens ({(orig_tokens - opt_tokens) / orig_tokens * 100:.1f}%)")

print("\n" + "-" * 100)
print("FINAL OUTPUT:")
print("-" * 100)
print(optimized)
print("-" * 100)

# Quality checks
checks = []
if 'please' in optimized.lower():
    checks.append("❌ Contains 'please'")
if opt_content[0].islower():
    checks.append("❌ Starts with lowercase")
if 'for your help' in optimized.lower():
    checks.append("❌ Orphaned phrase")
if 'that you might find' in optimized.lower():
    checks.append("❌ Contains 'that you might find'")
if 'in this particular way' in optimized.lower():
    checks.append("❌ Contains 'in this particular way'")

if not checks:
    print("\n✅ All quality checks passed - no 'please', proper capitalization, no orphans")
else:
    print("\nQuality Issues:")
    for check in checks:
        print(f"  {check}")
