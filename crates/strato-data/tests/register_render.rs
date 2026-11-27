//! The generated tables of `STRATO_Parameter_Register.md` (`strato-data register`): the committed
//! document equals what the CSV files say, a stale table is found and rewritten, and the
//! hand-written text outside the markers is never changed.

mod common;

use common::{data_dir, real_texts};
use strato_data::register_render::{process, Tables, BEGIN, END};

fn document() -> String {
    std::fs::read_to_string(
        data_dir()
            .join("reference")
            .join("STRATO_Parameter_Register.md"),
    )
    .expect("the register document is readable")
}

/// The document with every generated table (markers included) removed: the hand-written part.
fn hand_written(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim_end_matches(['\r', '\n']);
        if t.starts_with(BEGIN) {
            inside = true;
        } else if t == END {
            inside = false;
        } else if !inside {
            out.push_str(line);
        }
    }
    out
}

#[test]
fn the_committed_register_equals_what_the_csv_files_say() {
    let tables = Tables::read(&real_texts()).unwrap();
    let doc = document();
    let (same, reports) = process(&doc, &tables, false).unwrap();
    assert_eq!(same, doc, "a check never changes the text");
    assert!(reports.len() >= 26, "{} generated tables", reports.len());
    for r in &reports {
        assert!(
            r.current,
            "{} (line {}): {:?}",
            r.spec, r.line, r.first_difference
        );
    }
    // Every kind of table is among them.
    for kind in ["geom", "radii", "arr", "decay", "fy", "temp", "rel"] {
        assert!(
            reports.iter().any(|r| r.spec.starts_with(kind)),
            "no generated table of kind {kind}"
        );
    }
}

#[test]
fn a_changed_csv_cell_makes_its_table_stale_and_only_that_one() {
    let mut texts = real_texts();
    texts.decay = texts.decay.replacen("Cs-137,", "Cs-137X,", 1);
    let tables = Tables::read(&texts).unwrap();
    let (_, reports) = process(&document(), &tables, false).unwrap();
    let stale: Vec<&str> = reports
        .iter()
        .filter(|r| !r.current)
        .map(|r| r.spec.as_str())
        .collect();
    assert_eq!(stale, vec!["decay"]);
}

#[test]
fn writing_regenerates_the_tables_and_keeps_every_other_byte() {
    let doc = document();
    // A stale state: the tables of the document generated from a changed CSV file.
    let mut texts = real_texts();
    texts.decay = texts.decay.replacen("Cs-137,", "Cs-137X,", 1);
    let stale_tables = Tables::read(&texts).unwrap();
    let (stale_doc, _) = process(&doc, &stale_tables, true).unwrap();
    assert_ne!(stale_doc, doc);
    assert_eq!(hand_written(&stale_doc), hand_written(&doc));
    // Writing from the real files brings the document back, byte for byte.
    let tables = Tables::read(&real_texts()).unwrap();
    let (back, reports) = process(&stale_doc, &tables, true).unwrap();
    assert_eq!(back, doc);
    assert_eq!(reports.iter().filter(|r| !r.current).count(), 1);
}

#[test]
fn hand_written_text_beside_the_tables_survives_a_write() {
    let doc = document();
    let marker = doc.find(END).unwrap() + END.len();
    let mut edited = doc.clone();
    edited.insert_str(
        marker,
        "\n\nA sentence written by hand, with | pipes and $math$.\n",
    );
    let tables = Tables::read(&real_texts()).unwrap();
    let (written, _) = process(&edited, &tables, true).unwrap();
    assert_eq!(written, edited);
    assert!(written.contains("A sentence written by hand, with | pipes and $math$."));
}

#[test]
fn a_broken_marker_is_an_error_and_never_a_silent_rewrite() {
    let tables = Tables::read(&real_texts()).unwrap();
    let doc = document();
    // The marker also appears in the prose of the document: remove a closing marker line.
    let closing_line = format!("\n{END}\n");
    assert!(doc.contains(&closing_line));
    let unterminated = doc.replacen(&closing_line, "\n", 1);
    assert!(process(&unterminated, &tables, true).is_err());
    let unknown = doc.replacen("table geom", "table nothing", 1);
    assert!(process(&unknown, &tables, true).is_err());
    let stray = format!("{doc}\n{END}\n");
    assert!(process(&stray, &tables, false).is_err());
}
