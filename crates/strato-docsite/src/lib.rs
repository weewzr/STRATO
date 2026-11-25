//! strato-docsite renders the derivation and register documents of the STRATO repository into the
//! web build and checks every link of the built site.
//!
//! The Markdown is converted with pulldown-cmark. The mathematics stays TeX inside
//! `<span class="math ...">` elements and is typeset in the browser by the KaTeX copy that the web
//! demonstration already ships (copied beside the pages, so no third-party request is made). The
//! tool computes nothing scientific: it converts text, rewrites links and copies files.
//!
//! What is published is fixed in [`site::DOCS`]; nothing under `prompt/` can be published
//! ([`site::is_forbidden`]), and the README, the CHANGELOG and the development notes are not part of
//! the list (owner decisions M3 and M5: the text about the internal comparison stays off the site).

pub mod check;
pub mod markdown;
pub mod paths;
pub mod site;
