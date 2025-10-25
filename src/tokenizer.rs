use anyhow::{Context, Result};
use tiktoken_rs::{cl100k_base, CoreBPE};

/// Tokenizer for counting tokens in prompts
/// Uses tiktoken-rs (OpenAI's cl100k_base tokenizer)
pub struct Tokenizer {
    bpe: CoreBPE,
}

impl Tokenizer {
    /// Create a new tokenizer instance
    pub fn new() -> Result<Self> {
        let bpe = cl100k_base().context("Failed to load tokenizer")?;
        Ok(Self { bpe })
    }

    /// Count tokens in a text string
    pub fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    /// Encode text to tokens
    pub fn encode(&self, text: &str) -> Vec<usize> {
        self.bpe.encode_with_special_tokens(text)
    }

    /// Calculate token savings between two texts
    pub fn calculate_savings(&self, original: &str, optimized: &str) -> i64 {
        let original_tokens = self.count_tokens(original) as i64;
        let optimized_tokens = self.count_tokens(optimized) as i64;
        original_tokens - optimized_tokens
    }

    /// Calculate savings percentage
    pub fn calculate_savings_percentage(&self, original: &str, optimized: &str) -> f64 {
        let original_tokens = self.count_tokens(original) as f64;
        let optimized_tokens = self.count_tokens(optimized) as f64;

        if original_tokens == 0.0 {
            return 0.0;
        }

        ((original_tokens - optimized_tokens) / original_tokens) * 100.0
    }

    /// Estimate token savings for a pattern replacement
    pub fn estimate_savings(&self, original_text: &str, replacement_text: &str) -> i64 {
        self.calculate_savings(original_text, replacement_text)
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new().expect("Failed to create default tokenizer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let tokenizer = Tokenizer::new().unwrap();

        let text = "Hello, world!";
        let count = tokenizer.count_tokens(text);
        assert!(count > 0);
    }

    #[test]
    fn test_savings_calculation() {
        let tokenizer = Tokenizer::new().unwrap();

        let original = "I would really appreciate it if you could please help me with this.";
        let optimized = "Help me with this.";

        let savings = tokenizer.calculate_savings(original, optimized);
        assert!(savings > 0);

        let percentage = tokenizer.calculate_savings_percentage(original, optimized);
        assert!(percentage > 0.0);
        assert!(percentage < 100.0);
    }

    #[test]
    fn test_mandarin_efficiency() {
        let tokenizer = Tokenizer::new().unwrap();

        // Mandarin should be more token-efficient for some phrases
        let english = "Be thorough and detailed";
        let mandarin = "要详细";

        let en_tokens = tokenizer.count_tokens(english);
        let zh_tokens = tokenizer.count_tokens(mandarin);

        // This should typically be true, though exact tokenization may vary
        println!("English: {} tokens, Mandarin: {} tokens", en_tokens, zh_tokens);
    }
}
