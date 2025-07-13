use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// TermFreq is the term to frequency table for each file.
pub type TermFreq = HashMap<String, usize>;

/// TermFreqPerDoc is the TermFreq for each Doc in the directory
/// each directory caontains multiple files that are each a PathBuf
pub type TermFreqPerDoc = HashMap<PathBuf, TermFreq>;

/// DocFreq is the document frequency index, which maps terms to the number of documents they appear in.
/// This is useful for calculating the inverse document frequency (IDF) of terms.
/// It is a HashMap where the key is a term (String) and the value is the number of documents (usize) that contain that term.
/// This is used in the search algorithm to determine the importance of a term across the entire index
pub type DocFreq = HashMap<String, usize>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Model {
    pub tfpd: TermFreqPerDoc,
    pub df: DocFreq,
}


pub struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        // This function trims the left side of the content until a non-whitespace character is found
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.trim_left();
        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(
                self.chop_while(|x| x.is_numeric())
                    .iter()
                    .collect::<String>(),
            );
        }

        if self.content[0].is_alphabetic() {
            return Some(
                self.chop_while(|x| x.is_alphanumeric())
                    .iter()
                    .map(|x| x.to_ascii_uppercase())
                    .collect::<String>(),
            );
        }

        Some(self.chop(1).iter().collect::<String>())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Returns the total frequency of the term `t` in the document frequency index `d`.
/// It sums up the term frequencies across all documents in the index.
/// If the term is not found in a document, it contributes 0 to the sum.
fn tf(t: &str, d: &TermFreq) -> f32 {
    d.get(t).cloned().unwrap_or(0) as f32 / d.iter().map(|(_, v)| *v).sum::<usize>() as f32
}

/// Returns the inverse document frequency (IDF) of the term `t` in the document frequency index `d`.
/// It calculates the logarithm of the ratio of the total number of documents to the number of documents containing the term.
/// If the term is not found in any document, it returns 0.
/// The IDF is a measure of how important a term is in the context of the entire document collection.
fn idf(t: &str, d: &TermFreqPerDoc) -> f32 {
    let n: f32 = d.len() as f32;
    let m: f32 = 1f32 + d.values().filter(|tf| tf.contains_key(t)).count().max(1) as f32;
    (n / m).log10()
}

pub fn search_query<'a>(model: &'a Model, query: &'a [char]) -> Vec<(&'a Path, f32)> {
    let mut results: Vec<(&Path, f32)> = Vec::new();
    for (path, tf_table) in &model.tfpd {
        let mut rank = 0.0;
        let tokens = Lexer::new(query).collect::<Vec<_>>();
        for token in tokens {
            rank += tf(&token, tf_table) * idf(&token, &model.tfpd);
        }
        results.push((path, rank));
    }

    results.sort_by(|(_, rank1), (_, rank2)| rank2.partial_cmp(rank1).unwrap());
    results
}
