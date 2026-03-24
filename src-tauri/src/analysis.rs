//! Core analysis module: multi-format file loading, MD5 duplicate detection, N-gram matching.

use dotext::*;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::sentence::{self, SuspiciousPair};
use crate::tfidf;

/// Supported file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["txt", "docx", "html", "htm"];

/// A pair of files with their similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityPair {
    pub file_a: String,
    pub file_b: String,
    pub score: f64,
}

/// Complete scan result
#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub pairs: Vec<SimilarityPair>,
    pub duplicate_groups: Vec<Vec<String>>,
    pub file_count: usize,
    pub message: String,
}

/// Detail result for side-by-side comparison
#[derive(Debug, Clone, Serialize)]
pub struct DetailResult {
    pub file_a: String,
    pub file_b: String,
    pub content_a: String,
    pub content_b: String,
    pub highlights_a: Vec<[usize; 2]>,
    pub highlights_b: Vec<[usize; 2]>,
    pub common_phrase_count: usize,
    pub suspicious_sentences: Vec<SuspiciousPair>,
}

/// Check if a file has a supported extension
fn is_supported_file(fname: &str) -> bool {
    let lower = fname.to_lowercase();
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Extract text from a template file path (public for commands.rs)
pub fn extract_template_text(path: &str) -> Option<String> {
    extract_text(Path::new(path))
}

/// Extract text content from a file based on its extension
fn extract_text(path: &Path) -> Option<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" => fs::read_to_string(path).ok(),
        "docx" => {
            let mut docx = Docx::open(path).ok()?;
            let mut content = String::new();
            docx.read_to_string(&mut content).ok()?;
            Some(content)
        }
        "html" | "htm" => {
            let html_content = fs::read_to_string(path).ok()?;
            let document = scraper::Html::parse_document(&html_content);
            let text: String = document
                .root_element()
                .text()
                .collect::<Vec<_>>()
                .join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        }
        _ => None,
    }
}

/// Load all supported files from a folder, returns map of filename -> content
pub fn load_files(folder: &str) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let path = Path::new(folder);

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if is_supported_file(&fname) {
                if let Some(content) = extract_text(&entry.path()) {
                    if !content.trim().is_empty() {
                        files.insert(fname, content);
                    }
                }
            }
        }
    }

    files
}

/// Count supported files in a folder
pub fn count_supported_files(folder: &str) -> usize {
    let path = Path::new(folder);
    if let Ok(entries) = fs::read_dir(path) {
        entries
            .flatten()
            .filter(|e| is_supported_file(&e.file_name().to_string_lossy()))
            .count()
    } else {
        0
    }
}

/// Normalize text for MD5 comparison (lowercase, collapse whitespace, strip punctuation)
fn normalize_for_hash(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Find groups of files with identical normalized MD5 hashes
pub fn find_exact_duplicates(files: &HashMap<String, String>) -> Vec<Vec<String>> {
    let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();

    for (fname, content) in files {
        let normalized = normalize_for_hash(content);
        let hash = format!("{:x}", Md5::digest(normalized.as_bytes()));
        hash_map.entry(hash).or_default().push(fname.clone());
    }

    hash_map
        .into_values()
        .filter(|group| group.len() >= 2)
        .collect()
}

/// Run the full similarity scan on a folder
pub fn scan_folder(folder: &str, threshold: f64, template_text: Option<&str>) -> ScanResult {
    let mut files = load_files(folder);
    let n = files.len();

    if n < 2 {
        return ScanResult {
            pairs: vec![],
            duplicate_groups: vec![],
            file_count: n,
            message: format!("Need at least 2 supported files to compare. Found {n}."),
        };
    }

    // Strip template content if provided
    if let Some(template) = template_text {
        for content in files.values_mut() {
            *content = sentence::strip_template(content, template);
        }
    }

    // Exact duplicates
    let duplicate_groups = find_exact_duplicates(&files);

    // Sort filenames for consistent ordering
    let mut filenames: Vec<String> = files.keys().cloned().collect();
    filenames.sort();
    let contents: Vec<String> = filenames.iter().map(|f| files[f].clone()).collect();

    // TF-IDF + Cosine Similarity
    let (vectors, _) = tfidf::compute_tfidf_vectors(&contents);
    let raw_pairs = tfidf::compute_pairwise_similarities(&vectors, threshold);

    let pairs: Vec<SimilarityPair> = raw_pairs
        .into_iter()
        .map(|(i, j, score)| SimilarityPair {
            file_a: filenames[i].clone(),
            file_b: filenames[j].clone(),
            score,
        })
        .collect();

    let dupe_count: usize = duplicate_groups.iter().map(|g| g.len()).sum();
    let mut message = format!(
        "Found {} pairs with cosine score ≥ {:.0} among {} files.",
        pairs.len(),
        threshold * 100.0,
        n
    );
    if !duplicate_groups.is_empty() {
        message.push_str(&format!(
            " ({} files are exact duplicates in {} groups)",
            dupe_count,
            duplicate_groups.len()
        ));
    }

    ScanResult {
        pairs,
        duplicate_groups,
        file_count: n,
        message,
    }
}

/// Extract word-level N-grams from text
fn get_ngrams(text: &str, n: usize) -> HashSet<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < n {
        return HashSet::new();
    }

    words
        .windows(n)
        .map(|window| {
            window
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// Find common N-gram phrases between two texts
pub fn find_common_phrases(text_a: &str, text_b: &str, n: usize) -> HashSet<String> {
    let ngrams_a = get_ngrams(text_a, n);
    let ngrams_b = get_ngrams(text_b, n);
    ngrams_a.intersection(&ngrams_b).cloned().collect()
}

/// Find character ranges in text that match any common phrase, merged to avoid overlaps
pub fn get_highlight_ranges(text: &str, common_phrases: &HashSet<String>) -> Vec<[usize; 2]> {
    let text_lower = text.to_lowercase();
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for phrase in common_phrases {
        let mut start = 0;
        while let Some(idx) = text_lower[start..].find(phrase.as_str()) {
            let abs_idx = start + idx;
            ranges.push((abs_idx, abs_idx + phrase.len()));
            start = abs_idx + 1;
        }
    }

    if ranges.is_empty() {
        return vec![];
    }

    // Merge overlapping ranges
    ranges.sort();
    let mut merged: Vec<(usize, usize)> = vec![ranges[0]];
    for &(s, e) in &ranges[1..] {
        let last = merged.last_mut().unwrap();
        if s <= last.1 {
            last.1 = last.1.max(e);
        } else {
            merged.push((s, e));
        }
    }

    merged.iter().map(|&(s, e)| [s, e]).collect()
}

/// Get detail comparison with N-gram highlights + sentence-level suspicious pairs
pub fn get_detail(
    folder: &str,
    file_a: &str,
    file_b: &str,
    ngram_size: usize,
    sentence_threshold: f64,
) -> DetailResult {
    let files = load_files(folder);
    let content_a = files.get(file_a).cloned().unwrap_or_default();
    let content_b = files.get(file_b).cloned().unwrap_or_default();

    let common = find_common_phrases(&content_a, &content_b, ngram_size);
    let highlights_a = get_highlight_ranges(&content_a, &common);
    let highlights_b = get_highlight_ranges(&content_b, &common);

    // Sentence-level analysis
    let suspicious_sentences =
        sentence::find_suspicious_sentences(&content_a, &content_b, sentence_threshold);

    DetailResult {
        file_a: file_a.to_string(),
        file_b: file_b.to_string(),
        content_a,
        content_b,
        highlights_a,
        highlights_b,
        common_phrase_count: common.len(),
        suspicious_sentences,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        assert!(is_supported_file("test.txt"));
        assert!(is_supported_file("essay.docx"));
        assert!(is_supported_file("page.html"));
        assert!(!is_supported_file("image.png"));
        assert!(!is_supported_file("paper.pdf"));
    }

    #[test]
    fn test_ngrams() {
        let ngrams = get_ngrams("the quick brown fox jumps", 3);
        assert!(ngrams.contains("the quick brown"));
        assert!(ngrams.contains("quick brown fox"));
        assert!(ngrams.contains("brown fox jumps"));
        assert_eq!(ngrams.len(), 3);
    }

    #[test]
    fn test_common_phrases() {
        let a = "smoking should be completely banned at all restaurants";
        let b = "smoking should be completely banned in every restaurant";
        let common = find_common_phrases(a, b, 5);
        assert!(common.contains("smoking should be completely banned"));
    }

    #[test]
    fn test_highlight_ranges_merge() {
        let text = "hello world hello world";
        let mut phrases = HashSet::new();
        phrases.insert("hello world".to_string());
        let ranges = get_highlight_ranges(text, &phrases);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], [0, 11]);
        assert_eq!(ranges[1], [12, 23]);
    }

    #[test]
    fn test_md5_duplicates_normalized() {
        let mut files = HashMap::new();
        files.insert("a.txt".to_string(), "Same Content!".to_string());
        files.insert("b.txt".to_string(), "same  content".to_string());
        files.insert("c.txt".to_string(), "different content".to_string());

        let dupes = find_exact_duplicates(&files);
        assert_eq!(dupes.len(), 1, "Normalized MD5 should catch formatting-only diffs");
        assert_eq!(dupes[0].len(), 2);
    }

    #[test]
    fn test_normalize_for_hash() {
        assert_eq!(normalize_for_hash("Hello, World!"), "hello world");
        assert_eq!(normalize_for_hash("  spaces   everywhere  "), "spaces everywhere");
    }
}
