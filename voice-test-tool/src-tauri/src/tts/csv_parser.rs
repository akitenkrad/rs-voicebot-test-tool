use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvError {
    #[error("Failed to open file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    ParseError(#[from] csv::Error),
    #[error("Empty CSV file")]
    EmptyFile,
}

/// A single test case loaded from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsTestCase {
    /// Unique identifier for this test case (e.g., "001", "002")
    pub id: String,
    /// The text to be converted to speech
    pub text: String,
}

/// Parse a CSV file containing TTS test cases.
///
/// The CSV is expected to have a single column with text entries.
/// Each non-empty line becomes a test case with an auto-generated ID.
///
/// # Arguments
///
/// * `path` - Path to the CSV file
///
/// # Returns
///
/// A vector of `TtsTestCase` items, or an error if parsing fails.
pub fn parse_csv(path: &Path) -> Result<Vec<TtsTestCase>, CsvError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut test_cases = Vec::new();
    for (index, result) in reader.records().enumerate() {
        let record = result?;
        if let Some(text) = record.get(0) {
            let text = text.trim();
            if !text.is_empty() {
                test_cases.push(TtsTestCase {
                    id: format!("{:03}", index + 1),
                    text: text.to_string(),
                });
            }
        }
    }

    if test_cases.is_empty() {
        return Err(CsvError::EmptyFile);
    }

    Ok(test_cases)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv_basic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file, "This is a test.").unwrap();
        writeln!(file, "Another line.").unwrap();

        let cases = parse_csv(file.path()).unwrap();
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].id, "001");
        assert_eq!(cases[0].text, "Hello, world!");
        assert_eq!(cases[1].id, "002");
        assert_eq!(cases[1].text, "This is a test.");
        assert_eq!(cases[2].id, "003");
        assert_eq!(cases[2].text, "Another line.");
    }

    #[test]
    fn test_parse_csv_skips_empty_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Line one").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "   ").unwrap();
        writeln!(file, "Line two").unwrap();

        let cases = parse_csv(file.path()).unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].text, "Line one");
        assert_eq!(cases[1].text, "Line two");
    }

    #[test]
    fn test_parse_csv_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let result = parse_csv(file.path());
        assert!(matches!(result, Err(CsvError::EmptyFile)));
    }

    #[test]
    fn test_parse_csv_trims_whitespace() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "  trimmed text  ").unwrap();

        let cases = parse_csv(file.path()).unwrap();
        assert_eq!(cases[0].text, "trimmed text");
    }
}
