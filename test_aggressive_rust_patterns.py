#!/usr/bin/env python3
"""
Test the NEW aggressive Rust patterns we just added
Apply patterns in the same order as Rust code
"""

import re

def apply_rust_patterns(text):
    """Apply patterns in Rust detection order"""
    result = text

    # Phase 1: Boilerplate removal (from BOILERPLATE_PATTERNS)
    boilerplate = [
        (r"(?i)Thank you (so much )?in advance for .+?[.!]", ""),
        (r"(?i)I would (really )?appreciate (it )?if you could\s*", ""),
        (r"(?i)Please make sure to\s*", ""),
        (r"(?i)If you don't mind,?\s*", ""),
        (r"(?i)I('m| am) looking for help with\s*", ""),
        (r"(?i)Could you please\s*", ""),
        (r"(?i)Would you mind\s*", ""),
        (r"(?i)I would like you to\s*", ""),
        (r"(?i)It would be great if\s*", ""),
        (r"(?i)I need you to\s*", ""),
        (r"(?i)\bplease\b\s+", ""),  # Standalone please
        (r"(?i)\bkindly\b\s+", ""),
    ]

    for pattern, repl in boilerplate:
        result = re.sub(pattern, repl, result)

    # Phase 2: Instruction compression (from INSTRUCTION_PATTERNS)
    instructions = [
        (r"(?i)I want you to\s+", ""),
        (r"(?i)I would like you to\s+", ""),
        (r"(?i)I would also like you to\s+", ""),
        (r"(?i)I need you to\s+", ""),
        (r"(?i)take the time to\s+", ""),
    ]

    for pattern, repl in instructions:
        result = re.sub(pattern, repl, result)

    # Phase 3: AGGRESSIVE v0.3 patterns (from REDUNDANT_PHRASES - our new additions)
    # Apply in order from most specific to least specific

    # Complex multi-word patterns first
    aggressive = [
        # Three-part phrase compression (most specific)
        (r"(?i)what\s+(?:the\s+)?code\s+does,?\s+how\s+it\s+works,?\s+and\s+why\s+it\s+was\s+implemented(?:\s+in\s+this\s+particular\s+way)?\.?",
         "functionality, implementation, rationale."),

        # Long conditional sentence
        (r"(?i)If\s+you\s+find\s+(?:any\s+)?problems?\s+or\s+issues?,?\s+(?:please\s+)?provide\s+detailed\s+suggestions\s+on\s+how\s+to\s+fix\s+them\.?",
         "Suggest fixes."),

        # Performance check phrase
        (r"(?i)and\s+check\s+for\s+(?:any\s+)?performance\s+problems?\s+or\s+areas\s+where\s+(?:the\s+)?code\s+could\s+be\s+improved\s+or\s+optimized\.?",
         "performance/improvements."),

        # Look into bugs
        (r"(?i)Look\s+into\s+any\s+bugs?\s+or\s+issues\s+(?:that\s+you\s+might\s+find)?",
         "Identify bugs"),

        # Research and explain
        (r"(?i)Research\s+and\s+explain\s+whether\s+(?:this\s+)?code\s+follows\s+", "Verify "),

        # Provide detailed explanation
        (r"(?i)Provide\s+a\s+detailed\s+explanation\s+of\s+", "Explain: "),
        (r"(?i)provide\s+a\s+detailed\s+explanation\s+", "explain "),
        (r"(?i)provide\s+detailed\s+", ""),

        # Best practices
        (r"(?i)best\s+practices\s+and\s+coding\s+standards", "best practices"),

        # Context removals
        (r"(?i)in\s+this\s+particular\s+way", ""),
        (r"(?i)that\s+I'?m\s+working\s+on", ""),
        (r"(?i)this\s+code\s+snippet", "this code"),
        (r"(?i)you\s+might\s+find", ""),
        (r"(?i)any\s+potential\s+", ""),
        (r"(?i)or\s+areas\s+where", ""),
    ]

    for pattern, repl in aggressive:
        result = re.sub(pattern, repl, result)

    # Phase 4: Filler words
    fillers = ['really', 'very', 'quite', 'carefully', 'also', 'any']
    for filler in fillers:
        result = re.sub(r'(?i)\b' + filler + r'\b\s*', '', result)

    # Phase 5: Redundant phrases (original v0.2 patterns)
    result = re.sub(r"(?i)very\s+detailed\s+and\s+thorough", "detailed", result)
    result = re.sub(r"(?i)detailed\s+and\s+thorough", "detailed", result)
    result = re.sub(r"(?i)bugs?\s+(or|and)\s+issues", "bugs", result)
    result = re.sub(r"(?i)problems?\s+(or|and)\s+issues", "issues", result)

    # Phase 6: Clean whitespace
    result = re.sub(r'  +', ' ', result)
    result = re.sub(r' ([.,;:])', r'\1', result)
    result = re.sub(r'\s+([.,;:])', r'\1', result)
    result = result.strip()

    # Phase 7: Capitalize sentences
    def capitalize_sentences(text):
        if not text:
            return text

        if text[0].isalpha():
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

                if j < len(text) and text[j].isalpha():
                    result.append(text[j].upper())
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
print("RUST AGGRESSIVE PATTERNS TEST (v0.3)")
print("=" * 100)

optimized = apply_rust_patterns(original)

# Save
with open('examples/optimized.txt', 'w') as f:
    f.write(optimized)

print("\n✓ Saved to: examples/optimized.txt")

# Stats
def count_words(text):
    return len(text.split())

orig_words = count_words(original)
opt_words = count_words(optimized.split('\n\n[output_language')[0])

print(f"\nOriginal: {orig_words} words")
print(f"Optimized: {opt_words} words")
print(f"Savings: {orig_words - opt_words} words ({(orig_words - opt_words) / orig_words * 100:.1f}%)")

print("\n" + "-" * 100)
print("OUTPUT:")
print("-" * 100)
print(optimized)
print("-" * 100)

# Quality checks
checks = []
if 'please' in optimized.lower():
    checks.append("❌ Contains 'please'")
if optimized[0].islower():
    checks.append("❌ Starts with lowercase")
if 'for your help' in optimized.lower():
    checks.append("❌ Orphaned phrase")

if not checks:
    print("\n✅ All quality checks passed")
else:
    print("\n" + "\n".join(checks))
