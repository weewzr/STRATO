//! The research catalogue (prompt/TECHNICAL_ARCHITECTURE.md Section 5.1): the tables of
//! data/research/ and the entry files of docs/research/ as `web/src/generated/research_catalogue.json`.
//!
//! Values are copied; nothing is computed. Each entry file is also written as a list of display
//! blocks (headings, paragraphs, lists, tables, display maths, with inline code, maths, links and
//! `[ref:id]` citations), so the web page renders the text without parsing Markdown, and each
//! reference gets a short in-text label taken from its citation.

use fancy_regex::Regex;
use serde_json::{json, Map, Value};
use std::path::Path;
use std::sync::OnceLock;

use crate::util::{self, Result};
use crate::{bail, GENERATOR, GENERATOR_VERSION, SCHEMA};

pub const CATALOGUE_KIND: &str = "research_catalogue";
pub const RESEARCH_DIR: &str = "data/research";
pub const ENTRY_DIR: &str = "docs/research";
pub const GALLERY_DIR: &str = "docs/research/gallery";
pub const CATALOGUE_JSON: &str = "web/src/generated/research_catalogue.json";

/// Exact headers, in order (Section 5.1).
pub const TABLES: [(&str, &[&str]); 4] = [
    (
        "mechanisms.csv",
        &[
            "mechanism_id",
            "name",
            "regime",
            "brick",
            "builder_stage",
            "species",
            "material_domain",
            "status",
            "status_basis",
            "evidence",
            "validation_ids",
            "equation_ids",
            "coupled",
            "solver_capability",
            "required_parameters",
            "missing_work",
            "reference_ids",
            "entry",
            "status_date",
        ],
    ),
    (
        "species.csv",
        &[
            "species",
            "element",
            "nuclides",
            "layers",
            "regimes",
            "summary",
            "reference_ids",
            "entry",
        ],
    ),
    (
        "references.csv",
        &[
            "reference_id",
            "citation",
            "doi_or_url",
            "link_checked",
            "register_link",
            "notes",
        ],
    ),
    (
        "gallery.csv",
        &[
            "item_id",
            "kind",
            "title",
            "caption",
            "alt_text",
            "material",
            "species",
            "method",
            "conditions",
            "provenance",
            "file",
            "status",
            "mechanism_ids",
            "reference_ids",
        ],
    ),
];

/// The two tables a catalogue cannot do without.
pub const REQUIRED_TABLES: [&str; 2] = ["mechanisms.csv", "references.csv"];

/// Cells that hold ';'-separated lists.
const LIST_COLUMNS: [&str; 10] = [
    "regime",
    "evidence",
    "validation_ids",
    "equation_ids",
    "required_parameters",
    "reference_ids",
    "nuclides",
    "layers",
    "regimes",
    "mechanism_ids",
];
const INT_COLUMNS: [&str; 2] = ["brick", "builder_stage"];

/// Fields that record when and where a catalogue file was written; `--check` ignores them.
pub const CATALOGUE_VOLATILE: [&str; 2] = ["created_utc", "commit"];

pub fn table_header(name: &str) -> Option<&'static [&'static str]> {
    TABLES.iter().find(|(n, _)| *n == name).map(|(_, h)| *h)
}

fn rx(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("fixed pattern compiles"))
}

macro_rules! lazy_regex {
    ($name:ident, $pat:expr) => {
        fn $name() -> &'static Regex {
            static CELL: OnceLock<Regex> = OnceLock::new();
            rx(&CELL, $pat)
        }
    };
}

lazy_regex!(display_math, r"(?s)\$\$(.+?)\$\$");
lazy_regex!(heading_any, r"(?m)^(#{1,6})\s+(.*?)\s*$");
lazy_regex!(heading_line, r"^(#{1,6})\s+(.*?)\s*$");
lazy_regex!(
    inline_re,
    r"`(?P<code>[^`]+)`|(?<!\$)\$(?!\$)(?P<math>[^$\n]+?)(?<!\\)\$(?!\$)|\[ref:(?P<ref>[^\]\s]+)\]|\[(?P<ltext>[^\]]+)\]\((?P<lhref>[^)\s]+)\)|\*\*(?P<strong>.+?)\*\*|(?<![*\w])\*(?=\S)(?P<em>[^*]+?)(?<=\S)\*(?![*\w])"
);
lazy_regex!(
    list_item,
    r"^(?:(?P<ul>[-*])|(?P<ol>\d+)\.)\s+(?P<text>.*)$"
);
lazy_regex!(table_sep, r"^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)*\|?\s*$");
lazy_regex!(slug_ref, r"\[ref:[^\]]+\]");
lazy_regex!(slug_link, r"\[([^\]]+)\]\([^)]+\)");
lazy_regex!(slug_marks, r"[`*$]");
lazy_regex!(slug_punct, r"[^\w\- ]");
lazy_regex!(scheme, r"^[a-z]+://");
lazy_regex!(label_year, r"\s*\((\d{4}[a-z]?)\)\s*$");
lazy_regex!(
    label_citation,
    r"^(?P<authors>.+?)\s*\((?P<year>\d{4}[a-z]?)\)"
);
lazy_regex!(label_split, r",?\s*&\s*");

/// A table as read from data/research/: header, data rows without empty rows, raw bytes.
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub raw: Vec<u8>,
}

/// The table `data/research/<name>`, or `None` when the file does not exist.
pub fn read_table(root: &Path, name: &str) -> Result<Option<Table>> {
    let path = root.join(RESEARCH_DIR).join(name);
    if !path.exists() {
        return Ok(None);
    }
    let raw = util::read_bytes(&path)?;
    let text = util::decode_utf8(&raw, &path.display().to_string())?;
    let mut rows: Vec<Vec<String>> = util::csv_rows(&text)
        .into_iter()
        .filter(|r| !r.is_empty())
        .collect();
    if rows.is_empty() {
        return Ok(Some(Table {
            header: Vec::new(),
            rows: Vec::new(),
            raw,
        }));
    }
    let header = rows.remove(0);
    Ok(Some(Table { header, rows, raw }))
}

/// Items of a ';'-separated cell; a ';' inside parentheses belongs to its item.
pub fn split_list(cell: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0usize;
    for ch in cell.chars() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' && depth > 0 {
            depth -= 1;
        }
        if ch == ';' && depth == 0 {
            items.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    items.push(cur);
    items
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn record(header: &[String], row: &[String]) -> Result<Value> {
    let mut out = Map::new();
    for (key, cell) in header.iter().zip(row) {
        let cell = cell.trim();
        let value = if LIST_COLUMNS.contains(&key.as_str()) {
            json!(split_list(cell))
        } else if INT_COLUMNS.contains(&key.as_str()) {
            if cell.is_empty() {
                Value::Null
            } else {
                match cell.parse::<i64>() {
                    Ok(n) => json!(n),
                    Err(_) => bail!("{key}: '{cell}' is not an integer"),
                }
            }
        } else if key == "coupled" {
            json!(cell == "yes")
        } else {
            json!(cell)
        };
        out.insert(key.clone(), value);
    }
    Ok(Value::Object(out))
}

fn captures_text<'a>(c: &fancy_regex::Captures<'a>, name: &str) -> Option<&'a str> {
    c.name(name).map(|m| m.as_str())
}

/// Title, research-concept equations and the text blocks of one entry file (display data only).
fn entry_info(root: &Path, rel: &str) -> Result<Option<Value>> {
    if rel.is_empty() {
        return Ok(None);
    }
    let path = root.join(rel);
    if !path.exists() {
        return Ok(None);
    }
    let text = util::read_text(&path)?;
    let mut title = String::new();
    if let Some(m) = heading_any().captures(&text)? {
        title = m[2].to_string();
    }
    // Equations between $$ under a heading that names candidate governing equations.
    let mut concepts: Vec<Value> = Vec::new();
    let mut headings = Vec::new();
    for c in heading_any().captures_iter(&text) {
        let c = c?;
        let whole = c.get(0).expect("group 0");
        headings.push((whole.start(), whole.end(), c[1].len(), c[2].to_string()));
    }
    for (i, h) in headings.iter().enumerate() {
        if !h.3.to_lowercase().contains("candidate governing equations") {
            continue;
        }
        // The section runs to the next heading of the same or a higher level (its sub-headings,
        // such as "### Concept 1", belong to it).
        let level = h.2;
        let end = headings[i + 1..]
            .iter()
            .find(|g| g.2 <= level)
            .map(|g| g.0)
            .unwrap_or(text.len());
        for m in display_math().captures_iter(&text[h.1..end]) {
            concepts.push(json!(m?[1].trim()));
        }
    }
    Ok(Some(json!({
        "path": rel,
        "title": title,
        "concept_equations": concepts,
        "blocks": markdown_blocks(&text, rel)?,
    })))
}

/// GitHub-style anchor of a heading: lower case, punctuation dropped, spaces to hyphens.
pub fn heading_slug(text: &str) -> Result<String> {
    let plain = slug_ref().replace_all(text, "");
    let plain = slug_link().replace_all(&plain, "$1");
    let plain = slug_marks().replace_all(&plain, "");
    let plain = plain.trim().to_lowercase();
    let plain = slug_punct().replace_all(&plain, "");
    Ok(plain.replace(' ', "-"))
}

/// Lexical form of `PurePosixPath(base) / page`: empty and `.` parts dropped, `..` kept.
fn posix_join(base: &str, page: &str) -> String {
    let absolute = page.starts_with('/');
    let joined = if absolute || base.is_empty() || base == "." {
        page.to_string()
    } else {
        format!("{base}/{page}")
    };
    let parts: Vec<&str> = joined
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let text = parts.join("/");
    if absolute {
        format!("/{text}")
    } else {
        text
    }
}

fn posix_parent(rel: &str) -> String {
    match rel.rsplit_once('/') {
        Some((p, _)) => p.to_string(),
        None => ".".to_string(),
    }
}

/// A link target: an entry of docs/research/ (with an optional anchor) or an outside address.
fn link_fields(target: &str, rel: &str) -> Result<Vec<(&'static str, Value)>> {
    if scheme().is_match(target)? {
        return Ok(vec![("href", json!(target))]);
    }
    let (page, anchor) = match target.split_once('#') {
        Some((p, a)) => (p, a),
        None => (target, ""),
    };
    let path = if page.is_empty() {
        rel.to_string()
    } else {
        posix_join(&posix_parent(rel), page)
    };
    Ok(vec![("entry", json!(path)), ("anchor", json!(anchor))])
}

pub fn inline_spans(text: &str, rel: &str) -> Result<Vec<Value>> {
    let mut out: Vec<Value> = Vec::new();
    let mut pos = 0;
    for c in inline_re().captures_iter(text) {
        let c = c?;
        let whole = c.get(0).expect("group 0");
        if whole.start() > pos {
            out.push(json!({"t": "text", "v": &text[pos..whole.start()]}));
        }
        if let Some(v) = captures_text(&c, "code") {
            out.push(json!({"t": "code", "v": v}));
        } else if let Some(v) = captures_text(&c, "math") {
            out.push(json!({"t": "math", "v": v.trim()}));
        } else if let Some(v) = captures_text(&c, "ref") {
            out.push(json!({"t": "ref", "id": v}));
        } else if let Some(label) = captures_text(&c, "ltext") {
            let href = captures_text(&c, "lhref").unwrap_or("");
            let mut m = Map::new();
            m.insert("t".into(), json!("link"));
            for (k, v) in link_fields(href, rel)? {
                m.insert(k.into(), v);
            }
            m.insert("c".into(), json!(inline_spans(label, rel)?));
            out.push(Value::Object(m));
        } else if let Some(v) = captures_text(&c, "strong") {
            out.push(json!({"t": "strong", "c": inline_spans(v, rel)?}));
        } else if let Some(v) = captures_text(&c, "em") {
            out.push(json!({"t": "em", "c": inline_spans(v, rel)?}));
        }
        pos = whole.end();
    }
    if pos < text.len() {
        out.push(json!({"t": "text", "v": &text[pos..]}));
    }
    Ok(out)
}

/// Cells of a pipe-table row; a '|' inside `code` or $maths$ does not split.
fn table_cells(line: &str) -> Vec<String> {
    let mut s = line.trim();
    if let Some(r) = s.strip_prefix('|') {
        s = r;
    }
    if s.ends_with('|') && !s.ends_with("\\|") {
        s = &s[..s.len() - 1];
    }
    let mut cells = Vec::new();
    let mut cur = String::new();
    let (mut code, mut math) = (false, false);
    for ch in s.chars() {
        if ch == '`' && !math {
            code = !code;
        } else if ch == '$' && !code {
            math = !math;
        }
        if ch == '|' && !code && !math {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

fn flush(para: &mut Vec<String>, blocks: &mut Vec<Value>, rel: &str) -> Result<()> {
    if !para.is_empty() {
        let text = para.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" ");
        blocks.push(json!({"t": "p", "c": inline_spans(&text, rel)?}));
        para.clear();
    }
    Ok(())
}

/// The small Markdown subset of the entry files as a tree of blocks and inline spans. Supported:
/// ATX headings, paragraphs, "-" and "1." lists (one level, continuation lines allowed), pipe
/// tables, display maths between $$ lines, and inline code, $...$ maths, [ref:id] citations,
/// [text](target) links, **strong** and *emphasis*. Anything else is kept as plain text, so
/// nothing is silently dropped.
pub fn markdown_blocks(text: &str, rel: &str) -> Result<Vec<Value>> {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let mut blocks: Vec<Value> = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let stripped = line.trim();
        if stripped.is_empty() {
            flush(&mut para, &mut blocks, rel)?;
            i += 1;
            continue;
        }
        if let Some(h) = heading_line().captures(line)? {
            flush(&mut para, &mut blocks, rel)?;
            let slug = heading_slug(&h[2])?;
            let n = *seen.get(&slug).unwrap_or(&0);
            seen.insert(slug.clone(), n + 1);
            let id = if n == 0 { slug } else { format!("{slug}-{n}") };
            blocks.push(
                json!({"t": "h", "level": h[1].len(), "id": id, "c": inline_spans(&h[2], rel)?}),
            );
            i += 1;
            continue;
        }
        if let Some(body) = stripped.strip_prefix("$$") {
            flush(&mut para, &mut blocks, rel)?;
            let tex: String;
            if body.ends_with("$$") && body.len() >= 2 {
                tex = body[..body.len() - 2].to_string();
                i += 1;
            } else {
                let mut buf: Vec<String> = Vec::new();
                if !body.is_empty() {
                    buf.push(body.to_string());
                }
                i += 1;
                while i < lines.len() && !lines[i].trim().ends_with("$$") {
                    buf.push(lines[i].to_string());
                    i += 1;
                }
                if i < lines.len() {
                    let t = lines[i].trim();
                    let last = &t[..t.len() - 2];
                    if !last.is_empty() {
                        buf.push(last.to_string());
                    }
                    i += 1;
                }
                tex = buf.join("\n");
            }
            blocks.push(json!({"t": "math", "tex": tex.trim()}));
            continue;
        }
        if stripped.starts_with('|')
            && i + 1 < lines.len()
            && table_sep().is_match(lines[i + 1].trim())?
        {
            flush(&mut para, &mut blocks, rel)?;
            let mut head = Vec::new();
            for c in table_cells(stripped) {
                head.push(json!(inline_spans(&c, rel)?));
            }
            let mut rows = Vec::new();
            i += 2;
            while i < lines.len() && lines[i].trim().starts_with('|') {
                let mut row = Vec::new();
                for c in table_cells(lines[i]) {
                    row.push(json!(inline_spans(&c, rel)?));
                }
                rows.push(json!(row));
                i += 1;
            }
            blocks.push(json!({"t": "table", "head": head, "rows": rows}));
            continue;
        }
        if let Some(li) = list_item().captures(stripped)? {
            if !line.starts_with(' ') {
                flush(&mut para, &mut blocks, rel)?;
                let ordered = li.name("ol").is_some();
                let mut items: Vec<Vec<String>> = Vec::new();
                let start: i64 = li
                    .name("ol")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                while i < lines.len() {
                    let cur = lines[i];
                    let m = list_item().captures(cur.trim())?;
                    if let Some(m) =
                        m.filter(|m| !cur.starts_with(' ') && m.name("ol").is_some() == ordered)
                    {
                        items.push(vec![m["text"].to_string()]);
                        i += 1;
                    } else if !cur.trim().is_empty() && cur.starts_with(' ') && !items.is_empty() {
                        items
                            .last_mut()
                            .expect("non-empty")
                            .push(cur.trim().to_string());
                        i += 1;
                    } else {
                        break;
                    }
                }
                let mut spans = Vec::new();
                for it in &items {
                    spans.push(json!(inline_spans(&it.join(" "), rel)?));
                }
                let mut block = Map::new();
                block.insert("t".into(), json!(if ordered { "ol" } else { "ul" }));
                block.insert("items".into(), json!(spans));
                if ordered {
                    block.insert("start".into(), json!(start));
                }
                blocks.push(Value::Object(block));
                continue;
            }
        }
        para.push(line.to_string());
        i += 1;
    }
    flush(&mut para, &mut blocks, rel)?;
    Ok(blocks)
}

/// Short in-text label of a reference ("Author et al. 2016"), from its register citation or its
/// APA author list.
fn reference_label(row: &Value) -> Result<String> {
    let get = |k: &str| row.get(k).and_then(Value::as_str).unwrap_or("");
    let link = get("register_link");
    if !link.is_empty() {
        return Ok(label_year().replace(link.trim(), " $1").into_owned());
    }
    let citation = get("citation");
    let Some(m) = label_citation().captures(citation)? else {
        return Ok(get("reference_id").to_string());
    };
    let authors = m["authors"].trim().trim_end_matches('.').to_string();
    let year = &m["year"];
    if !authors.contains('&') {
        let first = authors.split(',').next().unwrap_or("").trim();
        return Ok(format!("{first} {year}"));
    }
    let names: Vec<String> = split_regex(label_split(), &authors)?
        .into_iter()
        .map(|p| p.trim().to_string())
        .collect();
    let count = authors.matches(".,").count() + 1;
    let first = names[0].split(',').next().unwrap_or("").trim().to_string();
    if count == 2 {
        let second = names[names.len() - 1]
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        return Ok(format!("{first} and {second} {year}"));
    }
    Ok(format!("{first} et al. {year}"))
}

/// `re.split`: the pieces between the matches of `re`.
fn split_regex(re: &Regex, text: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut pos = 0;
    for m in re.find_iter(text) {
        let m = m?;
        out.push(text[pos..m.start()].to_string());
        pos = m.end();
    }
    out.push(text[pos..].to_string());
    Ok(out)
}

/// Labels of all references; equal labels get the letters a, b, ... after the year, in table order.
pub fn reference_labels(rows: &[Value]) -> Result<Map<String, Value>> {
    let mut labels: Map<String, Value> = Map::new();
    for r in rows {
        let id = r
            .get("reference_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        labels.insert(id, json!(reference_label(r)?));
    }
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for (id, label) in &labels {
        let label = label.as_str().unwrap_or("").to_string();
        match groups.iter_mut().find(|(l, _)| *l == label) {
            Some((_, ids)) => ids.push(id.clone()),
            None => groups.push((label, vec![id.clone()])),
        }
    }
    for (label, ids) in groups {
        if ids.len() > 1 {
            for (k, id) in ids.iter().enumerate() {
                let letter = char::from(b'a' + k as u8);
                labels.insert(id.clone(), json!(format!("{label}{letter}")));
            }
        }
    }
    Ok(labels)
}

/// The sorted `*.md` files of docs/research/ (file names only, sorted).
pub fn entry_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let dir = root.join(ENTRY_DIR);
    let mut files = Vec::new();
    if dir.exists() {
        for e in std::fs::read_dir(&dir)? {
            let p = e?.path();
            if p.is_file() && p.extension().is_some_and(|x| x == "md") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}

pub fn strato_version(root: &Path) -> Result<String> {
    let path = root.join("Cargo.toml");
    if !path.exists() {
        return Ok("unknown".to_string());
    }
    let text = util::read_text(&path)?;
    let doc: toml::Value = text
        .parse()
        .map_err(|e| util::Error(format!("Cargo.toml: {e}")))?;
    match doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        Some(v) => Ok(v.to_string()),
        None => bail!("Cargo.toml: no package.version"),
    }
}

pub fn build(root: &Path) -> Result<Value> {
    let mut inputs: Vec<(String, String)> = Vec::new();
    let mut absent: Vec<String> = Vec::new();
    let mut data = Map::new();
    let mut references: Vec<Value> = Vec::new();
    for (name, _) in TABLES {
        let key = name.trim_end_matches(".csv").to_string();
        match read_table(root, name)? {
            None => {
                absent.push(name.to_string());
                data.insert(key, json!([]));
            }
            Some(t) => {
                inputs.push((format!("{RESEARCH_DIR}/{name}"), util::sha256_hex(&t.raw)));
                let mut recs = Vec::new();
                for r in &t.rows {
                    recs.push(record(&t.header, r)?);
                }
                if key == "references" {
                    references = recs.clone();
                }
                data.insert(key, json!(recs));
            }
        }
    }
    let mut entries = Map::new();
    for md in entry_files(root)? {
        let rel = util::rel_posix(root, &md);
        inputs.push((rel.clone(), util::sha256_hex(&util::read_bytes(&md)?)));
        if let Some(info) = entry_info(root, &rel)? {
            entries.insert(rel, info);
        }
    }
    data.insert("entries".into(), Value::Object(entries));
    data.insert(
        "reference_labels".into(),
        Value::Object(reference_labels(&references)?),
    );
    inputs.sort_by(|a, b| a.0.cmp(&b.0));
    let inputs: Vec<Value> = inputs
        .into_iter()
        .map(|(p, s)| json!({"path": p, "sha256": s}))
        .collect();
    let mut doc = Map::new();
    doc.insert("schema".into(), json!(SCHEMA));
    doc.insert("label".into(), json!("CATALOGUE"));
    doc.insert("kind".into(), json!(CATALOGUE_KIND));
    doc.insert(
        "generator".into(),
        json!(format!("{GENERATOR} v{GENERATOR_VERSION}")),
    );
    doc.insert("strato_version".into(), json!(strato_version(root)?));
    doc.insert("commit".into(), json!(util::head_commit(root)));
    doc.insert(
        "command".into(),
        json!(format!("python -I -X utf8 {GENERATOR} catalogue")),
    );
    doc.insert("created_utc".into(), json!(util::now_utc()));
    doc.insert(
        "note".into(),
        json!("CATALOGUE marks a data file of research statuses and references; it is not a result label."),
    );
    doc.insert(
        "entry_format".into(),
        json!("blocks-1: the entry files of docs/research/ as heading, paragraph, list, table and display-maths blocks with inline spans; text copied, nothing computed"),
    );
    doc.insert("inputs".into(), json!(inputs));
    doc.insert("tables_absent".into(), json!(absent));
    doc.insert("data".into(), Value::Object(data));
    Ok(Value::Object(doc))
}
