//! A naive implementation of Cosine Similarity for
//! measuring similarity between strings

use counter::Counter;
use regex::Regex;

use std::collections::HashSet;

/// Finds the cosine similarity between two strings
///
/// # Arguments
/// text1, text2
///
/// REMOVED in favour of a uniform alphanumeric pattern
/// regex: Name of the Mathing pattern
/// The default metric used is similarity in the string's alphaneumerics ("\w").
/// To base the similarity on words("\w+") instead, specify regex=words
pub async fn cosine_similarity(text1: &str, text2: &str) -> f64 {
    let regex = "alpha";
    let pattern = if regex == "words" { r"\w+" } else { r"\w" };
    let re = Regex::new(pattern).unwrap();

    let capture = re
        .captures_iter(&text1)
        .map(|c| c.get(0).map_or("", |z| z.as_str()))
        .collect::<Vec<&str>>();
    let capture2 = re
        .captures_iter(&text2)
        .map(|c| c.get(0).map_or("", |z| z.as_str()))
        .collect::<Vec<&str>>();

    let counter = capture.iter().collect::<Counter<_>>();
    let counter2 = capture2.iter().collect::<Counter<_>>();

    let hash1 = counter.into_map();
    let hash2 = counter2.into_map();

    let keys1 = hash1.keys().map(|c| **c).collect::<Vec<&str>>();
    let keys2 = hash2.keys().map(|c| **c).collect::<Vec<&str>>();

    let set1 = keys1.iter().map(|c| *c).collect::<HashSet<&str>>();
    let set2 = keys2.iter().map(|c| *c).collect::<HashSet<&str>>();

    let intersection: Vec<&str> = set1.intersection(&set2).map(|c| *c).collect();

    let numerator = intersection
        .iter()
        .map(|key| hash1[key] * hash2[key])
        .collect::<Vec<usize>>()
        .into_iter()
        .sum::<usize>();

    let sum1 = keys1
        .iter()
        .map(|key| hash1[key].pow(2))
        .collect::<Vec<usize>>()
        .into_iter()
        .sum::<usize>();
    let sum2 = keys2
        .iter()
        .map(|key| hash2[key].pow(2))
        .collect::<Vec<usize>>()
        .into_iter()
        .sum::<usize>();

    let denominator = (sum1 as f64).sqrt() * (sum2 as f64).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        (numerator as f64) / denominator
    }
}
