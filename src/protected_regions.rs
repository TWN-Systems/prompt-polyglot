/// Phase 3: Protected Regions - Detect areas that should NEVER be optimized
///
/// Purpose: Identify code blocks, template variables, technical terms, URLs,
/// and other regions where compression could break functionality.
///
/// Philosophy: "Efficiency without comprehension is a loop, not a shortcut"
/// → Never optimize instructions, code, or domain-specific terms

use regex::Regex;
use lazy_static::lazy_static;

/// A protected region in the text (byte range)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedRegion {
    pub start: usize,
    pub end: usize,
    pub region_type: RegionType,
    pub content: String,
}

/// Type of protected region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    /// Code block (```...``` or indented)
    CodeBlock,
    /// Template variable ({{...}}, ${...}, {%...%})
    TemplateVariable,
    /// URL or file path
    UrlOrPath,
    /// Technical identifier (CamelCase, snake_case, SCREAMING_CASE)
    Identifier,
    /// Quoted string
    QuotedString,
    /// Instruction keyword (MUST, REQUIRED, FORMAT, OUTPUT)
    InstructionKeyword,
}

/// Policy for protected region detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionPolicy {
    /// Protect more regions (safer, less compression)
    Conservative,
    /// Protect fewer regions (more compression, higher risk)
    Aggressive,
}

impl Default for ProtectionPolicy {
    fn default() -> Self {
        Self::Conservative
    }
}

lazy_static! {
    // Code blocks
    static ref FENCED_CODE_BLOCK: Regex = Regex::new(r"```[\s\S]*?```").unwrap();
    static ref INLINE_CODE: Regex = Regex::new(r"`[^`]+`").unwrap();

    // Template variables
    static ref MUSTACHE_VAR: Regex = Regex::new(r"\{\{[^}]+\}\}").unwrap();
    static ref DOLLAR_VAR: Regex = Regex::new(r"\$\{[^}]+\}").unwrap();
    static ref JINJA_VAR: Regex = Regex::new(r"\{%[^%]+%\}").unwrap();

    // URLs and paths
    static ref URL: Regex = Regex::new(r"https?://[^\s]+").unwrap();
    static ref FILE_PATH: Regex = Regex::new(r"(?:/[a-zA-Z0-9_.-]+)+|(?:[a-zA-Z]:\\[a-zA-Z0-9_.\\\-]+)").unwrap();

    // Identifiers
    static ref CAMEL_CASE: Regex = Regex::new(r"\b[a-z]+[A-Z][a-zA-Z0-9]*\b").unwrap();
    static ref SNAKE_CASE: Regex = Regex::new(r"\b[a-z]+_[a-z0-9_]+\b").unwrap();
    static ref SCREAMING_CASE: Regex = Regex::new(r"\b[A-Z][A-Z0-9_]{2,}\b").unwrap();

    // Quoted strings
    static ref DOUBLE_QUOTED: Regex = Regex::new(r#""[^"]*""#).unwrap();
    static ref SINGLE_QUOTED: Regex = Regex::new(r"'[^']*'").unwrap();

    // Instruction keywords
    static ref INSTRUCTION_KEYWORDS: Regex = Regex::new(
        r"(?i)\b(MUST|REQUIRED|MANDATORY|FORMAT|OUTPUT|RETURN|RESPOND|JSON|XML|YAML|CSV)\b"
    ).unwrap();
}

/// Protected region detector
pub struct ProtectedRegionDetector {
    policy: ProtectionPolicy,
}

impl ProtectedRegionDetector {
    /// Create new detector with policy
    pub fn new(policy: ProtectionPolicy) -> Self {
        Self { policy }
    }

    /// Detect all protected regions in text
    pub fn detect(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        // Always protect these (both policies)
        regions.extend(self.detect_code_blocks(text));
        regions.extend(self.detect_template_variables(text));
        regions.extend(self.detect_urls_and_paths(text));
        regions.extend(self.detect_instruction_keywords(text));

        // Conservative policy protects more
        if matches!(self.policy, ProtectionPolicy::Conservative) {
            regions.extend(self.detect_identifiers(text));
            regions.extend(self.detect_quoted_strings(text));
        }

        // Sort by start position and merge overlapping regions
        regions.sort_by_key(|r| r.start);
        self.merge_overlapping(regions)
    }

    /// Check if a byte range overlaps with any protected region
    pub fn is_protected(&self, regions: &[ProtectedRegion], start: usize, end: usize) -> bool {
        regions.iter().any(|r| {
            // Check for any overlap
            !(end <= r.start || start >= r.end)
        })
    }

    /// Detect fenced and inline code blocks
    fn detect_code_blocks(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        // Fenced code blocks (```...```)
        for mat in FENCED_CODE_BLOCK.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::CodeBlock,
                content: mat.as_str().to_string(),
            });
        }

        // Inline code (`...`)
        for mat in INLINE_CODE.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::CodeBlock,
                content: mat.as_str().to_string(),
            });
        }

        // Indented code blocks (4+ spaces at line start)
        for (line_num, line) in text.lines().enumerate() {
            if line.starts_with("    ") && !line.trim().is_empty() {
                // Find byte offset of this line
                let offset = text.lines().take(line_num).map(|l| l.len() + 1).sum();
                regions.push(ProtectedRegion {
                    start: offset,
                    end: offset + line.len(),
                    region_type: RegionType::CodeBlock,
                    content: line.to_string(),
                });
            }
        }

        regions
    }

    /// Detect template variables
    fn detect_template_variables(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        for mat in MUSTACHE_VAR.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::TemplateVariable,
                content: mat.as_str().to_string(),
            });
        }

        for mat in DOLLAR_VAR.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::TemplateVariable,
                content: mat.as_str().to_string(),
            });
        }

        for mat in JINJA_VAR.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::TemplateVariable,
                content: mat.as_str().to_string(),
            });
        }

        regions
    }

    /// Detect URLs and file paths
    fn detect_urls_and_paths(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        for mat in URL.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::UrlOrPath,
                content: mat.as_str().to_string(),
            });
        }

        for mat in FILE_PATH.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::UrlOrPath,
                content: mat.as_str().to_string(),
            });
        }

        regions
    }

    /// Detect programming identifiers
    fn detect_identifiers(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        for mat in CAMEL_CASE.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::Identifier,
                content: mat.as_str().to_string(),
            });
        }

        for mat in SNAKE_CASE.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::Identifier,
                content: mat.as_str().to_string(),
            });
        }

        for mat in SCREAMING_CASE.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::Identifier,
                content: mat.as_str().to_string(),
            });
        }

        regions
    }

    /// Detect quoted strings
    fn detect_quoted_strings(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        for mat in DOUBLE_QUOTED.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::QuotedString,
                content: mat.as_str().to_string(),
            });
        }

        for mat in SINGLE_QUOTED.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::QuotedString,
                content: mat.as_str().to_string(),
            });
        }

        regions
    }

    /// Detect instruction keywords
    fn detect_instruction_keywords(&self, text: &str) -> Vec<ProtectedRegion> {
        let mut regions = Vec::new();

        for mat in INSTRUCTION_KEYWORDS.find_iter(text) {
            regions.push(ProtectedRegion {
                start: mat.start(),
                end: mat.end(),
                region_type: RegionType::InstructionKeyword,
                content: mat.as_str().to_string(),
            });
        }

        regions
    }

    /// Merge overlapping regions
    fn merge_overlapping(&self, regions: Vec<ProtectedRegion>) -> Vec<ProtectedRegion> {
        if regions.is_empty() {
            return regions;
        }

        let mut merged = Vec::new();
        let mut current = regions[0].clone();

        for region in regions.into_iter().skip(1) {
            if region.start <= current.end {
                // Overlapping or adjacent - merge
                current.end = current.end.max(region.end);
                current.content = format!("{}...{}",
                    current.content.chars().take(20).collect::<String>(),
                    region.content.chars().rev().take(20).collect::<String>().chars().rev().collect::<String>()
                );
            } else {
                // No overlap - save current and start new
                merged.push(current);
                current = region;
            }
        }

        merged.push(current);
        merged
    }
}

impl Default for ProtectedRegionDetector {
    fn default() -> Self {
        Self::new(ProtectionPolicy::Conservative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_code_blocks() {
        let text = r#"
Here is some code:
```python
def hello():
    print("Hello")
```
And inline `code` too.
        "#;

        let detector = ProtectedRegionDetector::default();
        let regions = detector.detect(text);

        // Should find code blocks
        let code_blocks: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::CodeBlock)
            .collect();

        assert!(code_blocks.len() >= 1, "Should find at least the fenced code block");
    }

    #[test]
    fn test_detect_template_variables() {
        let text = "Hello {{name}}, your total is ${amount}. {% if admin %}Admin{% endif %}";

        let detector = ProtectedRegionDetector::default();
        let regions = detector.detect(text);

        let template_vars: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::TemplateVariable)
            .collect();

        // Should find {{name}}, ${amount}, and {% if admin %}
        assert!(template_vars.len() >= 3, "Expected at least 3 template vars, found {}", template_vars.len());
    }

    #[test]
    fn test_detect_urls() {
        let text = "Visit https://example.com or check /usr/local/bin/file.txt";

        let detector = ProtectedRegionDetector::default();
        let regions = detector.detect(text);

        let urls: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::UrlOrPath)
            .collect();

        assert!(urls.len() >= 2);
    }

    #[test]
    fn test_detect_identifiers() {
        let text = "Use camelCase, snake_case, and SCREAMING_CASE identifiers";

        let detector = ProtectedRegionDetector::new(ProtectionPolicy::Conservative);
        let regions = detector.detect(text);

        let identifiers: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::Identifier)
            .collect();

        assert_eq!(identifiers.len(), 3);
    }

    #[test]
    fn test_detect_quoted_strings() {
        let text = r#"Use "double quotes" and 'single quotes' for strings."#;

        let detector = ProtectedRegionDetector::new(ProtectionPolicy::Conservative);
        let regions = detector.detect(text);

        let quotes: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::QuotedString)
            .collect();

        assert_eq!(quotes.len(), 2);
    }

    #[test]
    fn test_detect_instruction_keywords() {
        let text = "MUST return JSON format. OUTPUT should be YAML.";

        let detector = ProtectedRegionDetector::default();
        let regions = detector.detect(text);

        let keywords: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == RegionType::InstructionKeyword)
            .collect();

        // MUST, return, JSON, format, OUTPUT, YAML are all matched
        assert!(keywords.len() >= 4, "Expected at least 4 instruction keywords, found {}", keywords.len());
    }

    #[test]
    fn test_is_protected() {
        let text = "Here is `code` and normal text";

        let detector = ProtectedRegionDetector::default();
        let regions = detector.detect(text);

        // Find the code region
        let code_region = regions.iter().find(|r| r.content.contains("code")).unwrap();

        // Test overlap detection
        assert!(detector.is_protected(&regions, code_region.start, code_region.end));
        assert!(detector.is_protected(&regions, code_region.start + 1, code_region.end - 1));

        // Test non-overlapping
        assert!(!detector.is_protected(&regions, 0, 5));
    }

    #[test]
    fn test_merge_overlapping() {
        let detector = ProtectedRegionDetector::default();

        let regions = vec![
            ProtectedRegion {
                start: 0,
                end: 10,
                region_type: RegionType::CodeBlock,
                content: "code1".to_string(),
            },
            ProtectedRegion {
                start: 5,
                end: 15,
                region_type: RegionType::CodeBlock,
                content: "code2".to_string(),
            },
            ProtectedRegion {
                start: 20,
                end: 30,
                region_type: RegionType::CodeBlock,
                content: "code3".to_string(),
            },
        ];

        let merged = detector.merge_overlapping(regions);

        assert_eq!(merged.len(), 2); // First two merged, third separate
        assert_eq!(merged[0].start, 0);
        assert_eq!(merged[0].end, 15);
        assert_eq!(merged[1].start, 20);
        assert_eq!(merged[1].end, 30);
    }

    #[test]
    fn test_aggressive_vs_conservative() {
        let text = "Use camelCase with \"quoted strings\"";

        let conservative = ProtectedRegionDetector::new(ProtectionPolicy::Conservative);
        let aggressive = ProtectedRegionDetector::new(ProtectionPolicy::Aggressive);

        let conservative_regions = conservative.detect(text);
        let aggressive_regions = aggressive.detect(text);

        // Conservative should protect more
        assert!(conservative_regions.len() > aggressive_regions.len());
    }

    #[test]
    fn test_real_world_prompt() {
        let text = r#"
Analyze this Python code and explain what it does:

```python
def calculate_total(items):
    return sum(item.price for item in items)
```

MUST return analysis in JSON format with keys: "functionality", "complexity", "issues".
The function uses camelCase naming. Check https://docs.python.org for best practices.
        "#;

        let detector = ProtectedRegionDetector::new(ProtectionPolicy::Conservative);
        let regions = detector.detect(text);

        // Should detect:
        // - Code block
        // - MUST, JSON keywords
        // - camelCase identifier
        // - URL

        assert!(regions.iter().any(|r| r.region_type == RegionType::CodeBlock));
        assert!(regions.iter().any(|r| r.region_type == RegionType::InstructionKeyword));
        assert!(regions.iter().any(|r| r.region_type == RegionType::Identifier));
        assert!(regions.iter().any(|r| r.region_type == RegionType::UrlOrPath));
    }
}
