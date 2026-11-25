# Owner request: Rust-first STRATO

This is the owner's brief of 2026-11-25 for making Rust the dominant language of STRATO by computational responsibility, kept verbatim for reference, followed by the two lines the owner gave with it. The inventory and the migration plan that follow from it are in `docs/development/rust_migration_plan.md`. Later owner decisions in `prompt/PROGRESS.md` and the other `prompt/` files take precedence where they differ.

---

For STRATO, I want Rust to become the dominant language of the codebase, not merely by GitHub language statistics but by actual computational responsibility.

Continue the planned Rust migration, including data generation, numerical tooling, validation, non-rendering web logic and other suitable Python/TypeScript functionality.

Target at least 60–70% Rust by GitHub language statistics where practical, without compromising maintainability, numerical correctness or the existing interface.

However, preserve a small, genuinely independent Python verification suite where it provides meaningful cross-language validation of the Rust numerical solver. Do not port independent reference implementations solely to inflate Rust's percentage.

Keep React, TypeScript, KaTeX, Plotly and three.js where appropriate for the interface.

Maintain byte-identical generated outputs where required, preserve existing validated numerical behaviour, and run regression and conservation tests throughout the migration.

After each migration stage, report the updated language percentages, the functionality moved into Rust, the tests performed and any remaining migration opportunities.

The ultimate goal is a Rust-first scientific computing architecture, not simply a repository that appears Rust-heavy.

Owner's own lines with the brief:

- "the framework can be in javascript yes for the interface"
- "for the python derivation checks, sure"
