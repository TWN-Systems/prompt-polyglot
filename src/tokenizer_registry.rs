/// Phase 3: Tokenizer Registry - Multi-tokenizer abstraction layer
///
/// Purpose: Abstract over different tokenizer backends (tiktoken, HuggingFace)
/// to enable token cost comparison across models (GPT, Claude, Llama, etc.)

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Tokenizer identifier for database lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenizerId {
    /// OpenAI GPT-4, GPT-3.5-turbo (cl100k_base)
    Cl100kBase,
    /// Meta Llama 3
    Llama3,
    /// Anthropic Claude (uses cl100k_base)
    Claude,
}

impl TokenizerId {
    /// Convert to database string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cl100kBase => "cl100k_base",
            Self::Llama3 => "llama3",
            Self::Claude => "claude",
        }
    }

    /// Parse from database string
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "cl100k_base" => Ok(Self::Cl100kBase),
            "llama3" => Ok(Self::Llama3),
            "claude" => Ok(Self::Claude),
            _ => Err(anyhow!("Unknown tokenizer ID: {}", s)),
        }
    }

    /// Get all supported tokenizer IDs
    pub fn all() -> Vec<Self> {
        vec![Self::Cl100kBase, Self::Llama3, Self::Claude]
    }
}

impl std::fmt::Display for TokenizerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Common interface for all tokenizer backends
pub trait TokenizerBackend: Send + Sync {
    /// Count tokens in text
    fn count_tokens(&self, text: &str) -> usize;

    /// Encode text to token IDs (for debugging/analysis)
    fn encode(&self, text: &str) -> Vec<u32>;

    /// Decode token IDs back to text (for debugging/analysis)
    fn decode(&self, tokens: &[u32]) -> Result<String>;

    /// Get tokenizer ID
    fn id(&self) -> TokenizerId;
}

/// tiktoken-based backend (for OpenAI models and Claude)
pub struct TiktokenBackend {
    bpe: tiktoken_rs::CoreBPE,
    id: TokenizerId,
}

impl TiktokenBackend {
    /// Create cl100k_base tokenizer (GPT-4, GPT-3.5-turbo)
    pub fn cl100k_base() -> Result<Self> {
        let bpe = tiktoken_rs::cl100k_base()
            .map_err(|e| anyhow!("Failed to load cl100k_base: {}", e))?;
        Ok(Self {
            bpe,
            id: TokenizerId::Cl100kBase,
        })
    }

    /// Create Claude tokenizer (uses cl100k_base)
    pub fn claude() -> Result<Self> {
        let bpe = tiktoken_rs::cl100k_base()
            .map_err(|e| anyhow!("Failed to load claude tokenizer: {}", e))?;
        Ok(Self {
            bpe,
            id: TokenizerId::Claude,
        })
    }
}

impl TokenizerBackend for TiktokenBackend {
    fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    fn encode(&self, text: &str) -> Vec<u32> {
        self.bpe
            .encode_with_special_tokens(text)
            .into_iter()
            .map(|t| t as u32)
            .collect()
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let tokens_usize: Vec<usize> = tokens.iter().map(|&t| t as usize).collect();
        self.bpe
            .decode(tokens_usize)
            .map_err(|e| anyhow!("Decode failed: {}", e))
    }

    fn id(&self) -> TokenizerId {
        self.id
    }
}

/// HuggingFace tokenizers backend (for Llama, etc.)
pub struct HuggingFaceBackend {
    tokenizer: tokenizers::Tokenizer,
    id: TokenizerId,
}

impl HuggingFaceBackend {
    /// Create Llama 3 tokenizer
    pub fn llama3() -> Result<Self> {
        // Note: In production, load from HuggingFace model hub or local file
        // For now, we'll use a placeholder that needs to be configured
        Err(anyhow!(
            "Llama3 tokenizer not yet configured. Please provide tokenizer.json path."
        ))
    }

    /// Create from tokenizer.json file
    pub fn from_file(path: &str, id: TokenizerId) -> Result<Self> {
        let tokenizer = tokenizers::Tokenizer::from_file(path)
            .map_err(|e| anyhow!("Failed to load tokenizer from {}: {}", path, e))?;
        Ok(Self { tokenizer, id })
    }
}

impl TokenizerBackend for HuggingFaceBackend {
    fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer
            .encode(text, false)
            .map(|enc| enc.len())
            .unwrap_or(0)
    }

    fn encode(&self, text: &str) -> Vec<u32> {
        self.tokenizer
            .encode(text, false)
            .map(|enc| enc.get_ids().to_vec())
            .unwrap_or_default()
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        self.tokenizer
            .decode(tokens, false)
            .map_err(|e| anyhow!("Decode failed: {}", e))
    }

    fn id(&self) -> TokenizerId {
        self.id
    }
}

/// Registry managing all available tokenizers
pub struct TokenizerRegistry {
    backends: HashMap<TokenizerId, Arc<dyn TokenizerBackend>>,
}

impl TokenizerRegistry {
    /// Create new registry with default tokenizers
    pub fn new() -> Result<Self> {
        let mut backends: HashMap<TokenizerId, Arc<dyn TokenizerBackend>> = HashMap::new();

        // Load cl100k_base (GPT-4, GPT-3.5)
        if let Ok(backend) = TiktokenBackend::cl100k_base() {
            backends.insert(TokenizerId::Cl100kBase, Arc::new(backend));
        }

        // Load Claude (same as cl100k_base)
        if let Ok(backend) = TiktokenBackend::claude() {
            backends.insert(TokenizerId::Claude, Arc::new(backend));
        }

        // Note: Llama3 requires external tokenizer.json file
        // Users can add via register_backend()

        if backends.is_empty() {
            return Err(anyhow!("No tokenizers loaded successfully"));
        }

        Ok(Self { backends })
    }

    /// Register a custom tokenizer backend
    pub fn register_backend(&mut self, backend: Arc<dyn TokenizerBackend>) {
        self.backends.insert(backend.id(), backend);
    }

    /// Get tokenizer backend by ID
    pub fn get(&self, id: TokenizerId) -> Option<Arc<dyn TokenizerBackend>> {
        self.backends.get(&id).cloned()
    }

    /// Check if tokenizer is available
    pub fn has(&self, id: TokenizerId) -> bool {
        self.backends.contains_key(&id)
    }

    /// Get all available tokenizer IDs
    pub fn available(&self) -> Vec<TokenizerId> {
        self.backends.keys().copied().collect()
    }

    /// Count tokens across all tokenizers for comparison
    pub fn count_all(&self, text: &str) -> HashMap<TokenizerId, usize> {
        self.backends
            .iter()
            .map(|(id, backend)| (*id, backend.count_tokens(text)))
            .collect()
    }

    /// Find tokenizer that produces minimum token count for given text
    pub fn find_cheapest(&self, text: &str) -> Option<(TokenizerId, usize)> {
        self.backends
            .iter()
            .map(|(id, backend)| (*id, backend.count_tokens(text)))
            .min_by_key(|(_, count)| *count)
    }
}

impl Default for TokenizerRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to initialize default tokenizer registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_id_conversion() {
        assert_eq!(TokenizerId::Cl100kBase.as_str(), "cl100k_base");
        assert_eq!(TokenizerId::Llama3.as_str(), "llama3");
        assert_eq!(TokenizerId::Claude.as_str(), "claude");

        assert_eq!(
            TokenizerId::from_str("cl100k_base").unwrap(),
            TokenizerId::Cl100kBase
        );
        assert_eq!(
            TokenizerId::from_str("llama3").unwrap(),
            TokenizerId::Llama3
        );
        assert_eq!(
            TokenizerId::from_str("claude").unwrap(),
            TokenizerId::Claude
        );

        assert!(TokenizerId::from_str("unknown").is_err());
    }

    #[test]
    fn test_cl100k_base_tokenization() {
        let backend = TiktokenBackend::cl100k_base().unwrap();

        // Test basic English
        let count = backend.count_tokens("Hello, world!");
        assert!(count > 0);
        assert!(count <= 5); // Should be 3-4 tokens

        // Test that we can encode and decode
        let tokens = backend.encode("Hello");
        assert!(!tokens.is_empty());

        let decoded = backend.decode(&tokens).unwrap();
        assert_eq!(decoded, "Hello");
    }

    #[test]
    fn test_tokenizer_registry() {
        let registry = TokenizerRegistry::new().unwrap();

        // Should have at least cl100k_base and claude
        assert!(registry.has(TokenizerId::Cl100kBase));
        assert!(registry.has(TokenizerId::Claude));

        let available = registry.available();
        assert!(!available.is_empty());
    }

    #[test]
    fn test_count_all() {
        let registry = TokenizerRegistry::new().unwrap();

        let counts = registry.count_all("Analyze this code thoroughly.");
        assert!(!counts.is_empty());

        // All counts should be positive
        for (id, count) in counts {
            assert!(count > 0, "Tokenizer {} produced 0 tokens", id);
        }
    }

    #[test]
    fn test_find_cheapest() {
        let registry = TokenizerRegistry::new().unwrap();

        let result = registry.find_cheapest("Hospital");
        assert!(result.is_some());

        let (id, count) = result.unwrap();
        assert!(count > 0);
        println!("Cheapest tokenizer for 'Hospital': {} ({} tokens)", id, count);
    }
}
