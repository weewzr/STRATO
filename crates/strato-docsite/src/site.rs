//! The published document set and the build of its pages into the web build folder.

use crate::markdown::{self, esc, Rendered, Target};
use crate::paths::{join_normalise, parent, relative, root_prefix};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const REPO_URL: &str = "https://github.com/weewzr/STRATO";

/// Where the shared files of the document pages go, from the site root.
pub const ASSETS: &str = "docs/_assets";

/// Folders that get a generated list page, besides the folders whose README.md is published.
pub const LISTED_DIRS: &[&str] = &["docs", "docs/mathematics", "docs/mathematics/btcs"];

/// What kind of document an entry is; the page says so above the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Btcs,
    Phase2,
    Register,
}

impl Group {
    pub fn heading(self) -> &'static str {
        match self {
            Group::Btcs => "BTCS mathematical documentation (version 2.0)",
            Group::Phase2 => "Phase 2 design: derived, not implemented",
            Group::Register => "Parameter register",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Group::Btcs => "btcs",
            Group::Phase2 => "phase2",
            Group::Register => "register",
        }
    }

    /// The status line shown above every document of the group.
    pub fn status(self) -> &'static str {
        match self {
            Group::Btcs => "Documents 0 to 4 of the spherical BTCS finite-difference solver of version 2.0: the account of the original documents, the derivation, the final formulation (Document 2, normative), the Rust pseudocode and the equation-to-code traceability. The solver is verified numerically on synthetic cases (verification, not validation against measurements).",
            Group::Phase2 => "Design documents written for the finite-volume form of the scheme before version 2.0. Fission-product generation, radioactive decay and temperature-dependent diffusivity are derived and specified here; the BTCS solver does not implement them yet (research status RESEARCHED in the web demonstration), and the BTCS documents supersede these as each mechanism is added.",
            Group::Register => "Literature parameters with the source, page and status of every value. The live simulation of the web demonstration uses the synthetic five-layer verification case, not these values.",
        }
    }
}

/// One published document.
pub struct Doc {
    /// Markdown source, from the repository root.
    pub src: &'static str,
    /// Title in the document lists.
    pub title: &'static str,
    pub group: Group,
    /// One sentence for the document lists.
    pub summary: &'static str,
}

/// Every published document. Nothing else is published; README.md, CHANGELOG.md and
/// docs/development/ are deliberately absent, and nothing under prompt/ may be listed.
pub const DOCS: &[Doc] = &[
    Doc {
        src: "docs/mathematics/btcs/00_original_documents.md",
        title: "Document 0: the original FTCS documents, an account and an algebra check",
        group: Group::Btcs,
        summary: "A faithful account of the two original documents with a verdict on every algebra step; the starting point of the BTCS reconstruction.",
    },
    Doc {
        src: "docs/mathematics/btcs/01_from_ftcs_to_five_layer_btcs.md",
        title: "Document 1: from the explicit scheme to the five-layer BTCS scheme",
        group: Group::Btcs,
        summary: "The full derivation of the five-layer BTCS scheme, with the properties proved, outlined or observed, each labelled as such.",
    },
    Doc {
        src: "docs/mathematics/btcs/02_final_formulation.md",
        title: "Document 2: the final BTCS formulation (normative)",
        group: Group::Btcs,
        summary: "The complete five-layer system, row by row: grid, matrix rows, right-hand side, solve, inventory, release and balance, with every numbered equation the code cites.",
    },
    Doc {
        src: "docs/mathematics/btcs/03_rust_pseudocode.md",
        title: "Document 3: Rust pseudocode",
        group: Group::Btcs,
        summary: "The routines that implement Document 2, with inputs, outputs, units and the equations each implements.",
    },
    Doc {
        src: "docs/mathematics/btcs/04_equation_to_code_traceability.md",
        title: "Document 4: equation-to-code traceability",
        group: Group::Btcs,
        summary: "Every numbered equation of Document 2 mapped to its pseudocode routine, the Rust item that holds it and the test that checks it.",
    },    Doc {
        src: "docs/mathematics/STRATO_Phase2_Derivations.md",
        title: "Phase 2 derivations",
        group: Group::Phase2,
        summary: "Derivations of fission-product generation, radioactive decay, inventory accounting and temperature-dependent diffusivity with layer-specific laws.",
    },
    Doc {
        src: "docs/mathematics/STRATO_Phase2_Pseudocode.md",
        title: "Phase 2 pseudocode",
        group: Group::Phase2,
        summary: "Pseudocode of the Phase 2 solver steps specified by the derivations.",
    },
    Doc {
        src: "data/reference/README.md",
        title: "Data folder guide",
        group: Group::Register,
        summary: "The data files behind the register, their columns, status words and evidence classes.",
    },
    Doc {
        src: "data/reference/STRATO_Parameter_Register.md",
        title: "STRATO Parameter Register",
        group: Group::Register,
        summary: "Notation, defining relations, the numbered parameter tables, verification output, gaps and references.",
    },
    Doc {
        src: "data/reference/B6_audit_log.md",
        title: "B6 audit log of the register",
        group: Group::Register,
        summary: "Record of the audit that added the evidence-class columns to the register tables.",
    },
];

/// True for a path that may never be published or linked from the site (owner decision M3).
pub fn is_forbidden(path: &str) -> bool {
    path == "prompt" || path.starts_with("prompt/")
}

/// The page of a document: README.md becomes the folder's index.html, any other file keeps its
/// name with `.html` (the addresses of the earlier Jekyll render).
pub fn out_path(src: &str) -> String {
    let (dir, name) = match src.rsplit_once('/') {
        Some((d, n)) => (d, n),
        None => ("", src),
    };
    let stem = name.strip_suffix(".md").unwrap_or(name);
    let file = if stem == "README" {
        "index.html".to_string()
    } else {
        format!("{stem}.html")
    };
    if dir.is_empty() {
        file
    } else {
        format!("{dir}/{file}")
    }
}

fn is_copied_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [".png", ".svg", ".jpg", ".jpeg", ".gif", ".pdf"]
        .iter()
        .any(|e| lower.ends_with(e))
}

/// Input of a build.
pub struct Config {
    /// Repository root.
    pub repo: PathBuf,
    /// Web build folder (web/dist after `npm run build`).
    pub out: PathBuf,
    /// Full commit hash the site is built from (links to GitHub use it).
    pub commit: String,
}

/// What a build wrote.
#[derive(Debug, Default)]
pub struct Report {
    /// One line per page or copied file.
    pub lines: Vec<String>,
}

struct Site {
    repo: PathBuf,
    commit: String,
    version: String,
    katex_version: String,
    published: BTreeMap<String, String>,
    indexed: BTreeSet<String>,
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| format!("cannot copy {} to {}: {e}", from.display(), to.display()))
}

/// The value of the first `key = "..."` line of a TOML file, or of `"key": "..."` in a JSON file.
fn quoted_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let l = line.trim();
        let rest = l
            .strip_prefix(&format!("{key} = \""))
            .or_else(|| l.strip_prefix(&format!("\"{key}\": \"")));
        if let Some(rest) = rest {
            return rest.split('"').next().map(str::to_string);
        }
    }
    None
}

/// The files of a folder, sorted by name.
fn files_of(dir: &Path) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_file() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names.sort();
    Ok(names)
}

impl Site {
    fn short(&self) -> &str {
        &self.commit[..self.commit.len().min(7)]
    }

    fn blob(&self, path: &str) -> String {
        format!("{REPO_URL}/blob/{}/{path}", self.commit)
    }

    /// What a link `dest` written in the document `src` becomes on the page `page`.
    fn resolve(
        &self,
        src: &str,
        page: &str,
        dest: &str,
        copied: &mut BTreeSet<String>,
        notes: &mut Vec<String>,
        problems: &mut Vec<String>,
    ) -> Target {
        let lower = dest.to_ascii_lowercase();
        if dest.is_empty()
            || dest.starts_with('#')
            || lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("mailto:")
        {
            return Target::Keep(dest.to_string());
        }
        let (path, frag) = match dest.find('#') {
            Some(k) => (&dest[..k], &dest[k..]),
            None => (dest, ""),
        };
        let Some(repo_path) = join_normalise(parent(src), path) else {
            problems.push(format!("{src}: the link {dest} leaves the repository"));
            return Target::Text;
        };
        if is_forbidden(&repo_path) {
            notes.push(format!(
                "NOTE {src}: link to {repo_path} kept as text (prompt/ is not published)"
            ));
            return Target::Text;
        }
        if let Some(out) = self.published.get(&repo_path) {
            return Target::Keep(format!("{}{frag}", relative(page, out)));
        }
        if self.indexed.contains(&repo_path) {
            return Target::Keep(format!(
                "{}{frag}",
                relative(page, &format!("{repo_path}/index.html"))
            ));
        }
        let on_disk = self.repo.join(&repo_path);
        if on_disk.is_file() && is_copied_file(&repo_path) {
            copied.insert(repo_path.clone());
            return Target::Keep(format!("{}{frag}", relative(page, &repo_path)));
        }
        if on_disk.is_dir() {
            return Target::Keep(format!("{REPO_URL}/tree/{}/{repo_path}{frag}", self.commit));
        }
        if on_disk.is_file() {
            return Target::Keep(format!("{}{frag}", self.blob(&repo_path)));
        }
        problems.push(format!(
            "{src}: the link {dest} names {repo_path}, which is not in the repository"
        ));
        Target::Text
    }

    /// One complete page.
    fn page(
        &self,
        out: &str,
        title: &str,
        meta: &[(String, String)],
        toc: &[(String, String)],
        body: &str,
        maths: bool,
    ) -> String {
        let root = root_prefix(out);
        let assets = relative(out, &format!("{ASSETS}/docs.css"));
        let assets = assets.trim_end_matches("docs.css");
        let docs_index = relative(out, "docs/index.html");
        let current = if out == "docs/index.html" {
            " aria-current=\"page\""
        } else {
            ""
        };
        let mut h = String::with_capacity(body.len() + 4096);
        h.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
        h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        h.push_str(&format!("<title>{} | STRATO</title>\n", esc(title)));
        h.push_str(&format!(
            "<meta name=\"strato-build\" content=\"{}\">\n",
            esc(self.short())
        ));
        h.push_str("<meta name=\"color-scheme\" content=\"light dark\">\n");
        h.push_str(&format!(
            "<link rel=\"icon\" href=\"{root}favicon.svg\" type=\"image/svg+xml\">\n"
        ));
        // The reader's theme choice on the demonstration (same key as web/src/theme.ts).
        h.push_str("<script>try{var t=localStorage.getItem('strato-theme');if(t==='light'||t==='dark')document.documentElement.setAttribute('data-theme',t)}catch(e){}</script>\n");
        if maths {
            h.push_str(&format!(
                "<link rel=\"stylesheet\" href=\"{assets}katex/katex.min.css\">\n"
            ));
        }
        h.push_str(&format!(
            "<link rel=\"stylesheet\" href=\"{assets}docs.css\">\n"
        ));
        h.push_str("</head>\n<body>\n<a class=\"skip\" href=\"#main\">Skip to the text</a>\n");
        h.push_str("<header class=\"site-header\"><div class=\"site-header__inner\">");
        h.push_str(&format!(
            "<a class=\"site-header__name\" href=\"{root}\">STRATO</a>"
        ));
        h.push_str("<nav aria-label=\"Site\"><ul>");
        h.push_str(&format!(
            "<li><a href=\"{root}\">Web demonstration</a></li>"
        ));
        h.push_str(&format!(
            "<li><a href=\"{docs_index}\"{current}>Documents</a></li>"
        ));
        h.push_str(&format!(
            "<li><a href=\"{REPO_URL}\">Source code on GitHub</a></li>"
        ));
        h.push_str("</ul></nav></div></header>\n<main id=\"main\" class=\"doc-wrap\">\n");
        if !meta.is_empty() {
            h.push_str("<dl class=\"doc-meta\">\n");
            for (k, v) in meta {
                h.push_str(&format!("<div><dt>{}</dt><dd>{v}</dd></div>\n", esc(k)));
            }
            h.push_str("</dl>\n");
        }
        if toc.len() >= 3 {
            h.push_str("<nav class=\"toc\" aria-label=\"Contents\"><details open><summary>Contents</summary><ol>\n");
            for (id, label) in toc {
                h.push_str(&format!("<li><a href=\"#{}\">{label}</a></li>\n", esc(id)));
            }
            h.push_str("</ol></details></nav>\n");
        }
        h.push_str("<article class=\"doc\">\n");
        h.push_str(body);
        h.push_str("</article>\n</main>\n<footer class=\"site-footer\"><div class=\"site-footer__inner\"><p>");
        h.push_str(&format!(
            "Rendered from the STRATO repository at commit <a href=\"{REPO_URL}/tree/{}\"><code>{}</code></a> (STRATO {}) by <code>crates/strato-docsite</code>.",
            esc(&self.commit),
            esc(self.short()),
            esc(&self.version)
        ));
        if maths {
            h.push_str(&format!(
                " Mathematics typeset in your browser by KaTeX {}, served from this site.",
                esc(&self.katex_version)
            ));
        }
        h.push_str(" Licence: AGPL-3.0-only.</p></div></footer>\n");
        if maths {
            h.push_str(&format!(
                "<script src=\"{assets}katex/katex.min.js\" defer></script>\n"
            ));
            h.push_str(&format!(
                "<script src=\"{assets}render-math.js\" defer></script>\n"
            ));
        }
        h.push_str("</body>\n</html>\n");
        h
    }

    fn doc_meta(&self, doc: &Doc, pdf: Option<&str>) -> Vec<(String, String)> {
        let mut meta = vec![
            ("Status".to_string(), esc(doc.group.status())),
            (
                "Source".to_string(),
                format!(
                    "<a href=\"{}\"><code>{}</code></a>",
                    esc(&self.blob(doc.src)),
                    esc(doc.src)
                ),
            ),
        ];
        if let Some(pdf) = pdf {
            let name = pdf.rsplit('/').next().unwrap_or(pdf);
            meta.push((
                "PDF".to_string(),
                format!(
                    "<a href=\"{}\">{}</a>",
                    esc(&relative(&out_path(doc.src), pdf)),
                    esc(name)
                ),
            ));
        }
        meta
    }

    fn pdf_of(&self, doc: &Doc) -> Option<String> {
        let pdf = format!("{}.pdf", doc.src.strip_suffix(".md").unwrap_or(doc.src));
        if self.repo.join(&pdf).is_file() {
            Some(pdf)
        } else {
            None
        }
    }

    fn doc_list(&self, page: &str, groups: &[Group]) -> String {
        let mut h = String::new();
        for g in groups {
            h.push_str(&format!("<section class=\"doc-group\" aria-labelledby=\"{}\">\n<h2 id=\"{}\">{}</h2>\n<p class=\"doc-group__status\">{}</p>\n<ul class=\"doc-list\">\n", g.id(), g.id(), esc(g.heading()), esc(g.status())));
            for d in DOCS.iter().filter(|d| d.group == *g) {
                h.push_str(&format!(
                    "<li><a class=\"doc-list__title\" href=\"{}\">{}</a>",
                    esc(&relative(page, &out_path(d.src))),
                    esc(d.title)
                ));
                h.push_str(&format!(
                    "<p>{}</p><p class=\"doc-list__links\">",
                    esc(d.summary)
                ));
                if let Some(pdf) = self.pdf_of(d) {
                    h.push_str(&format!(
                        "<a href=\"{}\">PDF</a> · ",
                        esc(&relative(page, &pdf))
                    ));
                }
                h.push_str(&format!(
                    "<a href=\"{}\">Markdown source on GitHub</a></p></li>\n",
                    esc(&self.blob(d.src))
                ));
            }
            h.push_str("</ul>\n</section>\n");
        }
        h
    }

    fn list_page(&self, dir: &str) -> (String, String) {
        let out = format!("{dir}/index.html");
        let (title, groups, intro): (&str, Vec<Group>, &str) = match dir {
            "docs" => ("Documents", vec![Group::Btcs, Group::Phase2, Group::Register], "The derivation and register documents of STRATO, rendered from the repository with the mathematics typeset. Each page links its Markdown source on GitHub and, where one exists, its PDF."),
            "docs/mathematics" => ("Mathematical documentation", vec![Group::Btcs, Group::Phase2], "The derivations and pseudocode of STRATO, rendered from the repository with the mathematics typeset."),
            _ => ("BTCS mathematical documentation (version 2.0)", vec![Group::Btcs], "Documents 0 to 4 of the spherical BTCS finite-difference solver, rendered from the repository with the mathematics typeset."),
        };
        let mut body = format!(
            "<h1>{}</h1>\n<p class=\"lede\">{}</p>\n",
            esc(title),
            esc(intro)
        );
        body.push_str(&self.doc_list(&out, &groups));
        (out.clone(), self.page(&out, title, &[], &[], &body, false))
    }
}

/// The CSS, scripts, KaTeX and fonts of the document pages: (source in the repository, target in
/// the site). KaTeX and the fonts come from web/node_modules (versions pinned by
/// web/package-lock.json), the token files from web/src/design.
fn asset_files(repo: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let mut list: Vec<(PathBuf, String)> = Vec::new();
    let katex = repo.join("web/node_modules/katex/dist");
    for f in ["katex.min.js", "katex.min.css"] {
        list.push((katex.join(f), format!("{ASSETS}/katex/{f}")));
    }
    for f in files_of(&katex.join("fonts"))? {
        list.push((
            katex.join("fonts").join(&f),
            format!("{ASSETS}/katex/fonts/{f}"),
        ));
    }
    let fonts = [
        ("literata", "literata-latin-400-normal.woff2"),
        ("literata", "literata-latin-400-italic.woff2"),
        ("literata", "literata-latin-600-normal.woff2"),
        ("ibm-plex-sans", "ibm-plex-sans-latin-400-normal.woff2"),
        ("ibm-plex-sans", "ibm-plex-sans-latin-600-normal.woff2"),
        ("ibm-plex-mono", "ibm-plex-mono-latin-400-normal.woff2"),
    ];
    for (pkg, f) in fonts {
        list.push((
            repo.join(format!("web/node_modules/@fontsource/{pkg}/files/{f}")),
            format!("{ASSETS}/fonts/{f}"),
        ));
    }
    for f in ["tokens.core.css", "tokens.css"] {
        list.push((repo.join("web/src/design").join(f), format!("{ASSETS}/{f}")));
    }
    Ok(list)
}

/// The commit to link to: the given one, else `GITHUB_SHA`, else `git rev-parse HEAD`.
pub fn resolve_commit(repo: &Path, given: Option<&str>) -> Result<String, String> {
    let from_env = std::env::var("GITHUB_SHA").ok();
    let commit = match given
        .filter(|c| !c.is_empty())
        .map(str::to_string)
        .or(from_env.filter(|c| !c.is_empty()))
    {
        Some(c) => c,
        None => {
            let out = std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo)
                .output()
                .map_err(|e| format!("no --commit given and git could not be run: {e}"))?;
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
    };
    if commit.len() != 40 || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "the commit must be a full 40-character hash, got {commit:?}"
        ));
    }
    Ok(commit)
}

/// Renders every document of [`DOCS`] into `cfg.out`, with the list pages, the copied figures and PDFs and the shared assets. Returns an error listing every problem (a
/// link to a file that does not exist, a `$` the parser did not take as mathematics, a document
/// path under prompt/).
pub fn build(cfg: &Config) -> Result<Report, String> {
    let mut problems: Vec<String> = Vec::new();
    let mut report = Report::default();
    for d in DOCS {
        if is_forbidden(d.src) {
            problems.push(format!(
                "{} is under prompt/ and may not be published",
                d.src
            ));
        }
        if !cfg.repo.join(d.src).is_file() {
            problems.push(format!("{} does not exist", d.src));
        }
    }
    if !cfg.out.join("index.html").is_file() {
        problems.push(format!(
            "{} has no index.html: run npm run build in web/ first",
            cfg.out.display()
        ));
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    let version = quoted_value(&read(&cfg.repo.join("Cargo.toml"))?, "version")
        .ok_or("no version in Cargo.toml")?;
    let katex_version = quoted_value(
        &read(&cfg.repo.join("web/node_modules/katex/package.json"))?,
        "version",
    )
    .ok_or("no version in the KaTeX package.json")?;
    let mut indexed: BTreeSet<String> = LISTED_DIRS.iter().map(|s| s.to_string()).collect();
    for d in DOCS {
        if d.src.ends_with("/README.md") {
            indexed.insert(parent(d.src).to_string());
        }
    }
    let site = Site {
        repo: cfg.repo.clone(),
        commit: cfg.commit.clone(),
        version,
        katex_version,
        published: DOCS
            .iter()
            .map(|d| (d.src.to_string(), out_path(d.src)))
            .collect(),
        indexed,
    };

    let mut copied: BTreeSet<String> = BTreeSet::new();
    let mut notes: Vec<String> = Vec::new();
    for d in DOCS {
        let out = out_path(d.src);
        let text = read(&cfg.repo.join(d.src))?;
        let mut link = |dest: &str, _img: bool| {
            site.resolve(d.src, &out, dest, &mut copied, &mut notes, &mut problems)
        };
        let r: Rendered = markdown::render(&text, &mut link);
        for s in &r.stray_dollars {
            problems.push(format!("{}: a $ that is not mathematics: {s}", d.src));
        }
        let source_tags = text.matches("\\tag{").count();
        if source_tags != r.tags_in_math + r.tags_in_code {
            problems.push(format!(
                "{}: {source_tags} \\tag{{ in the source but {} inside mathematics and {} in code",
                d.src, r.tags_in_math, r.tags_in_code
            ));
        }
        let pdf = site.pdf_of(d);
        if let Some(p) = &pdf {
            copied.insert(p.clone());
        }
        let title = if r.title.is_empty() {
            d.title.to_string()
        } else {
            r.title.clone()
        };
        let maths = r.inline_math + r.display_math > 0;
        let html = site.page(
            &out,
            &title,
            &site.doc_meta(d, pdf.as_deref()),
            &r.toc,
            &r.html,
            maths,
        );
        write(&cfg.out.join(&out), &html)?;
        report.lines.push(format!(
            "PAGE {} -> {out} ({} inline and {} display mathematics, {} numbered)",
            d.src, r.inline_math, r.display_math, r.tags_in_math
        ));
    }
    for dir in LISTED_DIRS {
        let (out, html) = site.list_page(dir);
        write(&cfg.out.join(&out), &html)?;
        report.lines.push(format!("LIST {out}"));
    }

    for p in &copied {
        if is_forbidden(p) {
            problems.push(format!("{p} is under prompt/ and may not be published"));
            continue;
        }
        copy(&cfg.repo.join(p), &cfg.out.join(p))?;
        report.lines.push(format!("COPY {p}"));
    }
    for (from, to) in asset_files(&cfg.repo)? {
        copy(&from, &cfg.out.join(&to))?;
    }
    write(
        &cfg.out.join(format!("{ASSETS}/docs.css")),
        include_str!("../assets/docs.css"),
    )?;
    write(
        &cfg.out.join(format!("{ASSETS}/render-math.js")),
        include_str!("../assets/render-math.js"),
    )?;
    report.lines.push(format!(
        "ASSETS {ASSETS}/ (KaTeX {}, fonts, tokens, docs.css, render-math.js)",
        site.katex_version
    ));
    report.lines.extend(notes);
    if problems.is_empty() {
        Ok(report)
    } else {
        Err(problems.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn nothing_under_prompt_is_published() {
        assert!(is_forbidden("prompt"));
        assert!(is_forbidden("prompt/PROGRESS.md"));
        assert!(!is_forbidden("docs/prompt-free.md"));
        for d in DOCS {
            assert!(!is_forbidden(d.src), "{}", d.src);
            assert!(!is_forbidden(&out_path(d.src)), "{}", d.src);
        }
    }

    #[test]
    fn the_readme_changelog_and_development_notes_are_not_published() {
        for d in DOCS {
            assert!(
                d.src.starts_with("docs/mathematics/") || d.src.starts_with("data/reference/"),
                "{}",
                d.src
            );
        }
    }

    #[test]
    fn every_published_document_exists() {
        for d in DOCS {
            assert!(repo().join(d.src).is_file(), "{}", d.src);
        }
    }

    #[test]
    fn page_addresses_keep_the_jekyll_addresses() {
        assert_eq!(
            out_path("docs/mathematics/btcs/02_final_formulation.md"),
            "docs/mathematics/btcs/02_final_formulation.html"
        );
        assert_eq!(
            out_path("data/reference/README.md"),
            "data/reference/index.html"
        );
        assert_eq!(out_path("README.md"), "index.html");
    }

    #[test]
    fn reads_versions() {
        assert_eq!(
            quoted_value(
                "[package]\nname = \"strato\"\nversion = \"2.0.0\"\n",
                "version"
            )
            .as_deref(),
            Some("2.0.0")
        );
        assert_eq!(
            quoted_value(
                "{\n  \"name\": \"katex\",\n  \"version\": \"0.19.1\",\n",
                "version"
            )
            .as_deref(),
            Some("0.19.1")
        );
    }

    fn test_site() -> Site {
        let mut indexed: BTreeSet<String> = LISTED_DIRS.iter().map(|s| s.to_string()).collect();
        indexed.insert("data/reference".to_string());
        Site {
            repo: repo(),
            commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
            version: "2.0.0".to_string(),
            katex_version: "0".to_string(),
            published: DOCS
                .iter()
                .map(|d| (d.src.to_string(), out_path(d.src)))
                .collect(),
            indexed,
        }
    }

    #[test]
    fn links_resolve_to_pages_files_folders_or_github() {
        let s = test_site();
        let src = "docs/mathematics/btcs/03_rust_pseudocode.md";
        let page = out_path(src);
        let (mut copied, mut notes, mut problems) = (BTreeSet::new(), Vec::new(), Vec::new());
        let mut r = |d: &str| s.resolve(src, &page, d, &mut copied, &mut notes, &mut problems);
        assert_eq!(
            r("02_final_formulation.md#notes"),
            Target::Keep("02_final_formulation.html#notes".into())
        );
        assert_eq!(
            r("../STRATO_Phase2_Derivations.pdf"),
            Target::Keep("../STRATO_Phase2_Derivations.pdf".into())
        );
        assert_eq!(
            r("../../../data/reference/"),
            Target::Keep("../../../data/reference/index.html".into())
        );
        assert_eq!(r("./"), Target::Keep("index.html".into()));
        assert_eq!(
            r("../../../.github/workflows/ci.yml"),
            Target::Keep("https://github.com/weewzr/STRATO/blob/0123456789abcdef0123456789abcdef01234567/.github/workflows/ci.yml".into())
        );
        assert_eq!(r("../../../src"), Target::Keep("https://github.com/weewzr/STRATO/tree/0123456789abcdef0123456789abcdef01234567/src".into()));
        assert_eq!(r("../../../prompt/PROGRESS.md"), Target::Text);
        assert_eq!(
            r("https://doi.org/10.1/x"),
            Target::Keep("https://doi.org/10.1/x".into())
        );
        assert_eq!(r("#local"), Target::Keep("#local".into()));
        assert_eq!(r("no_such_file.md"), Target::Text);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert_eq!(notes.len(), 1);
        assert!(copied.contains("docs/mathematics/STRATO_Phase2_Derivations.pdf"));
    }

    /// Every published document converts completely: each `$` is mathematics, each `\tag{` sits
    /// inside mathematics, and every link resolves.
    #[test]
    fn every_published_document_converts_completely() {
        let s = test_site();
        for d in DOCS {
            let text = fs::read_to_string(repo().join(d.src)).unwrap();
            let page = out_path(d.src);
            let (mut copied, mut notes, mut problems) = (BTreeSet::new(), Vec::new(), Vec::new());
            let r = markdown::render(&text, &mut |dest: &str, _| {
                s.resolve(d.src, &page, dest, &mut copied, &mut notes, &mut problems)
            });
            assert!(
                r.stray_dollars.is_empty(),
                "{}: {:?}",
                d.src,
                r.stray_dollars
            );
            assert_eq!(
                text.matches("\\tag{").count(),
                r.tags_in_math + r.tags_in_code,
                "{}",
                d.src
            );
            assert!(problems.is_empty(), "{}: {problems:?}", d.src);
            assert!(!r.title.is_empty(), "{} has a level-1 heading", d.src);
        }
    }
}
