//! Text hygiene of the files this crate documents and reads: no control character other than
//! TAB, LF and CR followed by LF, no byte-order mark, valid UTF-8. A stray CR or form feed once
//! damaged two words of `CHECKS.md` (the escapes `\r` and `\f` of a shell string).

mod common;

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    common::data_dir().join("..")
}

fn control_characters(text: &str) -> Vec<(usize, u32)> {
    let chars: Vec<char> = text.chars().collect();
    let mut bad = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        let code = *c as u32;
        let control = code < 32 || code == 127 || (0x80..0xa0).contains(&code);
        let allowed = *c == '\t' || *c == '\n' || (*c == '\r' && chars.get(i + 1) == Some(&'\n'));
        if control && !allowed {
            bad.push((i, code));
        }
    }
    bad
}

fn assert_clean(path: &Path) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert!(
        !bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
        "{} starts with a byte-order mark",
        path.display()
    );
    let text =
        String::from_utf8(bytes).unwrap_or_else(|e| panic!("{} is not UTF-8: {e}", path.display()));
    let bad = control_characters(&text);
    assert!(
        bad.is_empty(),
        "{} holds control characters (character index, code): {:?}",
        path.display(),
        bad
    );
}

#[test]
fn the_data_files_and_documents_hold_no_control_characters() {
    let root = repo_root();
    let mut files = vec![
        root.join("crates/strato-data/CHECKS.md"),
        root.join("data/reference/B6_audit_log.md"),
        root.join("data/reference/STRATO_Parameter_Register.md"),
        root.join("data/experimental/release_measurements.csv"),
        root.join("data/diffusivity/arrhenius.csv"),
    ];
    for dir in ["data/irradiation", "data/nuclear", "data/geometry"] {
        if let Ok(rd) = std::fs::read_dir(root.join(dir)) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "csv" || x == "md") {
                    files.push(p);
                }
            }
        }
    }
    for f in files.iter().filter(|f| f.exists()) {
        assert_clean(f);
    }
}

#[test]
fn the_detector_finds_a_bare_cr_and_a_form_feed_but_not_crlf() {
    assert_eq!(
        control_characters("a\r\nb\tc\n"),
        Vec::<(usize, u32)>::new()
    );
    assert_eq!(control_characters("column \range_kind"), vec![(7, 13)]);
    assert_eq!(control_characters("\u{c}itted range"), vec![(0, 12)]);
}
