//! A register file read against its exact header, and typed access to its cells.

use crate::csv::{self, Record};
use crate::error::{DataError, Result};

/// A CSV file whose header equals the expected header exactly and whose rows are all rectangular.
#[derive(Debug, Clone)]
pub struct Table {
    file: &'static str,
    header: Vec<String>,
    records: Vec<Record>,
}

impl Table {
    /// Parse `text`. The header must be exactly `expected` (same names, same order): a missing,
    /// extra, renamed or reordered column is an error, so no column can be lost unnoticed.
    pub fn parse(file: &'static str, text: &str, expected: &[&str]) -> Result<Table> {
        let mut records = csv::parse(file, text)?;
        if records.is_empty() {
            return Err(DataError::file(file, "the file is empty"));
        }
        let head = records.remove(0);
        if head.cells.len() != expected.len()
            || head.cells.iter().zip(expected).any(|(a, b)| a != b)
        {
            let missing: Vec<&str> = expected
                .iter()
                .copied()
                .filter(|e| !head.cells.iter().any(|h| h == e))
                .collect();
            let extra: Vec<&str> = head
                .cells
                .iter()
                .map(String::as_str)
                .filter(|h| !expected.contains(h))
                .collect();
            return Err(DataError::line(
                file,
                1,
                format!(
                    "the header differs from the expected {} columns (missing {missing:?}, unexpected {extra:?}, or a different order)",
                    expected.len()
                ),
            ));
        }
        for r in &records {
            if r.cells.len() != expected.len() {
                return Err(DataError::line(
                    file,
                    r.line,
                    format!(
                        "the row has {} cells, the header has {}",
                        r.cells.len(),
                        expected.len()
                    ),
                ));
            }
        }
        Ok(Table {
            file,
            header: head.cells,
            records,
        })
    }

    /// File name used in error messages.
    pub fn file(&self) -> &'static str {
        self.file
    }

    /// Number of data rows.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True when there is no data row.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The rows in file order.
    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.records.iter().map(move |rec| Row { table: self, rec })
    }
}

/// One data row with named, checked access to its cells.
#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    table: &'a Table,
    rec: &'a Record,
}

impl<'a> Row<'a> {
    /// Line of the file.
    pub fn line(&self) -> usize {
        self.rec.line
    }

    /// An error tied to this row (and, when given, a column).
    pub fn err(&self, column: Option<&str>, message: impl Into<String>) -> DataError {
        DataError {
            file: self.table.file.to_string(),
            line: self.rec.line,
            id: self.rec.cells.first().cloned(),
            column: column.map(str::to_string),
            message: message.into(),
        }
    }

    /// The cell text exactly as in the file.
    pub fn text(&self, column: &str) -> Result<&'a str> {
        match self.table.header.iter().position(|h| h == column) {
            Some(i) => Ok(self.rec.cells[i].as_str()),
            None => Err(self.err(
                Some(column),
                "the reader asks for a column that is not in the header",
            )),
        }
    }

    /// The cell text; an empty cell is an error.
    pub fn required(&self, column: &str) -> Result<&'a str> {
        let t = self.text(column)?;
        if t.is_empty() {
            return Err(self.err(
                Some(column),
                "the cell is empty but the reader requires a value",
            ));
        }
        Ok(t)
    }

    /// The cell text, `None` when the cell is empty.
    pub fn optional(&self, column: &str) -> Result<Option<&'a str>> {
        let t = self.text(column)?;
        Ok(if t.is_empty() { None } else { Some(t) })
    }

    /// A required number that keeps its printed text.
    pub fn number(&self, column: &str) -> Result<Printed> {
        let t = self.required(column)?;
        Printed::parse(t).map_err(|m| self.err(Some(column), m))
    }

    /// An optional number that keeps its printed text; a non-empty cell that is not a number is an error.
    pub fn opt_number(&self, column: &str) -> Result<Option<Printed>> {
        match self.optional(column)? {
            None => Ok(None),
            Some(t) => Printed::parse(t)
                .map(Some)
                .map_err(|m| self.err(Some(column), m)),
        }
    }

    /// A required unsigned integer.
    pub fn uint(&self, column: &str) -> Result<u32> {
        let t = self.required(column)?;
        t.parse::<u32>()
            .map_err(|_| self.err(Some(column), format!("`{t}` is not an unsigned integer")))
    }

    /// A required signed integer.
    pub fn int(&self, column: &str) -> Result<i32> {
        let t = self.required(column)?;
        t.parse::<i32>()
            .map_err(|_| self.err(Some(column), format!("`{t}` is not an integer")))
    }

    /// A required cell of a closed vocabulary; any other text is an error.
    pub fn vocab<T: Vocabulary>(&self, column: &str) -> Result<T> {
        let t = self.required(column)?;
        T::from_cell(t).ok_or_else(|| {
            self.err(
                Some(column),
                format!(
                    "`{t}` is not in the vocabulary of {}: {:?}",
                    T::NAME,
                    T::ALL
                ),
            )
        })
    }
}

/// A closed vocabulary of cell texts. Implemented by [`string_enum!`](crate::string_enum).
pub trait Vocabulary: Sized + Copy + 'static {
    /// Human name of the vocabulary, for error messages.
    const NAME: &'static str;
    /// Every accepted text.
    const ALL: &'static [&'static str];
    /// The variant for a cell text, if it is in the vocabulary.
    fn from_cell(text: &str) -> Option<Self>;
    /// The cell text of a variant.
    fn as_str(&self) -> &'static str;
}

/// Define an enum whose variants are exactly the listed cell texts.
#[macro_export]
macro_rules! string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident ($label:literal) { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name { $($(#[$vmeta])* $variant),+ }

        impl $crate::table::Vocabulary for $name {
            const NAME: &'static str = $label;
            const ALL: &'static [&'static str] = &[$($text),+];
            fn from_cell(text: &str) -> Option<Self> {
                match text { $($text => Some($name::$variant),)+ _ => None }
            }
            fn as_str(&self) -> &'static str {
                match self { $($name::$variant => $text),+ }
            }
        }
    };
}

/// A finite number together with the text it was printed as.
///
/// The text is kept so that the register's own digits can be shown and compared exactly; the
/// value is what the arithmetic uses.
#[derive(Debug, Clone, PartialEq)]
pub struct Printed {
    /// The number.
    pub value: f64,
    /// The cell text.
    pub text: String,
}

impl Printed {
    /// Parse a plain decimal or scientific number. Words such as `nan` or `inf` and any
    /// surrounding blank are rejected.
    pub fn parse(text: &str) -> std::result::Result<Printed, String> {
        let ok_chars = !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'));
        if !ok_chars {
            return Err(format!("`{text}` is not a plain number"));
        }
        match text.parse::<f64>() {
            Ok(v) if v.is_finite() => Ok(Printed {
                value: v,
                text: text.to_string(),
            }),
            _ => Err(format!("`{text}` is not a finite number")),
        }
    }
}

/// The finite number a cell holds or `None` for a blank or non-numeric cell, mirroring `fnum`
/// of the Python audit script (used only where a check needs that exact behaviour).
pub fn fnum(text: &str) -> Option<f64> {
    Printed::parse(text).ok().map(|p| p.value)
}
