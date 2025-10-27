#!/usr/bin/env python3
"""
AGGRESSIVE optimization mode - maximum compression via regex only
"""

import re

def aggressive_optimize(text):
    """Apply aggressive compression patterns"""
    result = text

    # Phase 1: Remove ALL politeness markers
    result = re.sub(r'(?i)\bplease\b\s*', '', result)
    result = re.sub(r'(?i)\bkindly\b\s*', '', result)

    # Phase 2: Aggressive boilerplate removal
    boilerplate = [
        (r"(?i)I would (really )?appreciate (it )?if you could\s*", ""),
        (r"(?i)Thank you (so much )?in advance[^.!?]*[.!?]\s*", ""),
        (r"(?i)Could you\s+", ""),
        (r"(?i)Would you\s+", ""),
        (r"(?i)I want you to\s+", ""),
        (r"(?i)I would like you to\s+", ""),
        (r"(?i)I need you to\s+", ""),
        (r"(?i)take the time to\s+", ""),
    ]

    for pattern, repl in boilerplate:
        result = re.sub(pattern, repl, result)

    # Phase 3: ALL filler words
    fillers = ['really', 'very', 'quite', 'just', 'actually', 'basically',
               'essentially', 'definitely', 'absolutely', 'certainly', 'carefully',
               'also', 'furthermore', 'moreover', 'indeed', 'clearly', 'obviously',
               'any', 'some']

    for filler in fillers:
        result = re.sub(r'(?i)\b' + filler + r'\b\s*', '', result)

    # Phase 4: Verbose phrase compression (aggressive)
    compressions = [
        # Verbose explanations → concise
        (r"(?i)Provide\s+a\s+detailed\s+explanation\s+of\s+", "Explain: "),
        (r"(?i)Provide\s+a\s+detailed\s+explanation\s+", "Explain "),
        (r"(?i)Provide\s+detailed\s+", ""),

        # "what X does, how X works, and why" → "X: functionality, implementation, rationale"
        (r"(?i)of\s+what\s+(?:the\s+)?code\s+does,?\s+how\s+it\s+works,?\s+and\s+why\s+it\s+was\s+implemented\s+(?:in\s+)?(?:this\s+)?(?:particular\s+)?(?:way\.?)?",
         "functionality, implementation, rationale."),

        # Look into X → Identify X
        (r"(?i)Look\s+into\s+", "Identify: "),
        (r"(?i)look\s+(?:for|at)\s+", "identify "),

        # "bugs or issues that you might find" → "bugs"
        (r"(?i)bugs?\s+or\s+issues\s+(?:that\s+you\s+might\s+find)?", "bugs"),
        (r"(?i)problems?\s+or\s+issues", "issues"),

        # "check for any X problems or areas where" → "X issues"
        (r"(?i)and\s+check\s+for\s+performance\s+problems?\s+or\s+areas\s+where\s+(?:the\s+)?code\s+could\s+be\s+improved\s+or\s+optimized\.?",
         "performance issues, improvements."),

        # "Research and explain whether" → "Verify"
        (r"(?i)Research\s+and\s+explain\s+whether\s+(?:this\s+)?code\s+follows\s+", "Verify "),
        (r"(?i)research\s+and\s+explain\s+", "verify "),

        # "best practices and coding standards" → "best practices"
        (r"(?i)best\s+practices\s+and\s+coding\s+standards", "best practices"),

        # Final cleanup sentence
        (r"(?i)If\s+you\s+find\s+problems?\s+or\s+issues?,?\s+provide\s+detailed\s+suggestions\s+on\s+how\s+to\s+fix\s+them\.?",
         "Suggest fixes."),
        (r"(?i)provide\s+suggestions\s+(?:on\s+how\s+)?(?:to\s+fix\s+)?(?:them)?", "suggest fixes"),

        # Additional compressions
        (r"(?i)this\s+code\s+snippet", "this code"),
        (r"(?i)that\s+I'?m\s+working\s+on", ""),
        (r"(?i)in\s+this\s+particular\s+way", ""),
        (r"(?i)you\s+might\s+find", ""),
    ]

    for pattern, repl in compressions:
        result = re.sub(pattern, repl, result)

    # Phase 5: Colon-based compression for lists
    # "Identify: bugs, and check performance issues" → "Identify: bugs, performance issues"
    result = re.sub(r',?\s+and\s+check\s+', ', ', result)
    result = re.sub(r'\.\s+Identify:', '. Identify:', result)

    # Phase 6: Article removal (aggressive)
    result = re.sub(r'\ba\s+detailed\s+', '', result)
    result = re.sub(r'\bthe\s+code\b', 'code', result)

    # Phase 7: Clean whitespace
    result = re.sub(r'  +', ' ', result)
    result = re.sub(r' ([.,;:])', r'\1', result)
    result = re.sub(r'\s+([.,;:])', r'\1', result)
    result = result.strip()

    # Phase 8: Capitalize sentences
    def capitalize_sentences(text):
        if not text:
            return text

        # Capitalize first character
        if text[0].isalpha():
            text = text[0].upper() + text[1:]

        # Capitalize after sentence-ending punctuation
        result = []
        i = 0
        while i < len(text):
            result.append(text[i])

            if text[i] in '.!?' and i + 1 < len(text):
                # Skip whitespace
                j = i + 1
                while j < len(text) and text[j].isspace():
                    result.append(text[j])
                    j += 1

                # Capitalize next letter
                if j < len(text) and text[j].isalpha():
                    result.append(text[j].upper())
                    i = j
                else:
                    i = j - 1

            i += 1

        return ''.join(result)

    result = capitalize_sentences(result)

    # Add language directive
    result += "\n\n[output_language: english]"

    return result

# Test on example prompt
with open('examples/verbose_prompt.txt', 'r') as f:
    original = f.read().strip()

print("=" * 100)
print("AGGRESSIVE OPTIMIZATION MODE - MAXIMUM COMPRESSION")
print("=" * 100)

optimized = aggressive_optimize(original)

# Save
with open('examples/optimized_aggressive.txt', 'w') as f:
    f.write(optimized)

print("\n✓ Saved to: examples/optimized_aggressive.txt")

# Count words/tokens
def count_words(text):
    return len(text.split())

def estimate_tokens(text):
    words = count_words(text)
    punct = len(re.findall(r'[.,!?;:]', text))
    return int(words * 0.75 + punct * 0.3)

orig_words = count_words(original)
opt_words = count_words(optimized.split('\n\n[output_language')[0])

orig_tokens = estimate_tokens(original)
opt_tokens = estimate_tokens(optimized.split('\n\n[output_language')[0])

print(f"\nOriginal: {orig_words} words (~{orig_tokens} tokens)")
print(f"Optimized: {opt_words} words (~{opt_tokens} tokens)")
print(f"Savings: {orig_words - opt_words} words ({(orig_words - opt_words) / orig_words * 100:.1f}%)")
print(f"         ~{orig_tokens - opt_tokens} tokens ({(orig_tokens - opt_tokens) / orig_tokens * 100:.1f}%)")

print("\n" + "-" * 100)
print("OPTIMIZED OUTPUT:")
print("-" * 100)
print(optimized)
print("-" * 100)

# Quality check
issues = []
lines = optimized.split('\n')
for i, line in enumerate(lines[:5]):
    line = line.strip()
    if line and 'please' in line.lower():
        issues.append(f"❌ Line {i+1} still contains 'please': {line}")

if not issues:
    print("\n✅ Quality check passed - no 'please' found")
else:
    for issue in issues:
        print(issue)
