//! Self-contained TF-IDF + Cosine Similarity implementation.
//! Supports unigrams + bigrams, English stop words removal, and Snowball stemming.

use rust_stemmers::{Algorithm, Stemmer};
use std::collections::{HashMap, HashSet};

/// English stop words list
const STOP_WORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and",
    "any", "are", "aren't", "as", "at", "be", "because", "been", "before", "being",
    "below", "between", "both", "but", "by", "can't", "cannot", "could", "couldn't",
    "did", "didn't", "do", "does", "doesn't", "doing", "don't", "down", "during",
    "each", "few", "for", "from", "further", "get", "got", "had", "hadn't", "has",
    "hasn't", "have", "haven't", "having", "he", "her", "here", "hers", "herself",
    "him", "himself", "his", "how", "i", "if", "in", "into", "is", "isn't", "it",
    "its", "itself", "just", "let's", "me", "more", "most", "mustn't", "my", "myself",
    "no", "nor", "not", "of", "off", "on", "once", "only", "or", "other", "ought",
    "our", "ours", "ourselves", "out", "over", "own", "same", "shan't", "she",
    "should", "shouldn't", "so", "some", "such", "than", "that", "the", "their",
    "theirs", "them", "themselves", "then", "there", "these", "they", "this", "those",
    "through", "to", "too", "under", "until", "up", "very", "was", "wasn't", "we",
    "were", "weren't", "what", "when", "where", "which", "while", "who", "whom",
    "why", "will", "with", "won't", "would", "wouldn't", "you", "your", "yours",
    "yourself", "yourselves",
];

/// Tokenize text into lowercase words, filtering non-alphabetic tokens
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Generate unigrams + bigrams from tokens, removing stop words and applying stemming
fn generate_terms(tokens: &[String], stop_words: &HashSet<&str>, stemmer: &Stemmer) -> Vec<String> {
    let mut terms = Vec::new();

    // Stem all tokens
    let stemmed: Vec<String> = tokens.iter().map(|t| stemmer.stem(t).to_string()).collect();

    // Unigrams (skip stop words, use stemmed form)
    for (i, token) in tokens.iter().enumerate() {
        if !stop_words.contains(token.as_str()) {
            terms.push(stemmed[i].clone());
        }
    }

    // Bigrams (use stemmed forms)
    for pair in stemmed.windows(2) {
        terms.push(format!("{} {}", pair[0], pair[1]));
    }

    terms
}

/// Sparse vector representation
type SparseVec = HashMap<usize, f64>;

/// Compute TF-IDF vectors for a collection of documents
pub fn compute_tfidf_vectors(documents: &[String]) -> (Vec<SparseVec>, Vec<String>) {
    let stop_words: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    let stemmer = Stemmer::create(Algorithm::English);
    let n_docs = documents.len();

    // Step 1: Tokenize and generate terms for each document
    let doc_terms: Vec<Vec<String>> = documents
        .iter()
        .map(|doc| {
            let tokens = tokenize(doc);
            generate_terms(&tokens, &stop_words, &stemmer)
        })
        .collect();

    // Step 2: Build vocabulary (term -> index), limited to top 10000 by document frequency
    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    for terms in &doc_terms {
        let unique: HashSet<&String> = terms.iter().collect();
        for term in unique {
            *doc_freq.entry(term.clone()).or_insert(0) += 1;
        }
    }

    // Sort by document frequency (descending) and take top 10000
    let mut vocab_list: Vec<(String, usize)> = doc_freq.into_iter().collect();
    vocab_list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    vocab_list.truncate(10000);

    let vocabulary: HashMap<String, usize> = vocab_list
        .iter()
        .enumerate()
        .map(|(idx, (term, _))| (term.clone(), idx))
        .collect();

    let vocab_names: Vec<String> = {
        let mut names = vec![String::new(); vocabulary.len()];
        for (term, &idx) in &vocabulary {
            names[idx] = term.clone();
        }
        names
    };

    // IDF values
    let idf: Vec<f64> = vocab_names
        .iter()
        .map(|term| {
            let df = vocab_list
                .iter()
                .find(|(t, _)| t == term)
                .map(|(_, c)| *c)
                .unwrap_or(1);
            (n_docs as f64 / df as f64).ln() + 1.0
        })
        .collect();

    // Step 3: Compute TF-IDF for each document
    let vectors: Vec<SparseVec> = doc_terms
        .iter()
        .map(|terms| {
            // Term frequency
            let mut tf: HashMap<usize, f64> = HashMap::new();
            let total = terms.len() as f64;
            for term in terms {
                if let Some(&idx) = vocabulary.get(term) {
                    *tf.entry(idx).or_insert(0.0) += 1.0;
                }
            }

            // TF-IDF = tf * idf
            let mut vec: SparseVec = HashMap::new();
            for (&idx, &count) in &tf {
                let tfidf = (count / total) * idf[idx];
                if tfidf > 0.0 {
                    vec.insert(idx, tfidf);
                }
            }

            // L2 normalize
            let norm: f64 = vec.values().map(|v| v * v).sum::<f64>().sqrt();
            if norm > 0.0 {
                for val in vec.values_mut() {
                    *val /= norm;
                }
            }

            vec
        })
        .collect();

    (vectors, vocab_names)
}

/// Compute cosine similarity between two sparse vectors (already L2-normalized)
pub fn cosine_similarity(a: &SparseVec, b: &SparseVec) -> f64 {
    // Since both are L2-normalized, cosine = dot product
    let (smaller, larger) = if a.len() < b.len() { (a, b) } else { (b, a) };
    smaller
        .iter()
        .filter_map(|(idx, val_a)| larger.get(idx).map(|val_b| val_a * val_b))
        .sum()
}

/// Compute pairwise similarities above threshold, returns (idx_a, idx_b, score)
pub fn compute_pairwise_similarities(
    vectors: &[SparseVec],
    threshold: f64,
) -> Vec<(usize, usize, f64)> {
    let n = vectors.len();
    let mut results = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let score = cosine_similarity(&vectors[i], &vectors[j]);
            if score >= threshold {
                results.push((i, j, (score * 1000.0).round() / 10.0)); // round to 1 decimal as percentage
            }
        }
    }

    results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello, World! This is a TEST.");
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_identical_documents() {
        let docs = vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "The quick brown fox jumps over the lazy dog".to_string(),
        ];
        let (vectors, _) = compute_tfidf_vectors(&docs);
        let sim = cosine_similarity(&vectors[0], &vectors[1]);
        assert!((sim - 1.0).abs() < 0.001, "Identical docs should have similarity ~1.0, got {sim}");
    }

    #[test]
    fn test_different_documents() {
        let docs = vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "Mathematics and physics are fundamental sciences".to_string(),
        ];
        let (vectors, _) = compute_tfidf_vectors(&docs);
        let sim = cosine_similarity(&vectors[0], &vectors[1]);
        assert!(sim < 0.3, "Different docs should have low similarity, got {sim}");
    }

    #[test]
    fn test_pairwise_threshold() {
        let docs = vec![
            "The cat sat on the mat in the house".to_string(),
            "The cat sat on the mat in the garden".to_string(),
            "Python programming language is great for data science".to_string(),
        ];
        let (vectors, _) = compute_tfidf_vectors(&docs);
        let results = compute_pairwise_similarities(&vectors, 0.5);
        // First two docs should be similar, third should be different
        assert!(!results.is_empty(), "Should find at least one similar pair");
        assert_eq!(results[0].0, 0);
        assert_eq!(results[0].1, 1);
    }
}
