//! A small, strict CSV reader (RFC 4180 subset) for the register files.
//!
//! Handles quoted cells with embedded commas, doubled quotes and line breaks, LF and CRLF line
//! ends, and a leading byte-order mark. It rejects an unterminated quote and text after a closing
//! quote. It does not trim, convert or default any cell: what is in the file is what comes out.

use crate::error::{DataError, Result};

/// One record: its cells and the line of the file on which it starts (the header is line 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Line on which the record starts.
    pub line: usize,
    /// The cells, as text.
    pub cells: Vec<String>,
}

/// Split `text` into records. The final line break is optional; blank lines are an error.
pub fn parse(file: &str, text: &str) -> Result<Vec<Record>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut records = Vec::new();
    let mut chars = text.chars().peekable();
    let mut line = 1usize;
    let mut rec_line = 1usize;
    let mut cells: Vec<String> = Vec::new();
    let mut cell = String::new();
    // true when the current cell started with a quote and is still open
    let mut in_quotes = false;
    // true after a quoted cell closed, until the next separator
    let mut closed = false;
    let mut at_record_start = true;

    while let Some(c) = chars.next() {
        if at_record_start {
            rec_line = line;
            at_record_start = false;
        }
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        cell.push('"');
                    } else {
                        in_quotes = false;
                        closed = true;
                    }
                }
                '\n' => {
                    line += 1;
                    cell.push('\n');
                }
                '\r' => {
                    // keep the content of a quoted cell verbatim, except that CRLF becomes LF
                    if chars.peek() != Some(&'\n') {
                        cell.push('\r');
                    }
                }
                _ => cell.push(c),
            }
            continue;
        }
        match c {
            ',' => {
                cells.push(std::mem::take(&mut cell));
                closed = false;
            }
            '\r' => {}
            '\n' => {
                cells.push(std::mem::take(&mut cell));
                closed = false;
                if cells.len() == 1 && cells[0].is_empty() {
                    return Err(DataError::line(
                        file,
                        rec_line,
                        "blank line inside the file",
                    ));
                }
                records.push(Record {
                    line: rec_line,
                    cells: std::mem::take(&mut cells),
                });
                line += 1;
                at_record_start = true;
            }
            '"' => {
                if !cell.is_empty() || closed {
                    return Err(DataError::line(file, line, "quote in the middle of a cell"));
                }
                in_quotes = true;
            }
            _ => {
                if closed {
                    return Err(DataError::line(
                        file,
                        line,
                        "text after the closing quote of a cell",
                    ));
                }
                cell.push(c);
            }
        }
    }
    if in_quotes {
        return Err(DataError::line(
            file,
            rec_line,
            "a quoted cell is not closed before the end of the file",
        ));
    }
    if !at_record_start {
        cells.push(cell);
        records.push(Record {
            line: rec_line,
            cells,
        });
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_and_quoted_cells() {
        let r = parse("t", "a,b,c\n1,\"x,y\",\"say \"\"hi\"\"\"\n").unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[1].cells, vec!["1", "x,y", "say \"hi\""]);
        assert_eq!(r[1].line, 2);
    }

    #[test]
    fn crlf_bom_and_missing_final_newline() {
        let r = parse("t", "\u{feff}a,b\r\n1,2").unwrap();
        assert_eq!(r[0].cells, vec!["a", "b"]);
        assert_eq!(r[1].cells, vec!["1", "2"]);
    }

    #[test]
    fn empty_cells_are_kept() {
        let r = parse("t", "a,b,c\n,,\n").unwrap();
        assert_eq!(r[1].cells, vec!["", "", ""]);
    }

    #[test]
    fn embedded_line_break_counts_lines() {
        let r = parse("t", "a,b\n\"x\ny\",2\n3,4\n").unwrap();
        assert_eq!(r[1].cells[0], "x\ny");
        assert_eq!(r[2].line, 4);
    }

    #[test]
    fn rejects_malformed_input() {
        assert!(parse("t", "a,b\n\"open,2\n").is_err());
        assert!(parse("t", "a,b\n\"x\"y,2\n").is_err());
        assert!(parse("t", "a,b\nx\"y,2\n").is_err());
        assert!(parse("t", "a,b\n\n1,2\n").is_err());
    }
}
