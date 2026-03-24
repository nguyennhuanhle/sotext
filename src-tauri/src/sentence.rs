//! Sentence-level comparison using Jaccard Similarity and Levenshtein Distance.
//! Detects paraphrased content that N-gram matching would miss.

use serde::Serialize;
use std::collections::HashSet;

/// A pair of suspicious sentences with similarity scores
#[derive(Debug, Clone, Serialize)]
pub struct SuspiciousPair {
    pub sentence_a: String,
    pub sentence_b: String,
    pub jaccard_score: f64,
    pub levenshtein_score: f64,
    pub pos_a: [usize; 2], // [start, end] in original text
    pub pos_b: [usize; 2],
}

/// Split text into sentences (by period, exclamation, question mark, or double newline)
pub fn split_sentences(text: &str) -> Vec<(String, usize, usize)> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for sentence-ending punctuation
        if chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
            // Look ahead to skip multiple punctuation or whitespace
            let mut end = i + 1;
            while end < chars.len() && (chars[end].is_whitespace() || chars[end] == '.' || chars[end] == '!' || chars[end] == '?') {
                end += 1;
            }

            let sentence_text: String = chars[start..=i].iter().collect();
            let trimmed = sentence_text.trim();
            if !trimmed.is_empty() && trimmed.split_whitespace().count() >= 3 {
                // byte positions
                let byte_start = text.char_indices().nth(start).map(|(b, _)| b).unwrap_or(0);
                let byte_end = text.char_indices().nth(i + 1).map(|(b, _)| b).unwrap_or(text.len());
                sentences.push((trimmed.to_string(), byte_start, byte_end));
            }
            start = end;
            i = end;
        } else if chars[i] == '\n' && i + 1 < chars.len() && chars[i + 1] == '\n' {
            // Double newline = paragraph break
            let sentence_text: String = chars[start..i].iter().collect();
            let trimmed = sentence_text.trim();
            if !trimmed.is_empty() && trimmed.split_whitespace().count() >= 3 {
                let byte_start = text.char_indices().nth(start).map(|(b, _)| b).unwrap_or(0);
                let byte_end = text.char_indices().nth(i).map(|(b, _)| b).unwrap_or(text.len());
                sentences.push((trimmed.to_string(), byte_start, byte_end));
            }
            start = i + 2;
            i = start;
        } else {
            i += 1;
        }
    }

    // Remaining text
    if start < chars.len() {
        let sentence_text: String = chars[start..].iter().collect();
        let trimmed = sentence_text.trim();
        if !trimmed.is_empty() && trimmed.split_whitespace().count() >= 3 {
            let byte_start = text.char_indices().nth(start).map(|(b, _)| b).unwrap_or(0);
            sentences.push((trimmed.to_string(), byte_start, text.len()));
        }
    }

    sentences
}

/// Compute Jaccard similarity between two sentences (word-set based)
pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let words_a: HashSet<String> = a.split_whitespace().map(|w| w.to_lowercase()).collect();
    let words_b: HashSet<String> = b.split_whitespace().map(|w| w.to_lowercase()).collect();

    let intersection = words_a.intersection(&words_b).count() as f64;
    let union = words_a.union(&words_b).count() as f64;

    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

/// Compute normalized Levenshtein similarity (1 - distance/max_length)
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let dist = strsim::levenshtein(&a_lower, &b_lower) as f64;
    let max_len = a_lower.len().max(b_lower.len()) as f64;
    if max_len == 0.0 {
        1.0
    } else {
        1.0 - (dist / max_len)
    }
}

/// Find suspicious sentence pairs between two texts.
/// Returns pairs with Jaccard OR Levenshtein score above threshold.
pub fn find_suspicious_sentences(
    text_a: &str,
    text_b: &str,
    threshold: f64,
) -> Vec<SuspiciousPair> {
    let sentences_a = split_sentences(text_a);
    let sentences_b = split_sentences(text_b);
    let mut results = Vec::new();

    for (sent_a, start_a, end_a) in &sentences_a {
        for (sent_b, start_b, end_b) in &sentences_b {
            let jaccard = jaccard_similarity(sent_a, sent_b);
            let lev = levenshtein_similarity(sent_a, sent_b);

            // Flag if either metric exceeds threshold
            if jaccard >= threshold || lev >= threshold {
                results.push(SuspiciousPair {
                    sentence_a: sent_a.clone(),
                    sentence_b: sent_b.clone(),
                    jaccard_score: (jaccard * 100.0).round() / 100.0,
                    levenshtein_score: (lev * 100.0).round() / 100.0,
                    pos_a: [*start_a, *end_a],
                    pos_b: [*start_b, *end_b],
                });
            }
        }
    }

    // Sort by max of both scores, descending
    results.sort_by(|a, b| {
        let max_a = a.jaccard_score.max(a.levenshtein_score);
        let max_b = b.jaccard_score.max(b.levenshtein_score);
        max_b.partial_cmp(&max_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

/// Remove sentences from text that match template sentences (Jaccard > 0.9)
pub fn strip_template(text: &str, template: &str) -> String {
    let text_sentences = split_sentences(text);
    let template_sentences = split_sentences(template);

    let mut result_parts: Vec<&str> = Vec::new();
    let mut last_end = 0;

    for (sent, start, end) in &text_sentences {
        let is_template = template_sentences.iter().any(|(tmpl, _, _)| {
            jaccard_similarity(sent, tmpl) > 0.9
        });

        if is_template {
            // Include text before this sentence
            if *start > last_end {
                result_parts.push(&text[last_end..*start]);
            }
            last_end = *end;
        }
    }

    // Include remaining text
    if last_end < text.len() {
        result_parts.push(&text[last_end..]);
    }

    let result = result_parts.join("");
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences() {
        let text = "Hello world this is test. Another sentence here. And a third one.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert!(sentences[0].0.contains("Hello"));
        assert!(sentences[1].0.contains("Another"));
    }

    #[test]
    fn test_jaccard_reordered_words() {
        let a = "the cat is sleeping on the mat";
        let b = "on the mat the cat is sleeping";
        let score = jaccard_similarity(a, b);
        assert!((score - 1.0).abs() < 0.001, "Reordered words should be ~1.0, got {score}");
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let a = "the quick brown fox";
        let b = "the slow red fox";
        let score = jaccard_similarity(a, b);
        // intersection: {the, fox} = 2, union: {the, quick, brown, fox, slow, red} = 6
        assert!((score - 2.0 / 6.0).abs() < 0.01);
    }

    #[test]
    fn test_levenshtein_similar() {
        let a = "smoking should be completely banned";
        let b = "smoking should be completly banned";
        let score = levenshtein_similarity(a, b);
        assert!(score > 0.9, "One typo should give high similarity, got {score}");
    }

    #[test]
    fn test_strip_template() {
        let template = "Answer the following question. What is your opinion on smoking?";
        let student = "Answer the following question. Smoking should be banned in all restaurants. It is harmful.";
        let stripped = strip_template(student, template);
        assert!(!stripped.contains("Answer the following question"));
        assert!(stripped.contains("Smoking should be banned"));
    }

    #[test]
    fn test_find_suspicious_paraphrased() {
        // Use two sentences (with periods) that share most words in different order
        let text_a = "The cat was sleeping on the soft mat in the big living room.";
        let text_b = "In the big living room on the soft mat the cat was sleeping.";
        let pairs = find_suspicious_sentences(text_a, text_b, 0.6);
        assert!(!pairs.is_empty(), "Reordered sentence should be flagged, got: {:?}", pairs);
    }
}
