//! Markdown to HTML for one document.
//!
//! pulldown-cmark parses the text (CommonMark with pipe tables, footnotes, strikethrough and `$` /
//! `$$` mathematics, the syntax the documents are written in for pandoc's `tex_math_dollars`). The
//! events are then adjusted before the HTML writer runs:
//!
//! - headings get GitHub's anchor ids, so a link such as `...md#5-finite-volume-discretisation`
//!   that works on GitHub also works on the site;
//! - a paragraph that holds only an image becomes a figure with its caption (pandoc's
//!   `implicit_figures`, which the PDFs use);
//! - every link and image goes through the caller's resolver (published document, copied file,
//!   GitHub page at the build commit, or plain text);
//! - a fenced block with the language `math` becomes display mathematics (GitHub's syntax, used by
//!   the Phase 1 derivation);
//! - every table sits in a box that scrolls sideways, so the page itself never does.
//!
//! The mathematics is passed through as TeX; nothing is evaluated.

use pulldown_cmark::{
    html, CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, TextMergeStream,
};
use std::collections::HashMap;

/// What a link of a document becomes on the site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A link with this address.
    Keep(String),
    /// No link: the link text is kept as plain text.
    Text,
}

/// The result of rendering one document.
#[derive(Debug, Default)]
pub struct Rendered {
    /// The HTML of the document body.
    pub html: String,
    /// The text of the first level-1 heading.
    pub title: String,
    /// Anchor id and HTML label of every level-2 heading, in order.
    pub toc: Vec<(String, String)>,
    /// Number of inline mathematics spans.
    pub inline_math: usize,
    /// Number of display mathematics blocks (including fenced `math` blocks).
    pub display_math: usize,
    /// Number of `\tag{` inside the mathematics (numbered equations).
    pub tags_in_math: usize,
    /// Number of `\tag{` written as code (inline code or a code block), which is text, not an
    /// equation number.
    pub tags_in_code: usize,
    /// Text outside code and mathematics that still holds a `$` (mathematics the parser did not
    /// recognise); each entry is a short excerpt.
    pub stray_dollars: Vec<String>,
}

/// The Markdown extensions the documents use.
pub fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
}

/// HTML escaping of text and attribute values.
pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            _ => o.push(c),
        }
    }
    o
}

/// GitHub's heading anchor before de-duplication: lower case, every character that is not a
/// letter, a digit, a space, `_` or `-` removed, each white-space character replaced by `-` (the
/// rule of `headingSlug` in web/src/equation/links.ts).
pub fn slug_base(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.trim().to_lowercase().chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            s.push(c);
        } else if c.is_whitespace() {
            s.push('-');
        }
    }
    s
}

/// Unique heading anchors within one page: the second heading with the same text gets `-1`, the
/// third `-2`, as on GitHub.
#[derive(Default)]
pub struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let mut base = slug_base(text);
        if base.is_empty() {
            base = "section".to_string();
        }
        let n = self.seen.entry(base.clone()).or_insert(0);
        let id = if *n == 0 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        *n += 1;
        id
    }
}

struct State {
    slugger: Slugger,
    title: String,
    toc: Vec<(String, String)>,
    inline_math: usize,
    display_math: usize,
    tags_in_math: usize,
    tags_in_code: usize,
    stray: Vec<String>,
}

/// Renders `text` (Markdown). `link(destination, is_image)` decides what each link becomes.
pub fn render(text: &str, link: &mut dyn FnMut(&str, bool) -> Target) -> Rendered {
    let text = text.replace("\r\n", "\n");
    let events: Vec<Event<'_>> = TextMergeStream::new(Parser::new_ext(&text, options())).collect();
    let mut st = State {
        slugger: Slugger::default(),
        title: String::new(),
        toc: Vec::new(),
        inline_math: 0,
        display_math: 0,
        tags_in_math: 0,
        tags_in_code: 0,
        stray: Vec::new(),
    };
    let out = transform(&events, &mut st, link);
    let mut body = String::with_capacity(text.len() * 2);
    html::push_html(&mut body, out.into_iter());
    Rendered {
        html: body,
        title: st.title,
        toc: st.toc,
        inline_math: st.inline_math,
        display_math: st.display_math,
        tags_in_math: st.tags_in_math,
        tags_in_code: st.tags_in_code,
        stray_dollars: st.stray,
    }
}

/// Index of the `End` event that closes the `Start` event at `start`.
fn find_end(ev: &[Event<'_>], start: usize) -> usize {
    let mut depth = 0usize;
    for (k, e) in ev.iter().enumerate().skip(start) {
        match e {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return k;
                }
            }
            _ => {}
        }
    }
    ev.len().saturating_sub(1)
}

/// The text of a run of events (code and mathematics included as their source text).
fn plain_text(ev: &[Event<'_>]) -> String {
    let mut s = String::new();
    for e in ev {
        match e {
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                s.push_str(t)
            }
            Event::SoftBreak | Event::HardBreak => s.push(' '),
            _ => {}
        }
    }
    s
}

/// The label of a heading for the table of contents: text escaped, mathematics kept as a
/// mathematics span for the browser to typeset, links and emphasis dropped.
fn heading_label(ev: &[Event<'_>]) -> String {
    let mut s = String::new();
    for e in ev {
        match e {
            Event::Text(t) => s.push_str(&esc(t)),
            Event::Code(t) => {
                s.push_str("<code>");
                s.push_str(&esc(t));
                s.push_str("</code>");
            }
            Event::InlineMath(t) => {
                s.push_str("<span class=\"math math-inline\">");
                s.push_str(&esc(t));
                s.push_str("</span>");
            }
            Event::SoftBreak | Event::HardBreak => s.push(' '),
            _ => {}
        }
    }
    s.trim().to_string()
}

fn is_blank(e: &Event<'_>) -> bool {
    match e {
        Event::SoftBreak => true,
        Event::Text(t) => t.trim().is_empty(),
        _ => false,
    }
}

/// For a paragraph at `i` that holds one image and nothing else: (image start, image end,
/// paragraph end).
fn lone_image(ev: &[Event<'_>], i: usize) -> Option<(usize, usize, usize)> {
    let mut j = i + 1;
    while j < ev.len() && is_blank(&ev[j]) {
        j += 1;
    }
    if !matches!(ev.get(j), Some(Event::Start(Tag::Image { .. }))) {
        return None;
    }
    let img_end = find_end(ev, j);
    let mut k = img_end + 1;
    while k < ev.len() && is_blank(&ev[k]) {
        k += 1;
    }
    if matches!(ev.get(k), Some(Event::End(TagEnd::Paragraph))) {
        Some((j, img_end, k))
    } else {
        None
    }
}

fn excerpt(t: &str) -> String {
    let at = t.find('$').unwrap_or(0);
    let start = t[..at]
        .char_indices()
        .rev()
        .nth(30)
        .map(|(k, _)| k)
        .unwrap_or(0);
    let tail: String = t[start..].chars().take(70).collect();
    tail.replace('\n', " ")
}

fn title_attr(title: &str) -> String {
    if title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", esc(title))
    }
}

fn link_target(link: &mut dyn FnMut(&str, bool) -> Target, dest: &str, is_image: bool) -> String {
    match link(dest, is_image) {
        Target::Keep(s) => s,
        Target::Text => String::new(),
    }
}

fn count_math(st: &mut State, tex: &str, display: bool) {
    if display {
        st.display_math += 1;
    } else {
        st.inline_math += 1;
    }
    st.tags_in_math += tex.matches("\\tag{").count();
}

fn transform<'a>(
    ev: &[Event<'a>],
    st: &mut State,
    link: &mut dyn FnMut(&str, bool) -> Target,
) -> Vec<Event<'a>> {
    let mut out: Vec<Event<'a>> = Vec::with_capacity(ev.len() + 16);
    let mut dropped_links: Vec<bool> = Vec::new();
    let mut in_code_block = false;
    let mut i = 0;
    while i < ev.len() {
        if matches!(ev[i], Event::Start(Tag::Paragraph)) {
            if let Some((img, img_end, para_end)) = lone_image(ev, i) {
                if let Event::Start(Tag::Image {
                    dest_url, title, ..
                }) = &ev[img]
                {
                    let caption = &ev[img + 1..img_end];
                    let src = link_target(link, dest_url, true);
                    out.push(Event::Html("<figure>\n".into()));
                    out.push(Event::Html(
                        format!(
                            "<img src=\"{}\" alt=\"{}\"{} loading=\"lazy\">\n",
                            esc(&src),
                            esc(plain_text(caption).trim()),
                            title_attr(title)
                        )
                        .into(),
                    ));
                    out.push(Event::Html("<figcaption>".into()));
                    out.extend(transform(caption, st, link));
                    out.push(Event::Html("</figcaption>\n</figure>\n".into()));
                    i = para_end + 1;
                    continue;
                }
            }
        }
        match &ev[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                let end = find_end(ev, i);
                let inner = &ev[i + 1..end];
                let text = plain_text(inner);
                let id = st.slugger.slug(&text);
                if *level == HeadingLevel::H1 && st.title.is_empty() {
                    st.title = text.trim().to_string();
                }
                if *level == HeadingLevel::H2 {
                    st.toc.push((id.clone(), heading_label(inner)));
                }
                out.push(Event::Start(Tag::Heading {
                    level: *level,
                    id: Some(CowStr::from(id)),
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }));
            }
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                let end = find_end(ev, i);
                let src = link_target(link, dest_url, true);
                out.push(Event::InlineHtml(
                    format!(
                        "<img src=\"{}\" alt=\"{}\"{} loading=\"lazy\">",
                        esc(&src),
                        esc(plain_text(&ev[i + 1..end]).trim()),
                        title_attr(title)
                    )
                    .into(),
                ));
                i = end + 1;
                continue;
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => match link(dest_url, false) {
                Target::Keep(url) => {
                    out.push(Event::Start(Tag::Link {
                        link_type: *link_type,
                        dest_url: CowStr::from(url),
                        title: title.clone(),
                        id: id.clone(),
                    }));
                    dropped_links.push(false);
                }
                Target::Text => dropped_links.push(true),
            },
            Event::End(TagEnd::Link) => {
                if !dropped_links.pop().unwrap_or(false) {
                    out.push(Event::End(TagEnd::Link));
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) if lang.trim() == "math" => {
                let end = find_end(ev, i);
                let tex = plain_text(&ev[i + 1..end]);
                count_math(st, &tex, true);
                out.push(Event::Html(
                    format!(
                        "<p class=\"math-fence\"><span class=\"math math-display\">{}</span></p>\n",
                        esc(tex.trim_end())
                    )
                    .into(),
                ));
                i = end + 1;
                continue;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                out.push(ev[i].clone());
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                out.push(ev[i].clone());
            }
            Event::Start(Tag::Table(_)) => {
                out.push(Event::Html(
                    "<div class=\"table-wrap\" role=\"region\" tabindex=\"0\" aria-label=\"Table, scrolls sideways when wider than the page\">\n".into(),
                ));
                out.push(ev[i].clone());
            }
            Event::End(TagEnd::Table) => {
                out.push(ev[i].clone());
                out.push(Event::Html("</div>\n".into()));
            }
            Event::InlineMath(t) => {
                count_math(st, t, false);
                out.push(ev[i].clone());
            }
            Event::DisplayMath(t) => {
                count_math(st, t, true);
                out.push(ev[i].clone());
            }
            Event::Text(t) => {
                if in_code_block {
                    st.tags_in_code += t.matches("\\tag{").count();
                } else if t.contains('$') {
                    st.stray.push(excerpt(t));
                }
                out.push(ev[i].clone());
            }
            Event::Code(t) => {
                st.tags_in_code += t.matches("\\tag{").count();
                out.push(ev[i].clone());
            }
            _ => out.push(ev[i].clone()),
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keep_all(dest: &str, _img: bool) -> Target {
        Target::Keep(dest.to_string())
    }

    #[test]
    fn github_heading_anchors() {
        assert_eq!(
            slug_base("5. Finite-volume discretisation"),
            "5-finite-volume-discretisation"
        );
        assert_eq!(
            slug_base("Step 3: the R2.4 correction (D16)"),
            "step-3-the-r24-correction-d16"
        );
        assert_eq!(slug_base("  Code B vs Code C  "), "code-b-vs-code-c");
        let mut s = Slugger::default();
        assert_eq!(s.slug("Notes"), "notes");
        assert_eq!(s.slug("Notes"), "notes-1");
        assert_eq!(s.slug("Notes"), "notes-2");
        assert_eq!(s.slug("***"), "section");
    }

    #[test]
    fn mathematics_stays_tex_for_the_browser() {
        let r = render("Flux $J = -D\\,\\partial c/\\partial r$ here.\n\n$$\nF(t) = 1 - \\sum_n a_n\n\\tag{1.1}\n$$\n", &mut keep_all);
        assert!(
            r.html.contains(
                "<span class=\"math math-inline\">J = -D\\,\\partial c/\\partial r</span>"
            ),
            "{}",
            r.html
        );
        assert!(
            r.html.contains("<span class=\"math math-display\">"),
            "{}",
            r.html
        );
        assert!(r.html.contains("\\tag{1.1}"));
        assert_eq!((r.inline_math, r.display_math, r.tags_in_math), (1, 1, 1));
        assert!(r.stray_dollars.is_empty());
    }

    #[test]
    fn tex_is_escaped_not_interpreted() {
        let r = render("$a<b>c$ and $x_1 * y_2 * z$\n", &mut keep_all);
        assert!(r.html.contains("a&lt;b&gt;c"));
        assert!(r.html.contains("x_1 * y_2 * z"));
        assert!(!r.html.contains("<em>"));
    }

    #[test]
    fn fenced_math_block_becomes_display_mathematics() {
        let r = render(
            "```math\nc(r,0)=c_0\n```\n\n```text\n$ not math\n```\n",
            &mut keep_all,
        );
        assert!(
            r.html.contains(
                "<p class=\"math-fence\"><span class=\"math math-display\">c(r,0)=c_0</span></p>"
            ),
            "{}",
            r.html
        );
        assert!(r.html.contains("<code class=\"language-text\">$ not math"));
        assert_eq!(r.display_math, 1);
        assert!(
            r.stray_dollars.is_empty(),
            "a $ in a code block is not stray"
        );
    }

    #[test]
    fn unrecognised_dollar_is_reported() {
        let r = render("costs $ 5 and $ 6\n", &mut keep_all);
        assert_eq!(r.stray_dollars.len(), 1);
    }

    #[test]
    fn image_alone_in_a_paragraph_is_a_figure() {
        let r = render(
            "![Figure 1.1: Cross-section with $R_1$.](figures/x.png)\n",
            &mut |d, img| {
                assert!(img);
                Target::Keep(format!("site/{d}"))
            },
        );
        assert!(r.html.contains("<figure>"), "{}", r.html);
        assert!(r.html.contains("src=\"site/figures/x.png\""));
        assert!(r
            .html
            .contains("alt=\"Figure 1.1: Cross-section with R_1.\""));
        assert!(r.html.contains("<figcaption>Figure 1.1: Cross-section with <span class=\"math math-inline\">R_1</span>.</figcaption>"), "{}", r.html);
    }

    #[test]
    fn links_go_through_the_resolver() {
        let r = render(
            "See [the deliverable](B.md#notes) and [the package](../prompt/x.md).\n",
            &mut |d, _| {
                if d.contains("prompt/") {
                    Target::Text
                } else {
                    Target::Keep(d.replace(".md", ".html"))
                }
            },
        );
        assert!(
            r.html
                .contains("<a href=\"B.html#notes\">the deliverable</a>"),
            "{}",
            r.html
        );
        assert!(r.html.contains("and the package."), "{}", r.html);
        assert!(!r.html.contains("prompt/"));
    }

    #[test]
    fn headings_get_ids_title_and_contents() {
        let r = render(
            "# The title\n\n## 1. First $D_i$\n\n## Notes\n\n### Notes\n",
            &mut keep_all,
        );
        assert_eq!(r.title, "The title");
        assert!(r.html.contains("<h2 id=\"1-first-d_i\">"), "{}", r.html);
        assert!(r.html.contains("<h3 id=\"notes-1\">"), "{}", r.html);
        assert_eq!(r.toc.len(), 2);
        assert_eq!(
            r.toc[0].1,
            "1. First <span class=\"math math-inline\">D_i</span>"
        );
    }

    #[test]
    fn tables_scroll_in_their_own_box() {
        let r = render("| a | b |\n|---|---|\n| $x\\|y$ | 2 |\n", &mut keep_all);
        assert!(r.html.contains("<div class=\"table-wrap\""));
        assert!(r.html.contains("</table>\n</div>"), "{}", r.html);
        assert!(
            r.html
                .contains("<span class=\"math math-inline\">x|y</span>"),
            "{}",
            r.html
        );
    }
}
