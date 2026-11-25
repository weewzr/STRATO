//! The one error type of the data layer.
//!
//! Every failure names the file, the line of the CSV file (line 1 is the header), the row
//! identifier when the row has one, and the column. A reader never skips a row and never
//! replaces a cell it cannot read by a default value: it returns this error instead.

use std::fmt;

/// A problem found while reading or validating a register file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError {
    /// Register file, relative to the `data` folder (for example `geometry/geometry.csv`).
    pub file: String,
    /// Line of the CSV file; 0 when the problem is not tied to a line.
    pub line: usize,
    /// Row identifier, when the row has one.
    pub id: Option<String>,
    /// Column name, when the problem is tied to one column.
    pub column: Option<String>,
    /// What is wrong, in words.
    pub message: String,
}

impl DataError {
    /// An error tied to a whole file.
    pub fn file(file: &str, message: impl Into<String>) -> Self {
        DataError {
            file: file.to_string(),
            line: 0,
            id: None,
            column: None,
            message: message.into(),
        }
    }

    /// An error tied to a line of a file.
    pub fn line(file: &str, line: usize, message: impl Into<String>) -> Self {
        DataError {
            file: file.to_string(),
            line,
            id: None,
            column: None,
            message: message.into(),
        }
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        if self.line > 0 {
            write!(f, ":{}", self.line)?;
        }
        if let Some(id) = &self.id {
            write!(f, " [{id}]")?;
        }
        if let Some(c) = &self.column {
            write!(f, " column `{c}`")?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for DataError {}

/// Result type of the data layer.
pub type Result<T> = std::result::Result<T, DataError>;
