use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::SystemTime;

use super::lexer::Lexer;

pub trait Model {
    fn search_query(&self, query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()>;
    fn requires_reindexing(&mut self, file_path: &Path, last_modified: SystemTime) -> bool;
    fn add_document(
        &mut self,
        path: PathBuf,
        last_modified: SystemTime,
        content: &[char],
    ) -> Result<(), ()>;
}

pub struct SqliteModel {
    pub connection: sqlite::Connection,
}

impl SqliteModel {
    fn execute(&self, statement: &str) -> Result<(), ()> {
        self.connection.execute(statement).map_err(|err| {
            eprintln!("ERROR: could not execute statement: {statement}: {err}");
        })?;
        Ok(())
    }

    pub fn begin(&self) -> Result<(), ()> {
        self.execute("BEGIN;")
    }

    pub fn commit(&self) -> Result<(), ()> {
        self.execute("COMMIT;")
    }

    pub fn open(path: &Path) -> Result<Self, ()> {
        let connection = sqlite::open(path).map_err(|err| {
            eprintln!(
                "ERROR: could not open sqlite database {path}: {err}",
                path = path.display()
            );
        })?;
        let this = Self { connection };

        this.execute(
            "
            CREATE TABLE IF NOT EXISTS Documents (
                id INTEGER NOT NULL PRIMARY KEY,
                path TEXT,
                term_count INTEGER,
                UNIQUE(path)
            );
        ",
        )?;

        this.execute(
            "
            CREATE TABLE IF NOT EXISTS TermFreq (
                term TEXT,
                doc_id INTEGER,
                freq INTEGER,
                UNIQUE(term, doc_id),
                FOREIGN KEY(doc_id) REFERENCES Documents(id)
            );
       ",
        )?;

        this.execute(
            "
            CREATE TABLE IF NOT EXISTS DocFreq (
                term TEXT,
                freq INTEGER,
                UNIQUE(term)
            );
        ",
        )?;

        Ok(this)
    }
}

fn log_and_ignore(err: impl std::error::Error) {
    eprintln!("ERROR: {err}");
}

impl Model for SqliteModel {
    fn search_query(&self, _query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()> {
        todo!("Implement search_query for SqliteModel");
    }

    fn requires_reindexing(&mut self, _file_path: &Path, _last_modified: SystemTime) -> bool {
        // TODO: Implement this.
        return true;
    }

    fn add_document(
        &mut self,
        path: PathBuf,
        _last_modified: SystemTime,
        content: &[char],
    ) -> Result<(), ()> {
        let terms = Lexer::new(content).collect::<Vec<_>>();
        let doc_id = {
            let query = "INSERT INTO Documents (path, term_count) VALUES (:path, :term_count)";
            let log_err = |err| {
                eprintln!("ERROR: could not execute the query {query}: {err}");
            };
            let mut stmt = self.connection.prepare(query).map_err(log_err)?;
            stmt.bind_iter::<_, (_, sqlite::Value)>([
                (":path", path.display().to_string().as_str().into()),
                (":term_count", (terms.len() as i64).into()),
            ])
            .map_err(log_err)?;
            stmt.next().map_err(log_and_ignore)?;
            unsafe { sqlite3_sys::sqlite3_last_insert_rowid(self.connection.as_raw()) }
        };

        let mut tf = TermFreq::new();
        for term in Lexer::new(content) {
            *tf.entry(term).or_insert(0) += 1;
        }

        for (term, freq) in &tf {
            // TermFreq table
            {
                let query = "SELECT freq FROM TermFreq WHERe doc_id = :doc_id AND term = :term";
                let log_err = |err| {
                    eprintln!("ERROR: Could not execute query {query}: {err}");
                };
                let mut stmt = self.connection.prepare(query).map_err(log_err)?;
                stmt.bind_iter::<_, (_, sqlite::Value)>([
                    (":doc_id", doc_id.into()),
                    (":term", term.as_str().into()),
                    (":freq", (*freq as i64).into()),
                ])
                .map_err(log_err)?;
                stmt.next().map_err(log_and_ignore)?;
            }

            // DocFreq table

            {
                let freq = {
                    let query = "SELECT freq FROM DocFreq WHERE term = :term";
                    let log_err = |err| {
                        eprintln!("ERROR: Could not execute query {query}: {err}");
                    };
                    let mut stmt = self.connection.prepare(query).map_err(log_err)?;
                    stmt.bind_iter::<_, (_, sqlite::Value)>([(":term", term.as_str().into())])
                        .map_err(log_err)?;
                    match stmt.next().map_err(log_err)? {
                        sqlite::State::Row => stmt.read::<i64, _>("freq").map_err(log_err)?,
                        sqlite::State::Done => 0,
                    }
                };

                // TODO: find a better way to auto increment the frequency
                let query = "INSERT OR REPLACE INTO DocFreq(term, freq) VALUES (:term, :freq)";
                let log_err = |err| {
                    eprintln!("ERROR: Could not execute query {query}: {err}");
                };
                let mut stmt = self.connection.prepare(query).map_err(log_err)?;
                stmt.bind_iter::<_, (_, sqlite::Value)>([
                    (":term", term.as_str().into()),
                    (":freq", (freq + 1).into()), // TODO: increment the frequency -> bottleneck.
                ])
                .map_err(log_err)?;
                stmt.next().map_err(log_err)?;
            }
        }

        for term in &terms {
            let freq = {
                let query = "SELECT freq FROM TermFreq WHERE doc_id = :doc_id AND term = :term";
                let log_err = |err| {
                    eprintln!("ERROR: Could not execute query {query}: {err}");
                };
                let mut stmt = self.connection.prepare(query).map_err(log_err)?;
                stmt.bind_iter::<_, (_, sqlite::Value)>([
                    (":doc_id", doc_id.into()),
                    (":term", term.as_str().into()),
                ])
                .map_err(log_err)?;
                match stmt.next().map_err(log_err)? {
                    sqlite::State::Row => stmt.read::<i64, _>("freq").map_err(log_err)?,
                    sqlite::State::Done => 0,
                }
            };

            // TODO: find a better way to auto increment the frequency
            let query = "INSERT OR REPLACE INTO TermFreq(doc_id, term, freq) VALUES (:doc_id, :term, :freq)";
            let log_err = |err| {
                eprintln!("ERROR: Could not execute query {query}: {err}");
            };
            let mut stmt = self.connection.prepare(query).map_err(log_err)?;
            stmt.bind_iter::<_, (_, sqlite::Value)>([
                (":doc_id", doc_id.into()),
                (":term", term.as_str().into()),
                (":freq", (freq + 1).into()),
            ])
            .map_err(log_err)?;
            stmt.next().map_err(log_err)?;
        }

        Ok(())
    }
}

/// TermFreq is the term to frequency table for each file.
pub type TermFreq = HashMap<String, usize>;

/// TermFreqPerDoc is the TermFreq for each Doc in the directory
/// each directory caontains multiple files that are each a PathBuf
pub type TermFreqPerDoc = HashMap<PathBuf, (usize, TermFreq)>;

/// DocFreq is the document frequency index, which maps terms to the number of documents they appear in.
/// This is useful for calculating the inverse document frequency (IDF) of terms.
/// It is a HashMap where the key is a term (String) and the value is the number of documents (usize) that contain that term.
/// This is used in the search algorithm to determine the importance of a term across the entire index
pub type DocFreq = HashMap<String, usize>;

#[derive(Debug, Deserialize, Serialize)]
struct Doc {
    tf: TermFreq,
    count: usize,
    last_modified: SystemTime,
}
type Docs = HashMap<PathBuf, Doc>;
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InMemoryModel {
    docs: Docs,
    df: DocFreq,
}

impl InMemoryModel {
    fn remove_document(&mut self, file_path: &Path) {
        if let Some(doc) = self.docs.remove(file_path) {
            for term in doc.tf.keys() {
                if let Some(freq) = self.df.get_mut(term) {
                    *freq -= 1;
                }
            }
        }
    }
}

impl Model for InMemoryModel {
    fn search_query(&self, query: &[char]) -> Result<Vec<(PathBuf, f32)>, ()> {
        let mut result = Vec::new();
        let tokens = Lexer::new(query).collect::<Vec<_>>();
        for (path, doc) in &self.docs {
            let mut rank = 0f32;
            for token in &tokens {
                rank += compute_tf(token, doc) * compute_idf(token, self.docs.len(), &self.df);
            }
            result.push((path.clone(), rank));
        }
        result.sort_by(|(_, rank1), (_, rank2)| rank1.partial_cmp(rank2).unwrap());
        result.reverse();
        Ok(result)
    }

    fn requires_reindexing(&mut self, file_path: &Path, last_modified: SystemTime) -> bool {
        if let Some(doc) = self.docs.get(file_path) {
            return doc.last_modified < last_modified;
        }
        return true;
    }
    fn add_document(
        &mut self,
        file_path: PathBuf,
        last_modified: SystemTime,
        content: &[char],
    ) -> Result<(), ()> {
        eprintln!(
            "⚠️ Document {} is modified, updating...",
            file_path.display()
        );
        self.remove_document(file_path.as_path());

        let mut tf = TermFreq::new();

        let mut count = 0;
        for term in Lexer::new(content) {
            *tf.entry(term).or_insert(0) += 1;
            count += 1;
        }

        for t in tf.keys() {
            *self.df.entry(t.clone()).or_insert(0) += 1;
        }

        self.docs.insert(
            file_path,
            Doc {
                count,
                tf,
                last_modified,
            },
        );
        Ok(())
    }
}
/// Returns the total frequency of the term `t` in the document frequency index `d`.
/// It sums up the term frequencies across all documents in the index.
/// If the term is not found in a document, it contributes 0 to the sum.
fn compute_tf(t: &str, doc: &Doc) -> f32 {
    let n = doc.count as f32;
    let m = doc.tf.get(t).cloned().unwrap_or(0) as f32;
    m / n
}

/// Returns the inverse document frequency (IDF) of the term `t` in the document frequency index `d`.
/// It calculates the logarithm of the ratio of the total number of documents to the number of documents containing the term.
/// If the term is not found in any document, it returns 0.
/// The IDF is a measure of how important a term is in the context of the entire document collection.
fn compute_idf(t: &str, n: usize, df: &DocFreq) -> f32 {
    let n: f32 = n as f32;
    let m = df.get(t).cloned().unwrap_or(1) as f32;
    (n / m).log10()
}
