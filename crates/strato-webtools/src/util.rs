//! Shared helpers: errors, repository root, git, hashing, time, file reading and a CSV reader
//! with the behaviour of Python's `csv.reader` (the tables of the repository were read with it).

use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::pyfmt;

/// Every failure of the tool: one line of text for the person running it.
#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error(format!("JSON: {e}"))
    }
}

impl From<fancy_regex::Error> for Error {
    fn from(e: fancy_regex::Error) -> Self {
        Error(format!("regular expression: {e}"))
    }
}

/// Return early with a formatted error.
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => { return Err($crate::util::Error(format!($($arg)*))) };
}

/// A path under `root` as the forward-slash text that the data files record.
pub fn rel_posix(root: &Path, path: &Path) -> String {
    let p = path.strip_prefix(root).unwrap_or(path);
    p.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Absolute, with `.` and `..` resolved lexically and without the `\\?\` prefix of Windows.
pub fn absolute(path: &Path) -> Result<PathBuf> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    let text = out.to_string_lossy().into_owned();
    Ok(match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => out,
    })
}

/// The repository root: `--root`, else the nearest ancestor of the working directory that holds
/// the web demo and the wrapper crate.
pub fn find_root(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(r) = explicit.filter(|r| !r.is_empty()) {
        return absolute(Path::new(r));
    }
    let start = absolute(Path::new("."))?;
    let mut dir = Some(start.as_path());
    while let Some(d) = dir {
        if d.join("web").join("package.json").is_file()
            && d.join("crates").join("strato-wasm").is_dir()
        {
            return Ok(d.to_path_buf());
        }
        dir = d.parent();
    }
    bail!("the repository root was not found from the working directory; run inside the repository or pass --root")
}

/// `Path.read_text(encoding="utf-8")`: UTF-8 with universal newlines.
pub fn read_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| Error(format!("{}: not valid UTF-8", path.display())))?;
    Ok(pyfmt::universal_newlines(&text))
}

/// The file's bytes, decoded as UTF-8 without any newline translation.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| Error(format!("{}: {e}", path.display())))
}

pub fn decode_utf8(bytes: &[u8], what: &str) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| Error(format!("{what}: not valid UTF-8")))
}

/// `Path.write_text(text, encoding="utf-8", newline="\n")`, creating the parent folders.
pub fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text.as_bytes()).map_err(|e| Error(format!("{}: {e}", path.display())))
}

pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// `git <args>` in `cwd`; standard output without the trailing newline.
pub fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| Error(format!("git: {e}")))?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// HEAD of the tree at `root`, or "unknown" when it is not a Git working tree.
pub fn head_commit(root: &Path) -> String {
    if !root.join(".git").exists() {
        return "unknown".to_string();
    }
    git(root, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "unknown".to_string())
}

/// HEAD, with "+modified" appended when any of `paths` differs from HEAD, so that the commit
/// reproduces the file only when it carries no suffix.
pub fn source_commit(root: &Path, paths: &[&str]) -> String {
    let head = head_commit(root);
    if head == "unknown" {
        return head;
    }
    let mut args = vec!["status", "--porcelain", "--"];
    args.extend_from_slice(paths);
    match git(root, &args) {
        Ok(s) if s.is_empty() => head,
        Ok(_) => format!("{head}+modified"),
        Err(_) => format!("{head}+unknown-tree"),
    }
}

/// `%Y-%m-%dT%H:%M:%SZ` of the current time in UTC. `SOURCE_DATE_EPOCH` (seconds since the Unix
/// epoch, the reproducible-builds convention) replaces the clock, so a run can be repeated.
pub fn now_utc() -> String {
    let secs: i64 = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        });
    format_utc(secs)
}

pub fn format_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil date from days since 1970-01-01 (proleptic Gregorian).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Rows of `csv.reader(io.StringIO(text, newline=""))` with the default dialect (comma, double
/// quote, doubled quotes, no strict mode). An empty line gives an empty row, as in Python.
pub fn csv_rows(text: &str) -> Vec<Vec<String>> {
    #[derive(PartialEq, Clone, Copy)]
    enum S {
        StartRecord,
        StartField,
        InField,
        InQuoted,
        QuoteInQuoted,
        EatCrnl,
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut state = S::StartRecord;
    // Lines keep their terminators ("\n", "\r\n" or "\r"), as the lines of a StringIO do.
    let mut lines: Vec<&str> = Vec::new();
    let mut start = 0;
    let b = text.as_bytes();
    let mut k = 0;
    while k < b.len() {
        if b[k] == b'\n' {
            lines.push(&text[start..=k]);
            start = k + 1;
        } else if b[k] == b'\r' {
            if k + 1 < b.len() && b[k + 1] == b'\n' {
                k += 1;
            }
            lines.push(&text[start..=k]);
            start = k + 1;
        }
        k += 1;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    for line in lines {
        // `None` stands for the end-of-line marker Python's parser appends to every line.
        let chars: Vec<Option<char>> = line
            .chars()
            .map(Some)
            .chain(std::iter::once(None))
            .collect();
        for c in chars {
            let is_eol = c.is_none();
            let is_nl = matches!(c, Some('\n') | Some('\r'));
            match state {
                S::StartRecord => {
                    if is_eol {
                        rows.push(Vec::new());
                        continue;
                    }
                    if is_nl {
                        state = S::EatCrnl;
                        continue;
                    }
                    state = S::StartField;
                    // fall through to StartField handling below
                    match c {
                        Some('"') => state = S::InQuoted,
                        Some(',') => {
                            row.push(std::mem::take(&mut field));
                            state = S::StartField;
                        }
                        Some(ch) => {
                            field.push(ch);
                            state = S::InField;
                        }
                        None => {}
                    }
                }
                S::StartField => {
                    if is_eol || is_nl {
                        row.push(std::mem::take(&mut field));
                        if is_eol {
                            rows.push(std::mem::take(&mut row));
                            state = S::StartRecord;
                        } else {
                            state = S::EatCrnl;
                        }
                    } else {
                        match c {
                            Some('"') => state = S::InQuoted,
                            Some(',') => row.push(std::mem::take(&mut field)),
                            Some(ch) => {
                                field.push(ch);
                                state = S::InField;
                            }
                            None => {}
                        }
                    }
                }
                S::InField => {
                    if is_eol || is_nl {
                        row.push(std::mem::take(&mut field));
                        if is_eol {
                            rows.push(std::mem::take(&mut row));
                            state = S::StartRecord;
                        } else {
                            state = S::EatCrnl;
                        }
                    } else if c == Some(',') {
                        row.push(std::mem::take(&mut field));
                        state = S::StartField;
                    } else if let Some(ch) = c {
                        field.push(ch);
                    }
                }
                S::InQuoted => match c {
                    None => {}
                    Some('"') => state = S::QuoteInQuoted,
                    Some(ch) => field.push(ch),
                },
                S::QuoteInQuoted => {
                    if c == Some('"') {
                        field.push('"');
                        state = S::InQuoted;
                    } else if c == Some(',') {
                        row.push(std::mem::take(&mut field));
                        state = S::StartField;
                    } else if is_eol || is_nl {
                        row.push(std::mem::take(&mut field));
                        if is_eol {
                            rows.push(std::mem::take(&mut row));
                            state = S::StartRecord;
                        } else {
                            state = S::EatCrnl;
                        }
                    } else if let Some(ch) = c {
                        field.push(ch);
                        state = S::InField;
                    }
                }
                S::EatCrnl => {
                    if is_eol {
                        rows.push(std::mem::take(&mut row));
                        state = S::StartRecord;
                    }
                }
            }
        }
    }
    // End of input inside a record: Python returns what was read (non-strict dialect).
    if !field.is_empty() || state == S::InQuoted {
        row.push(std::mem::take(&mut field));
        rows.push(std::mem::take(&mut row));
    }
    rows
}

/// A CSV file read as `csv.DictReader` does: the first non-empty row names the columns.
pub struct DictRows {
    pub columns: Vec<String>,
    pub rows: Vec<std::collections::HashMap<String, String>>,
}

pub fn dict_rows(text: &str) -> DictRows {
    let mut all = csv_rows(text).into_iter().filter(|r| !r.is_empty());
    let columns = all.next().unwrap_or_default();
    let rows = all
        .map(|r| columns.iter().cloned().zip(r).collect())
        .collect();
    DictRows { columns, rows }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_text() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_764_000_000), "2025-11-24T16:00:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn csv_reader_follows_python() {
        let rows = csv_rows("a,b,c\n1,\"x,y\",\"q\"\"r\"\n\n2,,\n");
        assert_eq!(rows[0], ["a", "b", "c"]);
        assert_eq!(rows[1], ["1", "x,y", "q\"r"]);
        assert!(rows[2].is_empty());
        assert_eq!(rows[3], ["2", "", ""]);
        let multi = csv_rows("a,\"line1\nline2\"\r\nb,c");
        assert_eq!(multi[0], ["a", "line1\nline2"]);
        assert_eq!(multi[1], ["b", "c"]);
        assert_eq!(csv_rows("a\"b,c\n")[0], ["a\"b", "c"]);
    }
}
