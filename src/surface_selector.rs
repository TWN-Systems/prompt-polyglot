/// Phase 3: Surface Form Selector - Pick optimal variant for a concept
///
/// Purpose: Given a concept (Q-ID) and tokenizer, select the surface form
/// with minimum token count, respecting policy constraints.
///
/// Example: Q16917 with cl100k_base → "hospital" (1 token) vs "医院" (4 tokens)

use crate::database::{Database, SurfaceForm};
use crate::tokenizer_registry::TokenizerId;
use anyhow::Result;
use std::sync::Arc;

/// Policy for selecting surface forms
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionPolicy {
    /// Always pick minimum tokens, any language
    MinTokens,
    /// Minimum tokens, but only within same language as original
    SameLanguage { lang: String },
    /// Minimum tokens, but only from allowed languages
    AllowedLanguages { langs: Vec<String> },
    /// Minimum tokens, but prefer original language (tie-breaker)
    PreferOriginalLanguage { lang: String },
}

impl Default for SelectionPolicy {
    fn default() -> Self {
        Self::MinTokens
    }
}

/// Surface form selector
pub struct SurfaceSelector {
    db: Arc<Database>,
}

impl SurfaceSelector {
    /// Create new surface selector
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Select best surface form for concept
    pub fn select(
        &self,
        qid: &str,
        tokenizer_id: TokenizerId,
        policy: &SelectionPolicy,
    ) -> Result<Option<SurfaceForm>> {
        // Get all surface forms for this concept and tokenizer
        let forms = self
            .db
            .get_surface_forms(qid, tokenizer_id.as_str())?;

        if forms.is_empty() {
            return Ok(None);
        }

        // Apply policy to select best form
        let selected = match policy {
            SelectionPolicy::MinTokens => self.select_min_tokens(&forms),
            SelectionPolicy::SameLanguage { lang } => {
                self.select_same_language(&forms, lang)
            }
            SelectionPolicy::AllowedLanguages { langs } => {
                self.select_allowed_languages(&forms, langs)
            }
            SelectionPolicy::PreferOriginalLanguage { lang } => {
                self.select_prefer_language(&forms, lang)
            }
        };

        Ok(selected)
    }

    /// Select form with minimum token count
    fn select_min_tokens(&self, forms: &[SurfaceForm]) -> Option<SurfaceForm> {
        forms
            .iter()
            .min_by_key(|f| f.token_count)
            .cloned()
    }

    /// Select minimum tokens within same language
    fn select_same_language(&self, forms: &[SurfaceForm], lang: &str) -> Option<SurfaceForm> {
        forms
            .iter()
            .filter(|f| f.lang == lang)
            .min_by_key(|f| f.token_count)
            .cloned()
    }

    /// Select minimum tokens from allowed languages
    fn select_allowed_languages(
        &self,
        forms: &[SurfaceForm],
        langs: &[String],
    ) -> Option<SurfaceForm> {
        forms
            .iter()
            .filter(|f| langs.contains(&f.lang))
            .min_by_key(|f| f.token_count)
            .cloned()
    }

    /// Select minimum tokens, prefer original language on tie
    fn select_prefer_language(&self, forms: &[SurfaceForm], lang: &str) -> Option<SurfaceForm> {
        let min_tokens = forms.iter().map(|f| f.token_count).min()?;

        // Get all forms with minimum token count
        let min_forms: Vec<_> = forms
            .iter()
            .filter(|f| f.token_count == min_tokens)
            .collect();

        // Prefer original language
        min_forms
            .iter()
            .find(|f| f.lang == lang)
            .or_else(|| min_forms.first())
            .map(|f| (*f).clone())
    }

    /// Calculate token savings vs original
    pub fn calculate_savings(
        &self,
        qid: &str,
        tokenizer_id: TokenizerId,
        original_form: &str,
        original_tokens: usize,
        policy: &SelectionPolicy,
    ) -> Result<Option<OptimizationCandidate>> {
        let selected = self.select(qid, tokenizer_id, policy)?;

        if let Some(form) = selected {
            // Don't suggest if it's the same form
            if form.form == original_form {
                return Ok(None);
            }

            // Calculate savings
            let token_savings = original_tokens as i64 - form.token_count as i64;

            // Only suggest if we actually save tokens
            if token_savings > 0 {
                return Ok(Some(OptimizationCandidate {
                    original_form: original_form.to_string(),
                    original_tokens,
                    optimized_form: form.form.clone(),
                    optimized_tokens: form.token_count,
                    token_savings,
                    language: form.lang.clone(),
                    qid: qid.to_string(),
                }));
            }
        }

        Ok(None)
    }
}

/// Optimization candidate with token savings
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationCandidate {
    pub original_form: String,
    pub original_tokens: usize,
    pub optimized_form: String,
    pub optimized_tokens: usize,
    pub token_savings: i64,
    pub language: String,
    pub qid: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Concept, Database};

    fn setup_test_db() -> Arc<Database> {
        let db = Database::in_memory().unwrap();

        // Insert concept
        let concept = Concept {
            qid: "Q16917".to_string(),
            label_en: "hospital".to_string(),
            description: Some("healthcare facility".to_string()),
            category: Some("medical".to_string()),
        };
        db.upsert_concept(&concept).unwrap();

        // Insert surface forms with different token counts
        let forms = vec![
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "en".to_string(),
                form: "hospital".to_string(),
                token_count: 1,
                char_count: 8,
            },
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "zh".to_string(),
                form: "医院".to_string(),
                token_count: 4,
                char_count: 2,
            },
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "es".to_string(),
                form: "hospital".to_string(),
                token_count: 1,
                char_count: 8,
            },
            SurfaceForm {
                qid: "Q16917".to_string(),
                tokenizer_id: "cl100k_base".to_string(),
                lang: "fr".to_string(),
                form: "hôpital".to_string(),
                token_count: 2,
                char_count: 7,
            },
        ];

        for form in forms {
            db.insert_surface_form(&form).unwrap();
        }

        Arc::new(db)
    }

    #[test]
    fn test_select_min_tokens() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        let policy = SelectionPolicy::MinTokens;
        let result = selector
            .select("Q16917", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_some());
        let form = result.unwrap();
        assert_eq!(form.token_count, 1); // Should pick one of the 1-token forms
        assert!(form.form == "hospital"); // Either "en" or "es"
    }

    #[test]
    fn test_select_same_language() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        // Request French only
        let policy = SelectionPolicy::SameLanguage {
            lang: "fr".to_string(),
        };
        let result = selector
            .select("Q16917", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_some());
        let form = result.unwrap();
        assert_eq!(form.lang, "fr");
        assert_eq!(form.form, "hôpital");
        assert_eq!(form.token_count, 2);
    }

    #[test]
    fn test_select_allowed_languages() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        // Only allow French and Chinese
        let policy = SelectionPolicy::AllowedLanguages {
            langs: vec!["fr".to_string(), "zh".to_string()],
        };
        let result = selector
            .select("Q16917", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_some());
        let form = result.unwrap();
        assert_eq!(form.lang, "fr"); // French has 2 tokens, Chinese has 4
        assert_eq!(form.token_count, 2);
    }

    #[test]
    fn test_select_prefer_original_language() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        // Prefer English (which has 1 token - tied for minimum)
        let policy = SelectionPolicy::PreferOriginalLanguage {
            lang: "en".to_string(),
        };
        let result = selector
            .select("Q16917", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_some());
        let form = result.unwrap();
        assert_eq!(form.lang, "en"); // Should prefer English over Spanish
        assert_eq!(form.token_count, 1);

        // Prefer French (which has 2 tokens - not minimum)
        let policy = SelectionPolicy::PreferOriginalLanguage {
            lang: "fr".to_string(),
        };
        let result = selector
            .select("Q16917", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_some());
        let form = result.unwrap();
        // Should still pick a 1-token form since French isn't minimum
        assert_eq!(form.token_count, 1);
    }

    #[test]
    fn test_calculate_savings() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        // Original is Chinese (4 tokens), should optimize to English (1 token)
        let policy = SelectionPolicy::MinTokens;
        let result = selector
            .calculate_savings(
                "Q16917",
                TokenizerId::Cl100kBase,
                "医院",
                4,
                &policy,
            )
            .unwrap();

        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.original_form, "医院");
        assert_eq!(candidate.original_tokens, 4);
        assert_eq!(candidate.optimized_form, "hospital");
        assert_eq!(candidate.optimized_tokens, 1);
        assert_eq!(candidate.token_savings, 3);
    }

    #[test]
    fn test_no_savings() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        // Original is already optimal (1 token)
        let policy = SelectionPolicy::MinTokens;
        let result = selector
            .calculate_savings(
                "Q16917",
                TokenizerId::Cl100kBase,
                "hospital",
                1,
                &policy,
            )
            .unwrap();

        // Should be None because we can't improve
        assert!(result.is_none());
    }

    #[test]
    fn test_nonexistent_concept() {
        let db = setup_test_db();
        let selector = SurfaceSelector::new(db);

        let policy = SelectionPolicy::MinTokens;
        let result = selector
            .select("Q99999", TokenizerId::Cl100kBase, &policy)
            .unwrap();

        assert!(result.is_none());
    }
}
