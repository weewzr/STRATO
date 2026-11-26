//! Tests of the entry-text blocks that `strato-webtools catalogue` writes (port of
//! scripts/web/tests/test_catalogue_blocks.py).
//!
//! The web page renders the research entry files from these blocks without parsing Markdown
//! (web phase 5.5). These tests check the conversion on small inputs and on every entry file of
//! the repository: no Markdown marker is left in plain text, every citation becomes a citation
//! span, every link to another entry names an existing entry and heading, and every table row
//! has as many cells as its header.

mod common;

use common::repo_root;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap, HashSet};
use strato_webtools::catalogue::{self, markdown_blocks, reference_labels, split_list};

const REL: &str = "docs/research/x.md";

/// Every inline span of a block list, depth first.
fn spans(blocks: &[Value]) -> Vec<Value> {
    fn walk(list: &[Value], out: &mut Vec<Value>) {
        for s in list {
            out.push(s.clone());
            if let Some(c) = s.get("c").and_then(Value::as_array) {
                walk(c, out);
            }
        }
    }
    let mut out = Vec::new();
    for b in blocks {
        match b["t"].as_str().unwrap() {
            "p" | "h" => walk(b["c"].as_array().unwrap(), &mut out),
            "ul" | "ol" => {
                for it in b["items"].as_array().unwrap() {
                    walk(it.as_array().unwrap(), &mut out);
                }
            }
            "table" => {
                for c in b["head"].as_array().unwrap() {
                    walk(c.as_array().unwrap(), &mut out);
                }
                for r in b["rows"].as_array().unwrap() {
                    for c in r.as_array().unwrap() {
                        walk(c.as_array().unwrap(), &mut out);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn kinds(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|s| s["t"].as_str().unwrap())
        .collect()
}

#[test]
fn heading_paragraph_and_inline_spans() {
    let blocks = markdown_blocks(
        "# Title\n\nSome **bold $a_s^*$ text** and `code`,\nwith $D_i$ [ref:x2020] and [an entry](y.md#why-it-matters).\n",
        REL,
    )
    .unwrap();
    let types: Vec<&str> = blocks.iter().map(|b| b["t"].as_str().unwrap()).collect();
    assert_eq!(types, ["h", "p"]);
    assert_eq!(blocks[0]["id"], "title");
    assert_eq!(
        kinds(&blocks[1]["c"]),
        ["text", "strong", "text", "code", "text", "math", "text", "ref", "text", "link", "text"]
    );
    let strong = &blocks[1]["c"][1];
    assert_eq!(kinds(&strong["c"]), ["text", "math", "text"]);
    assert_eq!(strong["c"][1]["v"], "a_s^*");
    let link = &blocks[1]["c"][9];
    assert_eq!(link["entry"], "docs/research/y.md");
    assert_eq!(link["anchor"], "why-it-matters");
    assert_eq!(blocks[1]["c"][7], json!({"t": "ref", "id": "x2020"}));
}

#[test]
fn display_maths_lists_and_tables() {
    let text = "## Candidate governing equations (research concepts)\n\n\
        $$\n\\frac{\\partial c}{\\partial t} = -k c\n$$\n\n\
        $$ x = 1 $$\n\n\
        - one\n  continued\n- two [ref:a]\n\n\
        1. first\n2. second\n\n\
        | A | B |\n|---|---|\n| $|x|$ | `a|b` |\n";
    let blocks = markdown_blocks(text, REL).unwrap();
    let types: Vec<&str> = blocks.iter().map(|b| b["t"].as_str().unwrap()).collect();
    assert_eq!(types, ["h", "math", "math", "ul", "ol", "table"]);
    assert_eq!(
        blocks[0]["id"],
        "candidate-governing-equations-research-concepts"
    );
    assert_eq!(blocks[1]["tex"], r"\frac{\partial c}{\partial t} = -k c");
    assert_eq!(blocks[2]["tex"], "x = 1");
    assert_eq!(
        blocks[3]["items"][0],
        json!([{"t": "text", "v": "one continued"}])
    );
    assert_eq!(blocks[4]["start"], 1);
    assert_eq!(blocks[5]["head"].as_array().unwrap().len(), 2);
    assert_eq!(blocks[5]["rows"][0][0], json!([{"t": "math", "v": "|x|"}]));
    assert_eq!(blocks[5]["rows"][0][1], json!([{"t": "code", "v": "a|b"}]));
}

#[test]
fn repeated_headings_get_distinct_ids() {
    let blocks = markdown_blocks("## Notes\n\n## Notes\n", REL).unwrap();
    let ids: Vec<&str> = blocks.iter().map(|b| b["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["notes", "notes-1"]);
}

#[test]
fn list_cells_keep_a_semicolon_inside_parentheses() {
    assert_eq!(
        split_list("a (missing);b (missing; only a range in x2012);c (registered: f.csv#A1)"),
        [
            "a (missing)",
            "b (missing; only a range in x2012)",
            "c (registered: f.csv#A1)"
        ]
    );
    assert_eq!(split_list("x; y;;"), ["x", "y"]);
}

#[test]
fn reference_labels_follow_the_citation() {
    let rows = vec![
        json!({"reference_id": "a", "citation": "Smith, J. (2001). T.", "register_link": ""}),
        json!({"reference_id": "b", "citation": "Smith, J., & Jones, K. (2002). T.", "register_link": ""}),
        json!({"reference_id": "c", "citation": "Smith, J., Jones, K., & Lee, M. (2003). T.", "register_link": ""}),
        json!({"reference_id": "d", "citation": "Agency. (2004). T.", "register_link": "Agency (2004)"}),
        json!({"reference_id": "e", "citation": "Smith, J., Lee, K., & Jones, M. (2003). U.", "register_link": ""}),
    ];
    let labels = reference_labels(&rows).unwrap();
    assert_eq!(labels["a"], "Smith 2001");
    assert_eq!(labels["b"], "Smith and Jones 2002");
    assert_eq!(labels["c"], "Smith et al. 2003a");
    assert_eq!(labels["e"], "Smith et al. 2003b");
    assert_eq!(labels["d"], "Agency 2004");
}

fn repository() -> Value {
    catalogue::build(&repo_root()).unwrap()
}

fn entries(doc: &Value) -> &serde_json::Map<String, Value> {
    doc["data"]["entries"].as_object().unwrap()
}

fn ref_ids(doc: &Value) -> HashSet<String> {
    doc["data"]["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["reference_id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn every_entry_file_has_blocks_and_a_title() {
    let doc = repository();
    let root = repo_root();
    let mut files: Vec<String> = std::fs::read_dir(root.join(catalogue::ENTRY_DIR))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .map(|p| {
            format!(
                "{}/{}",
                catalogue::ENTRY_DIR,
                p.file_name().unwrap().to_string_lossy()
            )
        })
        .collect();
    files.sort();
    let mut got: Vec<&String> = entries(&doc).keys().collect();
    got.sort();
    assert_eq!(got, files.iter().collect::<Vec<_>>());
    for (rel, e) in entries(&doc) {
        let blocks = e["blocks"].as_array().unwrap();
        assert!(!blocks.is_empty(), "{rel}");
        assert_eq!(blocks[0]["t"], "h", "{rel}");
        assert_eq!(blocks[0]["level"], 1, "{rel}");
    }
}

#[test]
fn no_markdown_marker_is_left_in_plain_text() {
    let doc = repository();
    for (rel, e) in entries(&doc) {
        for s in spans(e["blocks"].as_array().unwrap()) {
            if s["t"] == "text" {
                let v = s["v"].as_str().unwrap();
                for marker in ["$", "**", "[ref:", "](", "`"] {
                    assert!(!v.contains(marker), "{rel}: {marker:?} in {v:?}");
                }
            }
        }
    }
}

#[test]
fn every_citation_is_a_citation_span_of_a_known_reference() {
    let doc = repository();
    let known = ref_ids(&doc);
    let re = fancy_regex::Regex::new(r"\[ref:([^\]\s]+)\]").unwrap();
    for (rel, e) in entries(&doc) {
        let source = std::fs::read_to_string(repo_root().join(rel)).unwrap();
        let mut in_text: Vec<String> = re
            .captures_iter(&source)
            .map(|c| c.unwrap()[1].to_string())
            .collect();
        let mut in_spans: Vec<String> = spans(e["blocks"].as_array().unwrap())
            .iter()
            .filter(|s| s["t"] == "ref")
            .map(|s| s["id"].as_str().unwrap().to_string())
            .collect();
        in_text.sort();
        in_spans.sort();
        assert_eq!(in_text, in_spans, "{rel}");
        assert!(in_spans.iter().all(|r| known.contains(r)), "{rel}");
    }
}

#[test]
fn every_link_to_an_entry_resolves_to_an_entry_and_heading() {
    let doc = repository();
    let anchors: HashMap<&String, BTreeSet<String>> = entries(&doc)
        .iter()
        .map(|(rel, e)| {
            (
                rel,
                e["blocks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|b| b["t"] == "h")
                    .map(|b| b["id"].as_str().unwrap().to_string())
                    .collect(),
            )
        })
        .collect();
    for (rel, e) in entries(&doc) {
        for s in spans(e["blocks"].as_array().unwrap()) {
            if s["t"] == "link" && s.get("entry").is_some() {
                let target = s["entry"].as_str().unwrap().to_string();
                let a = anchors
                    .get(&target)
                    .unwrap_or_else(|| panic!("{rel}: link to {target}, which is no entry"));
                let anchor = s["anchor"].as_str().unwrap();
                assert!(
                    anchor.is_empty() || a.contains(anchor),
                    "{rel}: {target}#{anchor}"
                );
            }
        }
    }
}

#[test]
fn every_display_equation_is_kept() {
    let doc = repository();
    let re = fancy_regex::Regex::new(r"(?s)\$\$(.+?)\$\$").unwrap();
    for (rel, e) in entries(&doc) {
        let source = std::fs::read_to_string(repo_root().join(rel))
            .unwrap()
            .replace("\r\n", "\n");
        let n = re.find_iter(&source).count();
        let kept = e["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["t"] == "math")
            .count();
        assert_eq!(kept, n, "{rel}");
    }
}

#[test]
fn concept_equations_include_those_under_sub_headings() {
    let doc = repository();
    for (rel, e) in entries(&doc) {
        let blocks = e["blocks"].as_array().unwrap();
        let Some(start) = blocks.iter().position(|b| {
            b["t"] == "h"
                && b["id"]
                    .as_str()
                    .unwrap()
                    .replace('-', " ")
                    .contains("candidate governing equations")
        }) else {
            continue;
        };
        let level = blocks[start]["level"].as_u64().unwrap();
        let mut section = Vec::new();
        for b in &blocks[start + 1..] {
            if b["t"] == "h" && b["level"].as_u64().unwrap() <= level {
                break;
            }
            section.push(b);
        }
        let want: Vec<&Value> = section
            .iter()
            .filter(|b| b["t"] == "math")
            .map(|b| &b["tex"])
            .collect();
        let got: Vec<&Value> = e["concept_equations"].as_array().unwrap().iter().collect();
        assert_eq!(got, want, "{rel}");
        assert!(!got.is_empty(), "{rel}");
    }
}

#[test]
fn table_rows_match_their_header() {
    let doc = repository();
    for (rel, e) in entries(&doc) {
        for b in e["blocks"].as_array().unwrap() {
            if b["t"] == "table" {
                let n = b["head"].as_array().unwrap().len();
                for r in b["rows"].as_array().unwrap() {
                    assert_eq!(r.as_array().unwrap().len(), n, "{rel}");
                }
            }
        }
    }
}

#[test]
fn every_reference_has_a_label() {
    let doc = repository();
    let labels = doc["data"]["reference_labels"].as_object().unwrap();
    let keys: HashSet<String> = labels.keys().cloned().collect();
    assert_eq!(keys, ref_ids(&doc));
    let values: HashSet<&str> = labels.values().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(values.len(), labels.len(), "two references share one label");
}
