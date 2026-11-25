//! Python-compatible text formatting.
//!
//! The data files of the web demo were written by Python (`json.dumps`, `repr`, `csv.writer`,
//! `str.splitlines`). The Rust tool must write the same bytes, so the few Python formats it
//! needs are reproduced here exactly: the shortest round-trip float text of `repr(float)`, the
//! JSON writer with `indent=2` or the default separators, `repr(str)`, `csv.writer` quoting and
//! universal newlines of `Path.read_text`.

use serde_json::Value;

/// `repr(float)` of CPython: the shortest text that reads back as the same double, in plain
/// notation for decimal exponents from -4 to 15 and in the form `1e-05` / `1.5e+16` outside.
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".to_string();
    }
    if x.is_infinite() {
        return if x < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    // Rust's `{:e}` prints the shortest digits that round-trip: "-1.2345e-5", "1e16", "0e0".
    let sci = format!("{x:e}");
    let (mant, exp) = sci
        .split_once('e')
        .expect("LowerExp always has an exponent");
    let exp: i32 = exp.parse().expect("exponent is an integer");
    let (sign, mant) = match mant.strip_prefix('-') {
        Some(m) => ("-", m),
        None => ("", mant),
    };
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    if digits.chars().all(|c| c == '0') {
        return format!("{sign}0.0");
    }
    let decpt = exp + 1;
    if !(-3..=16).contains(&decpt) {
        // Exponent form: d[.ddd]e[+-]XX with at least two exponent digits.
        let mut m = String::new();
        m.push_str(&digits[..1]);
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let esign = if exp < 0 { '-' } else { '+' };
        return format!("{sign}{m}e{esign}{:02}", exp.abs());
    }
    let n = digits.len() as i32;
    let body = if decpt <= 0 {
        format!("0.{}{}", "0".repeat((-decpt) as usize), digits)
    } else if decpt >= n {
        format!("{}{}.0", digits, "0".repeat((decpt - n) as usize))
    } else {
        format!(
            "{}.{}",
            &digits[..decpt as usize],
            &digits[decpt as usize..]
        )
    };
    format!("{sign}{body}")
}

/// `repr(str)` of CPython for the strings the checks print.
pub fn str_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `repr(list_of_str)`, for example `['a', 'b']`.
pub fn list_repr(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| str_repr(s)).collect();
    format!("[{}]", inner.join(", "))
}

/// `repr(tuple_of_str)`, for example `('a', 'b')`.
pub fn tuple_repr(items: &[&str]) -> String {
    let inner: Vec<String> = items.iter().map(|s| str_repr(s)).collect();
    if inner.len() == 1 {
        format!("({},)", inner[0])
    } else {
        format!("({})", inner.join(", "))
    }
}

/// `repr` of a JSON-like scalar as Python would print the value read from TOML.
pub fn value_repr(v: &Value) -> String {
    match v {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else {
                float_repr(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        Value::String(s) => str_repr(s),
        other => other.to_string(),
    }
}

fn json_string(s: &str, ensure_ascii: bool, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if ensure_ascii && !c.is_ascii() => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_number(n: &serde_json::Number, out: &mut String) {
    if let Some(i) = n.as_i64() {
        out.push_str(&i.to_string());
    } else if let Some(u) = n.as_u64() {
        out.push_str(&u.to_string());
    } else {
        out.push_str(&float_repr(n.as_f64().unwrap_or(f64::NAN)));
    }
}

#[derive(Clone, Copy)]
pub struct Dump {
    /// `Some(n)`: `indent=n`, with the separators `,` and `: `; `None`: the default `, ` and `: `.
    pub indent: Option<usize>,
    pub ensure_ascii: bool,
    pub sort_keys: bool,
}

/// `json.dumps(doc, indent=2, ensure_ascii=False) + "\n"`, the form of every JSON data file.
pub fn json_file(doc: &Value) -> String {
    let mut s = dumps(
        doc,
        Dump {
            indent: Some(2),
            ensure_ascii: false,
            sort_keys: false,
        },
    );
    s.push('\n');
    s
}

/// `json.dumps(doc, sort_keys=True)`: the canonical form compared by `--check` and hashed.
pub fn json_canonical(doc: &Value) -> String {
    dumps(
        doc,
        Dump {
            indent: None,
            ensure_ascii: true,
            sort_keys: true,
        },
    )
}

pub fn dumps(v: &Value, opt: Dump) -> String {
    let mut out = String::new();
    write_value(v, opt, 0, &mut out);
    out
}

fn newline(opt: Dump, level: usize, out: &mut String) {
    if let Some(n) = opt.indent {
        out.push('\n');
        out.push_str(&" ".repeat(n * level));
    }
}

fn write_value(v: &Value, opt: Dump, level: usize, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => json_number(n, out),
        Value::String(s) => json_string(s, opt.ensure_ascii, out),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (k, item) in a.iter().enumerate() {
                if k > 0 {
                    out.push_str(if opt.indent.is_some() { "," } else { ", " });
                }
                newline(opt, level + 1, out);
                write_value(item, opt, level + 1, out);
            }
            newline(opt, level, out);
            out.push(']');
        }
        Value::Object(m) => {
            if m.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            let mut keys: Vec<&String> = m.keys().collect();
            if opt.sort_keys {
                keys.sort();
            }
            for (k, key) in keys.into_iter().enumerate() {
                if k > 0 {
                    out.push_str(if opt.indent.is_some() { "," } else { ", " });
                }
                newline(opt, level + 1, out);
                json_string(key, opt.ensure_ascii, out);
                out.push_str(": ");
                write_value(&m[key], opt, level + 1, out);
            }
            newline(opt, level, out);
            out.push('}');
        }
    }
}

/// One CSV field as `csv.writer` (default dialect, QUOTE_MINIMAL) writes it.
fn csv_field(f: &str, out: &mut String) {
    if f.contains([',', '"', '\n', '\r']) {
        out.push('"');
        out.push_str(&f.replace('"', "\"\""));
        out.push('"');
    } else {
        out.push_str(f);
    }
}

/// One row as `csv.writer(buf, lineterminator="\n").writerow(row)`.
pub fn csv_row<S: AsRef<str>>(row: &[S]) -> String {
    let mut out = String::new();
    for (k, f) in row.iter().enumerate() {
        if k > 0 {
            out.push(',');
        }
        csv_field(f.as_ref(), &mut out);
    }
    out.push('\n');
    out
}

/// `csv_line!(a, b, c)`: one CSV row of fields that are `&str` or `String`.
#[macro_export]
macro_rules! csv_line {
    ($($x:expr),* $(,)?) => {
        $crate::pyfmt::csv_row(&[$(AsRef::<str>::as_ref(&$x)),*])
    };
}

/// `str.splitlines()`: breaks at the line boundaries Python knows, dropping them.
pub fn splitlines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes: Vec<(usize, char)> = text.char_indices().collect();
    let mut k = 0;
    while k < bytes.len() {
        let (pos, c) = bytes[k];
        let is_break = matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if is_break {
            lines.push(&text[start..pos]);
            if c == '\r' && k + 1 < bytes.len() && bytes[k + 1].1 == '\n' {
                k += 1;
            }
            start = if k + 1 < bytes.len() {
                bytes[k + 1].0
            } else {
                text.len()
            };
        }
        k += 1;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// Universal newlines of `Path.read_text`: `\r\n` and a lone `\r` become `\n`.
pub fn universal_newlines(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_string();
    }
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_repr_matches_python() {
        // Values and texts taken from CPython 3.12 repr().
        let cases: [(f64, &str); 18] = [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (1.0, "1.0"),
            (0.5, "0.5"),
            (0.0001, "0.0001"),
            (0.00001, "1e-05"),
            (1.5e-7, "1.5e-07"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (1.2345e20, "1.2345e+20"),
            (123456789.123, "123456789.123"),
            (1e100, "1e+100"),
            (1e-100, "1e-100"),
            (0.1 + 0.2, "0.30000000000000004"),
            (-2.5e-5, "-2.5e-05"),
            (9999999999999998.0, "9999999999999998.0"),
            (12345678901234567.0, "1.2345678901234568e+16"),
            (5e-324, "5e-324"),
        ];
        for (x, want) in cases {
            assert_eq!(float_repr(x), want, "{x:e}");
        }
    }

    #[test]
    fn json_layout_matches_python() {
        let v: Value =
            serde_json::from_str(r#"{"a":[],"b":{},"c":[1,2.0,"x\tyé"],"d":null}"#).unwrap();
        let want = "{\n  \"a\": [],\n  \"b\": {},\n  \"c\": [\n    1,\n    2.0,\n    \"x\\ty\u{e9}\"\n  ],\n  \"d\": null\n}\n";
        assert_eq!(json_file(&v), want);
        assert_eq!(
            json_canonical(&v),
            "{\"a\": [], \"b\": {}, \"c\": [1, 2.0, \"x\\ty\x5cu00e9\"], \"d\": null}"
        );
    }

    #[test]
    fn str_repr_chooses_quotes_like_python() {
        assert_eq!(str_repr("Verified"), "'Verified'");
        assert_eq!(str_repr("it's"), "\"it's\"");
        assert_eq!(str_repr("a\"b'c"), "'a\"b\\'c'");
        assert_eq!(tuple_repr(&["a", "b"]), "('a', 'b')");
    }

    #[test]
    fn csv_quoting_is_minimal() {
        assert_eq!(
            csv_row(&["a", "b,c", "d\"e", ""]),
            "a,\"b,c\",\"d\"\"e\",\n"
        );
    }

    #[test]
    fn splitlines_matches_python() {
        assert_eq!(splitlines("a\nb\r\nc\rd"), ["a", "b", "c", "d"]);
        assert_eq!(splitlines("a\n"), ["a"]);
        assert_eq!(splitlines("a\n\nb"), ["a", "", "b"]);
        assert!(splitlines("").is_empty());
    }
}
