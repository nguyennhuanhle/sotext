//! Self-contained TF-IDF + Cosine Similarity implementation.
//! Supports unigrams + bigrams, multi-language stop words removal, and Snowball stemming.

use rust_stemmers::{Algorithm, Stemmer};
use std::collections::{HashMap, HashSet};
use whatlang::{detect, Lang};

/// Map whatlang Lang to rust-stemmers Algorithm (returns None for unsupported languages)
fn lang_to_stemmer_algorithm(lang: Lang) -> Option<Algorithm> {
    match lang {
        Lang::Eng => Some(Algorithm::English),
        Lang::Fra => Some(Algorithm::French),
        Lang::Deu => Some(Algorithm::German),
        Lang::Spa => Some(Algorithm::Spanish),
        Lang::Por => Some(Algorithm::Portuguese),
        Lang::Ita => Some(Algorithm::Italian),
        Lang::Nld => Some(Algorithm::Dutch),
        Lang::Swe => Some(Algorithm::Swedish),
        Lang::Nob => Some(Algorithm::Norwegian),
        Lang::Dan => Some(Algorithm::Danish),
        Lang::Fin => Some(Algorithm::Finnish),
        Lang::Hun => Some(Algorithm::Hungarian),
        Lang::Ron => Some(Algorithm::Romanian),
        Lang::Rus => Some(Algorithm::Russian),
        Lang::Tur => Some(Algorithm::Turkish),
        Lang::Ara => Some(Algorithm::Arabic),
        Lang::Tam => Some(Algorithm::Tamil),
        Lang::Ell => Some(Algorithm::Greek),
        _ => None, // Vietnamese, CJK, etc. → no stemming
    }
}

/// Map whatlang Lang to stop-words language code (ISO 639-1)
fn lang_to_stop_words_code(lang: Lang) -> Option<&'static str> {
    match lang {
        Lang::Eng => Some("en"),
        Lang::Fra => Some("fr"),
        Lang::Deu => Some("de"),
        Lang::Spa => Some("es"),
        Lang::Por => Some("pt"),
        Lang::Ita => Some("it"),
        Lang::Nld => Some("nl"),
        Lang::Swe => Some("sv"),
        Lang::Nob => Some("no"),
        Lang::Dan => Some("da"),
        Lang::Fin => Some("fi"),
        Lang::Hun => Some("hu"),
        Lang::Ron => Some("ro"),
        Lang::Rus => Some("ru"),
        Lang::Tur => Some("tr"),
        Lang::Ara => Some("ar"),
        Lang::Ell => Some("el"),
        Lang::Vie => Some("vi"),
        Lang::Cmn => Some("zh"),
        Lang::Jpn => Some("ja"),
        Lang::Kor => Some("ko"),
        Lang::Tha => Some("th"),
        Lang::Ind => Some("id"),
        Lang::Hin => Some("hi"),
        Lang::Pol => Some("pl"),
        Lang::Ukr => Some("uk"),
        Lang::Ces => Some("cs"),
        Lang::Heb => Some("he"),
        _ => None,
    }
}

/// Get the display name of a detected language
pub fn lang_display_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Eng => "English",
        Lang::Fra => "French",
        Lang::Deu => "German",
        Lang::Spa => "Spanish",
        Lang::Por => "Portuguese",
        Lang::Ita => "Italian",
        Lang::Nld => "Dutch",
        Lang::Swe => "Swedish",
        Lang::Nob => "Norwegian",
        Lang::Dan => "Danish",
        Lang::Fin => "Finnish",
        Lang::Hun => "Hungarian",
        Lang::Ron => "Romanian",
        Lang::Rus => "Russian",
        Lang::Tur => "Turkish",
        Lang::Ara => "Arabic",
        Lang::Ell => "Greek",
        Lang::Vie => "Vietnamese",
        Lang::Cmn => "Chinese",
        Lang::Jpn => "Japanese",
        Lang::Kor => "Korean",
        Lang::Tha => "Thai",
        Lang::Ind => "Indonesian",
        Lang::Hin => "Hindi",
        Lang::Pol => "Polish",
        Lang::Ukr => "Ukrainian",
        Lang::Ces => "Czech",
        Lang::Heb => "Hebrew",
        _ => "Unknown",
    }
}

/// Detect the dominant language from a collection of documents.
/// Concatenates samples and uses whatlang for detection.
/// Returns the detected Lang or defaults to English.
pub fn detect_language(documents: &[String]) -> Lang {
    // Take a sample from each document (first 500 chars) to build a corpus sample
    let sample: String = documents
        .iter()
        .map(|doc| {
            let chars: Vec<char> = doc.chars().collect();
            let end = chars.len().min(500);
            chars[..end].iter().collect::<String>()
        })
        .collect::<Vec<String>>()
        .join(" ");

    detect(&sample)
        .map(|info| info.lang())
        .unwrap_or(Lang::Eng)
}

/// Load stop words for a given language, returns empty set if language not supported
fn get_stop_words(lang: Lang) -> HashSet<String> {
    if let Some(code) = lang_to_stop_words_code(lang) {
        let words: Vec<String> = stop_words::get(code);
        if !words.is_empty() {
            return words.into_iter().map(|w| w.to_lowercase()).collect();
        }
    }
    // Fallback: empty set (no stop words removed)
    HashSet::new()
}

/// Tokenize text into lowercase words, filtering non-alphabetic tokens
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Generate unigrams + bigrams from tokens, removing stop words and applying stemming
fn generate_terms(
    tokens: &[String],
    stop_words: &HashSet<String>,
    stemmer: Option<&Stemmer>,
) -> Vec<String> {
    let mut terms = Vec::new();

    // Stem all tokens (or use as-is if no stemmer)
    let stemmed: Vec<String> = tokens
        .iter()
        .map(|t| match stemmer {
            Some(s) => s.stem(t).to_string(),
            None => t.clone(),
        })
        .collect();

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

/// Compute TF-IDF vectors for a collection of documents with auto language detection
pub fn compute_tfidf_vectors(documents: &[String]) -> (Vec<SparseVec>, Vec<String>, Lang) {
    let lang = detect_language(documents);
    let stop_words = get_stop_words(lang);

    let stemmer = lang_to_stemmer_algorithm(lang).map(Stemmer::create);
    let stemmer_ref = stemmer.as_ref();

    let n_docs = documents.len();

    // Step 1: Tokenize and generate terms for each document
    let doc_terms: Vec<Vec<String>> = documents
        .iter()
        .map(|doc| {
            let tokens = tokenize(doc);
            generate_terms(&tokens, &stop_words, stemmer_ref)
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

    (vectors, vocab_names, lang)
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
        let (vectors, _, _) = compute_tfidf_vectors(&docs);
        let sim = cosine_similarity(&vectors[0], &vectors[1]);
        assert!((sim - 1.0).abs() < 0.001, "Identical docs should have similarity ~1.0, got {sim}");
    }

    #[test]
    fn test_different_documents() {
        let docs = vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "Mathematics and physics are fundamental sciences".to_string(),
        ];
        let (vectors, _, _) = compute_tfidf_vectors(&docs);
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
        let (vectors, _, _) = compute_tfidf_vectors(&docs);
        let results = compute_pairwise_similarities(&vectors, 0.5);
        // First two docs should be similar, third should be different
        assert!(!results.is_empty(), "Should find at least one similar pair");
        assert_eq!(results[0].0, 0);
        assert_eq!(results[0].1, 1);
    }

    #[test]
    fn test_detect_english() {
        let docs = vec![
            "The quick brown fox jumps over the lazy dog and runs away into the forest".to_string(),
        ];
        let lang = detect_language(&docs);
        assert_eq!(lang, Lang::Eng);
    }

    #[test]
    fn test_detect_vietnamese() {
        let docs = vec![
            "Hôm nay thời tiết rất đẹp, tôi muốn đi dạo trong công viên và ngắm cảnh thiên nhiên tuyệt vời".to_string(),
        ];
        let lang = detect_language(&docs);
        assert_eq!(lang, Lang::Vie);
    }

    #[test]
    fn test_vietnamese_documents_similarity() {
        let docs = vec![
            "Hút thuốc lá nên bị cấm hoàn toàn trong tất cả các nhà hàng vì nó gây hại cho sức khỏe của mọi người xung quanh".to_string(),
            "Hút thuốc lá cần phải bị cấm tại tất cả các nhà hàng bởi vì nó ảnh hưởng xấu đến sức khỏe của những người xung quanh".to_string(),
        ];
        let (vectors, _, lang) = compute_tfidf_vectors(&docs);
        assert_eq!(lang, Lang::Vie);
        let sim = cosine_similarity(&vectors[0], &vectors[1]);
        assert!(sim > 0.3, "Similar Vietnamese docs should have reasonable similarity, got {sim}");
    }
}
