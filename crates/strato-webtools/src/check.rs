//! The research catalogue check (prompt/TECHNICAL_ARCHITECTURE.md Section 5.2).
//!
//! Reads data/research/*.csv, docs/research/*.md, data/nuclear/decay.csv and
//! web/src/generated/research_catalogue.json, writes nothing, prints one PASS, FAIL or SKIP line
//! per rule and fails if any rule fails. It checks form and references, not scientific truth: a
//! status change still needs the evidence and the independent review of the phase that delivers
//! it.
//!
//! SKIP: species.csv and gallery.csv arrive with the catalogue content of web phase 5.5. While a
//! table is absent, the rules that need it print SKIP; a table that is present is always checked
//! in full.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::catalogue::{
    self, split_list, table_header, CATALOGUE_JSON, ENTRY_DIR, GALLERY_DIR, REQUIRED_TABLES,
    RESEARCH_DIR, TABLES,
};
use crate::pyfmt::{list_repr, tuple_repr};
use crate::util::{self, Result};

pub const STATUSES: [&str; 5] = [
    "PENDING",
    "RESEARCHED",
    "IMPLEMENTED",
    "VERIFIED",
    "VALIDATED",
];
pub const SPECIES: [&str; 9] = ["Ag", "Pd", "Cs", "Sr", "Eu", "I", "Ru", "Xe", "Kr"];
pub const REGIMES: [&str; 4] = ["baseline", "A", "B", "C"];
pub const GALLERY_KINDS: [&str; 3] = ["screenshot", "reconstruction", "placeholder"];
const NOT_IN_REGISTER: &str = "(not in register)";
/// Overview pages of docs/research/ (Section 5.1). They describe several rows at once, so no
/// single row can name them in its entry column. Rule 4 accepts exactly these names without a
/// row; every other file must still be named by a row, and rule 3 checks the [ref:...] ids of
/// these pages like any other file.
pub const OVERVIEW_PAGES: [&str; 5] = [
    "docs/research/regime-a.md",
    "docs/research/regime-b.md",
    "docs/research/regime-c.md",
    "docs/research/materials_studio_gallery.md",
    "docs/research/roadmap.md",
];

#[derive(Default)]
pub struct Report {
    pub lines: Vec<String>,
    pub failed: bool,
}

impl Report {
    fn rule(&mut self, number: u32, name: &str, problems: &[String], skipped: &str) {
        if !skipped.is_empty() {
            self.lines.push(format!("SKIP {number} {name}: {skipped}"));
        } else if !problems.is_empty() {
            self.failed = true;
            for p in problems {
                self.lines.push(format!("FAIL {number} {name}: {p}"));
            }
        } else {
            self.lines.push(format!("PASS {number} {name}"));
        }
    }

    /// The numbers of the rules that have a FAIL line.
    pub fn failed_rules(&self) -> HashSet<u32> {
        self.lines
            .iter()
            .filter(|l| l.starts_with("FAIL "))
            .filter_map(|l| l.split_whitespace().nth(1)?.parse().ok())
            .collect()
    }
}

struct Row {
    line: usize,
    cells: usize,
    fields: HashMap<String, String>,
}

impl Row {
    fn get(&self, key: &str) -> &str {
        self.fields.get(key).map(String::as_str).unwrap_or("")
    }
}

struct Loaded {
    header: Vec<String>,
    rows: Vec<Row>,
    raw: Vec<u8>,
}

fn load(root: &Path) -> Result<HashMap<&'static str, Option<Loaded>>> {
    let mut out = HashMap::new();
    for (name, _) in TABLES {
        let loaded = match catalogue::read_table(root, name)? {
            None => None,
            Some(t) => {
                let rows = t
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(i, r)| Row {
                        line: i + 2,
                        cells: r.len(),
                        fields: t.header.iter().cloned().zip(r.iter().cloned()).collect(),
                    })
                    .collect();
                Some(Loaded {
                    header: t.header,
                    rows,
                    raw: t.raw,
                })
            }
        };
        out.insert(name, loaded);
    }
    Ok(out)
}

/// 'data/diffusivity/arrhenius.csv#A1-01' -> 'data/diffusivity/arrhenius.csv'.
fn path_part(item: &str) -> &str {
    item.split('#').next().unwrap_or("").trim()
}

fn ref_ids_in_text(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(k) = rest.find("[ref:") {
        let after = &rest[k + 5..];
        match after.find(']') {
            Some(e) => {
                let id = &after[..e];
                if !id.is_empty() && !id.chars().any(char::is_whitespace) {
                    out.push(id.to_string());
                    rest = &after[e + 1..];
                } else {
                    rest = after;
                }
            }
            None => break,
        }
    }
    out
}

fn rows_of(t: Option<&Loaded>) -> &[Row] {
    t.map(|t| t.rows.as_slice()).unwrap_or(&[])
}

pub fn check(root: &Path) -> Result<Report> {
    let mut rep = Report::default();
    let tables = load(root)?;
    let mech = tables["mechanisms.csv"].as_ref();
    let refs = tables["references.csv"].as_ref();
    let species = tables["species.csv"].as_ref();
    let gallery = tables["gallery.csv"].as_ref();

    // 1. Headers, cell counts, encoding and line ends.
    let mut probs: Vec<String> = Vec::new();
    for name in REQUIRED_TABLES {
        if tables[name].is_none() {
            probs.push(format!("{RESEARCH_DIR}/{name} is missing"));
        }
    }
    for (name, _) in TABLES {
        let Some(t) = tables[name].as_ref() else {
            continue;
        };
        if t.raw.starts_with(b"\xef\xbb\xbf") {
            probs.push(format!("{name}: starts with a byte-order mark"));
        }
        if t.raw.contains(&b'\r') {
            probs.push(format!("{name}: contains CR line ends (LF only)"));
        }
        let want: Vec<String> = table_header(name)
            .unwrap_or(&[])
            .iter()
            .map(|s| s.to_string())
            .collect();
        if t.header != want {
            probs.push(format!(
                "{name}: header {} differs from {}",
                list_repr(&t.header),
                list_repr(&want)
            ));
        }
        for r in &t.rows {
            if r.cells != t.header.len() {
                probs.push(format!(
                    "{name} line {}: {} cells, header has {}",
                    r.line,
                    r.cells,
                    t.header.len()
                ));
            }
        }
    }
    rep.rule(1, "headers", &probs, "");

    let mrows = rows_of(mech);
    let rrows = rows_of(refs);
    let srows = rows_of(species);
    let grows = rows_of(gallery);

    // 2. Status words.
    let mut probs = Vec::new();
    for (table, rows) in [("mechanisms.csv", mrows), ("gallery.csv", grows)] {
        for r in rows {
            let status = r.get("status");
            if !STATUSES.contains(&status) {
                probs.push(format!(
                    "{table} line {}: status '{status}' is not one of {}",
                    r.line,
                    STATUSES.join(", ")
                ));
            }
        }
    }
    rep.rule(2, "status words", &probs, "");

    // 3. References.
    let mut probs = Vec::new();
    let ref_ids: Vec<&str> = rrows.iter().map(|r| r.get("reference_id")).collect();
    let known: HashSet<&str> = ref_ids.iter().copied().collect();
    let mut dups: Vec<&str> = known
        .iter()
        .copied()
        .filter(|x| ref_ids.iter().filter(|y| *y == x).count() > 1)
        .collect();
    dups.sort();
    for rid in dups {
        probs.push(format!(
            "references.csv: reference_id '{rid}' is not unique"
        ));
    }
    for (table, rows) in [
        ("mechanisms.csv", mrows),
        ("species.csv", srows),
        ("gallery.csv", grows),
    ] {
        for r in rows {
            for rid in split_list(r.get("reference_ids")) {
                if !known.contains(rid.as_str()) {
                    probs.push(format!(
                        "{table} line {}: reference '{rid}' is not in references.csv",
                        r.line
                    ));
                }
            }
        }
    }
    let md_files = catalogue::entry_files(root)?;
    for md in &md_files {
        let text = util::read_text(md)?;
        for rid in ref_ids_in_text(&text) {
            if !known.contains(rid.as_str()) {
                probs.push(format!(
                    "{}: [ref:{rid}] is not in references.csv",
                    util::rel_posix(root, md)
                ));
            }
        }
    }
    rep.rule(3, "references", &probs, "");

    // 4. Entry files.
    let mut probs = Vec::new();
    let mut referenced: HashSet<String> = HashSet::new();
    for (table, rows) in [("mechanisms.csv", mrows), ("species.csv", srows)] {
        for r in rows {
            let entry = r.get("entry").trim();
            if entry.is_empty() {
                continue;
            }
            referenced.insert(entry.to_string());
            if !entry.starts_with(&format!("{ENTRY_DIR}/")) {
                probs.push(format!(
                    "{table} line {}: entry '{entry}' is not under {ENTRY_DIR}/",
                    r.line
                ));
            }
            if !root.join(entry).is_file() {
                probs.push(format!(
                    "{table} line {}: entry file '{entry}' does not exist",
                    r.line
                ));
            }
        }
    }
    for md in &md_files {
        let rel = util::rel_posix(root, md);
        if !referenced.contains(&rel) && !OVERVIEW_PAGES.contains(&rel.as_str()) {
            probs.push(format!("{rel} is referenced by no catalogue row"));
        }
    }
    rep.rule(4, "entry files", &probs, "");

    // 5. Uniqueness, species and regimes.
    let mut probs = Vec::new();
    let mut seen: HashMap<(String, String, String), usize> = HashMap::new();
    for r in mrows {
        let key = (
            r.get("mechanism_id").to_string(),
            r.get("species").to_string(),
            r.get("material_domain").to_string(),
        );
        if let Some(first) = seen.get(&key) {
            probs.push(format!(
                "mechanisms.csv line {}: duplicate (mechanism_id, species, material_domain) {} (first at line {first})",
                r.line,
                tuple_repr(&[&key.0, &key.1, &key.2])
            ));
        } else {
            seen.insert(key, r.line);
        }
        let sp = r.get("species");
        if !SPECIES.contains(&sp) && sp != "all" {
            probs.push(format!(
                "mechanisms.csv line {}: species '{sp}' is not one of the nine or 'all'",
                r.line
            ));
        }
        let regimes = split_list(r.get("regime"));
        if regimes.is_empty() {
            probs.push(format!("mechanisms.csv line {}: empty regime", r.line));
        }
        for g in &regimes {
            if !REGIMES.contains(&g.as_str()) {
                probs.push(format!(
                    "mechanisms.csv line {}: regime '{g}' is not one of {}",
                    r.line,
                    REGIMES.join(", ")
                ));
            }
        }
    }
    if species.is_none() {
        rep.rule(5, "uniqueness, species and regimes", &probs, "");
        rep.rule(
            5,
            "species table",
            &[],
            "species.csv absent (catalogue content of web phase 5.5)",
        );
    } else {
        let names: Vec<&str> = srows.iter().map(|r| r.get("species")).collect();
        for s in SPECIES {
            let n = names.iter().filter(|x| **x == s).count();
            if n == 0 {
                probs.push(format!("species.csv: species '{s}' is missing"));
            } else if n > 1 {
                probs.push(format!("species.csv: species '{s}' appears {n} times"));
            }
        }
        for n in &names {
            if !SPECIES.contains(n) {
                probs.push(format!("species.csv: '{n}' is not one of the nine species"));
            }
        }
        for r in srows {
            for g in split_list(r.get("regimes")) {
                if !["A", "B", "C"].contains(&g.as_str()) {
                    probs.push(format!(
                        "species.csv line {}: regime '{g}' is not A, B or C",
                        r.line
                    ));
                }
            }
        }
        rep.rule(5, "uniqueness, species and regimes", &probs, "");
    }

    // 6. Nuclides against data/nuclear/decay.csv.
    if species.is_none() {
        rep.rule(
            6,
            "nuclides",
            &[],
            "species.csv absent (catalogue content of web phase 5.5)",
        );
    } else {
        let mut probs = Vec::new();
        let decay = root.join("data").join("nuclear").join("decay.csv");
        let mut registered: HashSet<String> = HashSet::new();
        if decay.exists() {
            let d = util::dict_rows(&util::read_text(&decay)?);
            for row in d.rows {
                registered.insert(row.get("nuclide").cloned().unwrap_or_default());
            }
        } else {
            probs.push("data/nuclear/decay.csv is missing".to_string());
        }
        for r in srows {
            for n in split_list(r.get("nuclides")) {
                if n.ends_with(NOT_IN_REGISTER) {
                    continue;
                }
                if !registered.contains(&n) {
                    probs.push(format!(
                        "species.csv line {}: nuclide '{n}' is not in data/nuclear/decay.csv and is not marked '{NOT_IN_REGISTER}'",
                        r.line
                    ));
                }
            }
        }
        rep.rule(6, "nuclides", &probs, "");
    }

    // 7. Evidence for the higher statuses.
    let mut probs = Vec::new();
    for r in mrows {
        let status = r.get("status");
        let line = r.line;
        let evidence = split_list(r.get("evidence"));
        for item in &evidence {
            let p = path_part(item);
            if !p.is_empty() && !root.join(p).exists() {
                probs.push(format!(
                    "mechanisms.csv line {line}: evidence path '{p}' does not exist"
                ));
            }
        }
        if ["RESEARCHED", "IMPLEMENTED", "VERIFIED", "VALIDATED"].contains(&status)
            && evidence.is_empty()
        {
            probs.push(format!(
                "mechanisms.csv line {line}: status {status} needs evidence"
            ));
        }
        if ["IMPLEMENTED", "VERIFIED", "VALIDATED"].contains(&status)
            && r.get("solver_capability").trim().is_empty()
        {
            probs.push(format!(
                "mechanisms.csv line {line}: status {status} needs a solver_capability"
            ));
        }
        if status == "VERIFIED" || status == "VALIDATED" {
            let ok = evidence.iter().any(|e| {
                let p = path_part(e);
                p.starts_with("tests/") || p.starts_with("crates/strato-wasm/tests/")
            });
            if !ok {
                probs.push(format!(
                    "mechanisms.csv line {line}: status {status} needs evidence under tests/ or crates/strato-wasm/tests/"
                ));
            }
        }
        let has_ids = !split_list(r.get("validation_ids")).is_empty();
        if status == "VALIDATED" && !has_ids {
            probs.push(format!(
                "mechanisms.csv line {line}: status VALIDATED needs validation_ids"
            ));
        }
        if status != "VALIDATED" && has_ids {
            probs.push(format!(
                "mechanisms.csv line {line}: validation_ids must be empty unless the status is VALIDATED"
            ));
        }
    }
    rep.rule(7, "evidence", &probs, "");

    // 8. Gallery.
    let gdir = root.join(GALLERY_DIR);
    let mut gfiles: Vec<Vec<String>> = Vec::new();
    if gdir.exists() {
        collect_files(&gdir, root, &mut gfiles)?;
    }
    gfiles.sort();
    let gfiles: Vec<String> = gfiles.into_iter().map(|p| p.join("/")).collect();
    if gallery.is_none() {
        if !gfiles.is_empty() {
            rep.rule(
                8,
                "gallery",
                &[format!(
                    "{} file(s) under {GALLERY_DIR}/ but no gallery.csv",
                    gfiles.len()
                )],
                "",
            );
        } else {
            rep.rule(
                8,
                "gallery",
                &[],
                "gallery.csv absent (catalogue content of web phase 5.5)",
            );
        }
    } else {
        let mut probs = Vec::new();
        let mut listed: HashSet<String> = HashSet::new();
        for r in grows {
            let kind = r.get("kind");
            let f = r.get("file").trim();
            let line = r.line;
            if !GALLERY_KINDS.contains(&kind) {
                probs.push(format!(
                    "gallery.csv line {line}: kind '{kind}' is not one of {}",
                    GALLERY_KINDS.join(", ")
                ));
            }
            if kind == "screenshot" || kind == "reconstruction" {
                if f.is_empty() || !root.join(f).is_file() {
                    probs.push(format!(
                        "gallery.csv line {line}: a {kind} row needs an existing file"
                    ));
                }
                if r.get("provenance").trim().is_empty() {
                    probs.push(format!(
                        "gallery.csv line {line}: a {kind} row needs a provenance"
                    ));
                }
            }
            if kind == "placeholder" && !f.is_empty() {
                probs.push(format!(
                    "gallery.csv line {line}: a placeholder row must have an empty file"
                ));
            }
            if !f.is_empty() {
                listed.insert(f.to_string());
            }
        }
        for rel in &gfiles {
            if !listed.contains(rel) {
                probs.push(format!("{rel} is not listed in gallery.csv"));
            }
        }
        rep.rule(8, "gallery", &probs, "");
    }

    // 9. The generated JSON equals a fresh regeneration.
    let target = root.join(CATALOGUE_JSON);
    if !target.exists() {
        rep.rule(
            9,
            "generated catalogue",
            &[format!(
                "{CATALOGUE_JSON} is missing; run strato-webtools catalogue"
            )],
            "",
        );
    } else {
        let fresh = crate::comparable_json(
            &crate::pyfmt::json_file(&catalogue::build(root)?),
            &catalogue::CATALOGUE_VOLATILE,
        )?;
        let old =
            crate::comparable_json(&util::read_text(&target)?, &catalogue::CATALOGUE_VOLATILE)?;
        if fresh == old {
            rep.rule(9, "generated catalogue", &[], "");
        } else {
            rep.rule(
                9,
                "generated catalogue",
                &[format!(
                    "{CATALOGUE_JSON} differs from a fresh regeneration (stale)"
                )],
                "",
            );
        }
    }
    Ok(rep)
}

/// Files below `dir` as the component lists of their paths relative to `root` (for sorting the
/// way `Path` objects sort: part by part).
fn collect_files(dir: &Path, root: &Path, out: &mut Vec<Vec<String>>) -> Result<()> {
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            collect_files(&p, root, out)?;
        } else if p.is_file() {
            out.push(
                util::rel_posix(root, &p)
                    .split('/')
                    .map(str::to_string)
                    .collect(),
            );
        }
    }
    Ok(())
}
