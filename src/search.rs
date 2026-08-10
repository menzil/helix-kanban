use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Return the best fuzzy score across independent searchable fields.
pub fn fuzzy_match_best_score(query: &str, fields: &[&str]) -> Option<u16> {
    if query.trim().is_empty() {
        return Some(0);
    }

    let pattern = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let mut matcher = Matcher::new(Config::DEFAULT);
    fields.iter().fold(None, |best, field| {
        let mut field_chars = Vec::new();
        let score = pattern.score(Utf32Str::new(field, &mut field_chars), &mut matcher);
        match (best, score) {
            (Some(best_score), Some(score)) => Some(best_score.max(score)),
            (None, Some(score)) => Some(score),
            (best_score, None) => best_score,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::fuzzy_match_best_score;

    #[test]
    fn matches_non_contiguous_characters_and_ranks_exact_prefix() {
        let feature_notes_score =
            fuzzy_match_best_score("fnt", &["feature notes"]).expect("feature notes should match");
        let frontend_score =
            fuzzy_match_best_score("fnt", &["frontend"]).expect("frontend should match");

        assert!(frontend_score > feature_notes_score);
        assert!(fuzzy_match_best_score("front", &["frontend"]).is_some());
    }

    #[test]
    fn matching_ignores_case() {
        assert!(fuzzy_match_best_score("rn", &["Release Notes"]).is_some());
    }

    #[test]
    fn returns_best_match_across_independent_fields() {
        let fields = vec!["42", "Release Notes"];

        assert!(fuzzy_match_best_score("42", &fields).is_some());
        assert!(fuzzy_match_best_score("rn", &fields).is_some());
        assert!(fuzzy_match_best_score("2r", &fields).is_none());
    }
}
