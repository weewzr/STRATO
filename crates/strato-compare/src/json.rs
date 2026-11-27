//! A small JSON value and writer. The output files are plain data; nothing here reads JSON.
//!
//! A non-finite number is never written as a number: it becomes `null`, and the callers only
//! pass `None` for a quantity that is not defined.

use std::fmt::Write as _;

/// A JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum J {
    /// `null`: not defined.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A finite number (a non-finite one is written as `null`).
    Num(f64),
    /// An integer.
    Int(i64),
    /// A string.
    Str(String),
    /// An array.
    Arr(Vec<J>),
    /// An object; the order of the keys is kept.
    Obj(Vec<(String, J)>),
}

impl J {
    /// A string value.
    pub fn s(text: impl Into<String>) -> J {
        J::Str(text.into())
    }

    /// A number that may be undefined.
    pub fn opt(x: Option<f64>) -> J {
        match x {
            Some(v) => J::Num(v),
            None => J::Null,
        }
    }

    /// An object from key and value pairs.
    pub fn obj(items: Vec<(&str, J)>) -> J {
        J::Obj(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    /// A string array.
    pub fn strs(items: &[&str]) -> J {
        J::Arr(items.iter().map(|s| J::s(*s)).collect())
    }

    /// Render with two-space indentation and a final newline.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            J::Null => out.push_str("null"),
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            J::Num(x) => {
                if x.is_finite() {
                    let _ = write!(out, "{x:e}");
                } else {
                    out.push_str("null");
                }
            }
            J::Int(i) => {
                let _ = write!(out, "{i}");
            }
            J::Str(s) => quote(s, out),
            J::Arr(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                // Short arrays of numbers stay on one line.
                let flat = items
                    .iter()
                    .all(|v| matches!(v, J::Num(_) | J::Int(_) | J::Null | J::Bool(_)));
                if flat {
                    out.push('[');
                    for (k, v) in items.iter().enumerate() {
                        if k > 0 {
                            out.push(',');
                        }
                        v.write(out, depth);
                    }
                    out.push(']');
                    return;
                }
                out.push_str("[\n");
                for (k, v) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    v.write(out, depth + 1);
                    if k + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push(']');
            }
            J::Obj(items) => {
                if items.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (k, (key, v)) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    quote(key, out);
                    out.push_str(": ");
                    v.write(out, depth + 1);
                    if k + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push('}');
            }
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_escapes_and_undefined_numbers() {
        let v = J::obj(vec![
            ("a", J::s("x\"y\\z\nq")),
            ("b", J::Num(f64::NAN)),
            ("c", J::Arr(vec![J::Num(1.5), J::Int(2), J::Null])),
            ("d", J::opt(None)),
            ("e", J::Obj(vec![])),
        ]);
        let t = v.render();
        assert!(t.contains("\"a\": \"x\\\"y\\\\z\\nq\""));
        assert!(t.contains("\"b\": null"));
        assert!(t.contains("\"c\": [1.5e0,2,null]"));
        assert!(t.contains("\"d\": null"));
        assert!(t.contains("\"e\": {}"));
    }

    #[test]
    fn numbers_round_trip_through_text() {
        let x = 0.1_f64 + 0.2;
        let t = J::Num(x).render();
        assert_eq!(t.trim().parse::<f64>().unwrap(), x);
    }
}
