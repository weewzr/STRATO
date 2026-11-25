//! Link check of the built site: every `href` and `src` of every HTML page that points into the
//! site must name a file that exists, and every `#anchor` must name an id on its target page.
//! Links to other sites are not fetched (the check runs without network); the content check of
//! web/scripts/check_content.mjs covers what they may not point to.

use crate::paths::{join_normalise, parent};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

/// The values of every `name="..."` (or single-quoted) attribute in `html`.
pub fn attribute_values(html: &str, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = html.as_bytes();
    let pat = format!("{name}=");
    let mut from = 0;
    while let Some(k) = html[from..].find(&pat) {
        let at = from + k;
        from = at + pat.len();
        // The attribute name must start after white space (so data-href= is not href=).
        if at == 0 || !(bytes[at - 1] as char).is_ascii_whitespace() {
            continue;
        }
        let Some(&q) = bytes.get(from) else { break };
        if q != b'"' && q != b'\'' {
            continue;
        }
        let start = from + 1;
        if let Some(len) = html[start..].find(q as char) {
            out.push(html[start..start + len].replace("&amp;", "&"));
            from = start + len + 1;
        }
    }
    out
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))? {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            walk(&p, root, out)?;
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn is_external(url: &str) -> bool {
    match url.find(':') {
        Some(c) => {
            let scheme = &url[..c];
            !scheme.is_empty()
                && !scheme.contains('/')
                && scheme
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '+' || ch == '-' || ch == '.')
        }
        None => url.starts_with("//"),
    }
}

/// Result of a link check.
#[derive(Debug, Default)]
pub struct LinkReport {
    pub pages: usize,
    pub links: usize,
    pub external: usize,
    pub failures: Vec<String>,
}

/// Checks every page under `dist`. `base` is the path the site is served under (`/STRATO/`).
pub fn check_links(dist: &Path, base: &str) -> Result<LinkReport, String> {
    let mut files = Vec::new();
    walk(dist, dist, &mut files)?;
    let all: BTreeSet<String> = files.iter().cloned().collect();
    let mut ids: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut report = LinkReport::default();
    let pages: Vec<&String> = files.iter().filter(|f| f.ends_with(".html")).collect();
    report.pages = pages.len();
    for page in pages {
        let html =
            fs::read_to_string(dist.join(page)).map_err(|e| format!("cannot read {page}: {e}"))?;
        let mut urls = attribute_values(&html, "href");
        urls.extend(attribute_values(&html, "src"));
        for url in urls {
            report.links += 1;
            if is_external(&url) {
                report.external += 1;
                continue;
            }
            let (path, frag) = match url.find('#') {
                Some(k) => (&url[..k], &url[k + 1..]),
                None => (url.as_str(), ""),
            };
            let path = path.split('?').next().unwrap_or("");
            let target = if path.is_empty() {
                Some(page.clone())
            } else if let Some(rest) = path.strip_prefix(base) {
                join_normalise("", rest)
            } else if path.starts_with('/') {
                report
                    .failures
                    .push(format!("{page}: {url} is an absolute path outside {base}"));
                continue;
            } else {
                join_normalise(parent(page), path)
            };
            let Some(mut target) = target else {
                report
                    .failures
                    .push(format!("{page}: {url} leaves the site"));
                continue;
            };
            if path.ends_with('/') || target.is_empty() || dist.join(&target).is_dir() {
                target = if target.is_empty() {
                    "index.html".to_string()
                } else {
                    format!("{target}/index.html")
                };
            }
            if !all.contains(&target) {
                report
                    .failures
                    .push(format!("{page}: {url} -> {target} does not exist"));
                continue;
            }
            // "#/..." is a route of the single-page demonstration, not an anchor.
            if !frag.is_empty() && !frag.starts_with('/') && target.ends_with(".html") {
                if !ids.contains_key(&target) {
                    let text = fs::read_to_string(dist.join(&target))
                        .map_err(|e| format!("cannot read {target}: {e}"))?;
                    let s: BTreeSet<String> = attribute_values(&text, "id")
                        .into_iter()
                        .chain(attribute_values(&text, "name"))
                        .collect();
                    ids.insert(target.clone(), s);
                }
                if !ids[&target].contains(frag) {
                    report
                        .failures
                        .push(format!("{page}: {url} -> no id \"{frag}\" on {target}"));
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_attribute_values() {
        let h = "<a href=\"a.html#x\">a</a> <img src='f.png'> <div data-href=\"no\"></div> <a\nhref=\"q?a=1&amp;b=2\">";
        assert_eq!(
            attribute_values(h, "href"),
            vec!["a.html#x".to_string(), "q?a=1&b=2".to_string()]
        );
        assert_eq!(attribute_values(h, "src"), vec!["f.png".to_string()]);
    }

    #[test]
    fn tells_external_links() {
        assert!(is_external("https://github.com/weewzr/STRATO"));
        assert!(is_external("mailto:x"));
        assert!(is_external("//cdn.example"));
        assert!(!is_external("docs/index.html"));
        assert!(!is_external("../a.html#b"));
        assert!(!is_external("#/research"));
    }

    #[test]
    fn finds_missing_files_and_anchors() {
        let dir = std::env::temp_dir().join(format!("strato-docsite-check-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs")).unwrap();
        fs::write(
            dir.join("index.html"),
            "<a href=\"/STRATO/docs/\">d</a> <a href=\"#/explore\">e</a>",
        )
        .unwrap();
        fs::write(
            dir.join("docs/index.html"),
            "<h2 id=\"x\">x</h2> <a href=\"#x\">ok</a> <a href=\"#y\">bad anchor</a> <a href=\"../\">root</a> <a href=\"gone.html\">gone</a> <a href=\"https://example.org/\">ext</a> <a href=\"/other/\">abs</a>",
        )
        .unwrap();
        let r = check_links(&dir, "/STRATO/").unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(r.pages, 2);
        assert_eq!(r.external, 1);
        assert_eq!(r.failures.len(), 3, "{:?}", r.failures);
        assert!(r.failures.iter().any(|f| f.contains("no id \"y\"")));
        assert!(r.failures.iter().any(|f| f.contains("gone.html")));
        assert!(r.failures.iter().any(|f| f.contains("outside /STRATO/")));
    }
}
