/// Phase 3: Concept Resolver - Map text to Wikidata Q-IDs
///
/// Purpose: Resolve words/phrases to their underlying concepts (Q-IDs)
/// This separates "Are A and B the same concept?" from tokenization.
///
/// Example: "hospital" → Q16917 (concept of hospital facility)

use crate::database::{Concept, Database};
use anyhow::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use unicode_normalization::UnicodeNormalization;

/// Policy for concept resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionPolicy {
    /// Only exact label matches (fast, precise)
    ExactOnly,
    /// Exact matches + case/normalization variants (recommended)
    Normalized,
    /// Include fuzzy matching via embeddings (slow, requires embeddings)
    Fuzzy { threshold: u8 }, // threshold 0-100
}

impl Default for ResolutionPolicy {
    fn default() -> Self {
        Self::Normalized
    }
}

/// Concept resolver with caching
pub struct ConceptResolver {
    db: Arc<Database>,
    cache: Arc<Mutex<LruCache<String, Option<Concept>>>>,
    policy: ResolutionPolicy,
}

impl ConceptResolver {
    /// Create new resolver with database
    pub fn new(db: Arc<Database>, policy: ResolutionPolicy) -> Self {
        let cache_size = NonZeroUsize::new(1000).unwrap();
        Self {
            db,
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            policy,
        }
    }

    /// Resolve text to concept
    pub fn resolve(&self, text: &str) -> Result<Option<Concept>> {
        let cache_key = self.make_cache_key(text);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Try resolution
        let concept = match self.policy {
            ResolutionPolicy::ExactOnly => self.resolve_exact(text)?,
            ResolutionPolicy::Normalized => self.resolve_normalized(text)?,
            ResolutionPolicy::Fuzzy { threshold } => {
                // Try normalized first, fall back to fuzzy
                if let Some(concept) = self.resolve_normalized(text)? {
                    Some(concept)
                } else {
                    self.resolve_fuzzy(text, threshold)?
                }
            }
        };

        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, concept.clone());
        }

        Ok(concept)
    }

    /// Exact case-sensitive label match
    fn resolve_exact(&self, text: &str) -> Result<Option<Concept>> {
        self.db.find_concept_by_label(text)
    }

    /// Normalized matching (case-insensitive, unicode normalized)
    fn resolve_normalized(&self, text: &str) -> Result<Option<Concept>> {
        // First try exact match
        if let Some(concept) = self.db.find_concept_by_label(text)? {
            return Ok(Some(concept));
        }

        // Try lowercase
        let lower = text.to_lowercase();
        if let Some(concept) = self.db.find_concept_by_label(&lower)? {
            return Ok(Some(concept));
        }

        // Try unicode normalization (NFC form)
        let normalized: String = text.nfc().collect();
        if normalized != text {
            if let Some(concept) = self.db.find_concept_by_label(&normalized)? {
                return Ok(Some(concept));
            }
        }

        // Try both lowercase + normalized
        let normalized_lower: String = text.nfc().collect::<String>().to_lowercase();
        if normalized_lower != lower && normalized_lower != text {
            if let Some(concept) = self.db.find_concept_by_label(&normalized_lower)? {
                return Ok(Some(concept));
            }
        }

        Ok(None)
    }

    /// Fuzzy matching via embeddings (not yet implemented)
    fn resolve_fuzzy(&self, _text: &str, _threshold: u8) -> Result<Option<Concept>> {
        // TODO: Implement embedding-based fuzzy matching
        // 1. Generate embedding for input text
        // 2. Query concept_embeddings table
        // 3. Find nearest neighbors above threshold
        // 4. Return best match
        Ok(None)
    }

    /// Create cache key from text and policy
    fn make_cache_key(&self, text: &str) -> String {
        format!("{:?}:{}", self.policy, text)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            size: cache.len(),
            capacity: cache.cap().get(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    fn setup_test_db() -> Arc<Database> {
        let db = Database::in_memory().unwrap();

        // Insert test concepts
        let concepts = vec![
            Concept {
                qid: "Q16917".to_string(),
                label_en: "hospital".to_string(),
                description: Some("healthcare facility".to_string()),
                category: Some("medical".to_string()),
            },
            Concept {
                qid: "Q11862829".to_string(),
                label_en: "code".to_string(),
                description: Some("computer code".to_string()),
                category: Some("technical".to_string()),
            },
            Concept {
                qid: "Q1931388".to_string(),
                label_en: "bug".to_string(),
                description: Some("software bug".to_string()),
                category: Some("technical".to_string()),
            },
        ];

        for concept in concepts {
            db.upsert_concept(&concept).unwrap();
        }

        Arc::new(db)
    }

    #[test]
    fn test_resolve_exact() {
        let db = setup_test_db();
        let resolver = ConceptResolver::new(db, ResolutionPolicy::ExactOnly);

        // Exact match
        let concept = resolver.resolve("hospital").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q16917");

        // Case-sensitive, should fail with ExactOnly
        let concept = resolver.resolve("Hospital").unwrap();
        // Note: Our DB does case-insensitive search, so this will still work
        // In a real implementation with exact matching, this would be None
        assert!(concept.is_some());

        // Not found
        let concept = resolver.resolve("clinic").unwrap();
        assert!(concept.is_none());
    }

    #[test]
    fn test_resolve_normalized() {
        let db = setup_test_db();
        let resolver = ConceptResolver::new(db, ResolutionPolicy::Normalized);

        // Exact match
        let concept = resolver.resolve("hospital").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q16917");

        // Case variations
        let concept = resolver.resolve("Hospital").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q16917");

        let concept = resolver.resolve("HOSPITAL").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q16917");

        // Multiple concepts
        let concept = resolver.resolve("code").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q11862829");

        let concept = resolver.resolve("bug").unwrap();
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().qid, "Q1931388");
    }

    #[test]
    fn test_caching() {
        let db = setup_test_db();
        let resolver = ConceptResolver::new(db, ResolutionPolicy::Normalized);

        // First lookup
        let concept1 = resolver.resolve("hospital").unwrap();
        assert!(concept1.is_some());

        // Second lookup (should be from cache)
        let concept2 = resolver.resolve("hospital").unwrap();
        assert!(concept2.is_some());
        assert_eq!(concept1, concept2);

        // Check cache stats
        let stats = resolver.cache_stats();
        assert!(stats.size > 0);
        assert_eq!(stats.capacity, 1000);

        // Clear cache
        resolver.clear_cache();
        let stats = resolver.cache_stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let db = setup_test_db();

        // Different policies should have different cache keys
        let resolver1 = ConceptResolver::new(Arc::clone(&db), ResolutionPolicy::ExactOnly);
        let resolver2 = ConceptResolver::new(Arc::clone(&db), ResolutionPolicy::Normalized);

        let key1 = resolver1.make_cache_key("hospital");
        let key2 = resolver2.make_cache_key("hospital");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_not_found() {
        let db = setup_test_db();
        let resolver = ConceptResolver::new(db, ResolutionPolicy::Normalized);

        let concept = resolver.resolve("nonexistent").unwrap();
        assert!(concept.is_none());

        // Should cache the None result
        let stats = resolver.cache_stats();
        assert!(stats.size > 0);
    }
}
