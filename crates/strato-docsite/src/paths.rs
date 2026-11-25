//! Repository and site paths, always written with `/`, and the relative links between pages.

/// The folder part of a path (`""` for a file at the root).
pub fn parent(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir,
        None => "",
    }
}

/// Joins `rel` to the folder `base_dir` and removes `.` and `..` parts. A `rel` that starts with
/// `/` is taken from the root. Returns `None` if the path would leave the root.
pub fn join_normalise(base_dir: &str, rel: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    if !rel.starts_with('/') {
        parts.extend(base_dir.split('/').filter(|p| !p.is_empty()));
    }
    for p in rel.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// The relative link from the page `from_file` to the file `to` (both from the site root).
pub fn relative(from_file: &str, to: &str) -> String {
    let from_dir: Vec<&str> = parent(from_file)
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();
    let to_parts: Vec<&str> = to.split('/').filter(|p| !p.is_empty()).collect();
    let mut common = 0;
    while common < from_dir.len()
        && common + 1 < to_parts.len()
        && from_dir[common] == to_parts[common]
    {
        common += 1;
    }
    let mut out = "../".repeat(from_dir.len() - common);
    out.push_str(&to_parts[common..].join("/"));
    out
}

/// The relative link from the page `from_file` to the site root (the web demonstration).
pub fn root_prefix(from_file: &str) -> String {
    let depth = parent(from_file)
        .split('/')
        .filter(|p| !p.is_empty())
        .count();
    if depth == 0 {
        "./".to_string()
    } else {
        "../".repeat(depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_and_normalises() {
        assert_eq!(
            join_normalise("docs/mathematics/phase1", "../v1.0.0/A.md").as_deref(),
            Some("docs/mathematics/v1.0.0/A.md")
        );
        assert_eq!(
            join_normalise("docs/mathematics/v1.0.0", "figures/x.png").as_deref(),
            Some("docs/mathematics/v1.0.0/figures/x.png")
        );
        assert_eq!(
            join_normalise("docs", "./a/./b/").as_deref(),
            Some("docs/a/b")
        );
        assert_eq!(
            join_normalise("docs/x", "/README.md").as_deref(),
            Some("README.md")
        );
        assert_eq!(join_normalise("docs", "../../x"), None);
    }

    #[test]
    fn relative_links() {
        assert_eq!(
            relative(
                "docs/mathematics/v1.0.0/A.html",
                "docs/mathematics/v1.0.0/B.html"
            ),
            "B.html"
        );
        assert_eq!(
            relative(
                "docs/mathematics/v1.0.0/A.html",
                "docs/mathematics/v1.0.0/figures/x.png"
            ),
            "figures/x.png"
        );
        assert_eq!(
            relative(
                "docs/mathematics/phase1/index.html",
                "docs/mathematics/v1.0.0/A.html"
            ),
            "../v1.0.0/A.html"
        );
        assert_eq!(
            relative("data/reference/index.html", "docs/index.html"),
            "../../docs/index.html"
        );
        assert_eq!(relative("docs/index.html", "docs/index.html"), "index.html");
    }

    #[test]
    fn root_prefixes() {
        assert_eq!(root_prefix("index.html"), "./");
        assert_eq!(root_prefix("docs/index.html"), "../");
        assert_eq!(root_prefix("docs/mathematics/v1.0.0/A.html"), "../../../");
    }
}
