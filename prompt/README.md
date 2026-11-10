# STRATO web demo instruction package

This folder is the complete brief for building the interactive STRATO web demonstration. It is written for an autonomous Claude Code session (run in an authorised collaborator's environment) and for the repository owner. A new session needs only this repository and these files.

To start a session, give it this instruction: **"Read prompt/MASTER_PROMPT.md and follow it."**

## Reading order

| File | What it holds |
|---|---|
| `MASTER_PROMPT.md` | Mission, authority order, non-negotiables, start-of-session procedure, phase loop, end-of-session procedure. |
| `PROGRESS.md` | Current phase, milestones, commits, tests run, blockers, next actions, the owner decisions table D1 to D14 and the session log. Read it second, every time. |
| `SCIENTIFIC_REQUIREMENTS.md` | The model, implemented versus planned physics, the Phase 6 gate, parameter provenance, verification versus validation, result labels, forbidden actions. |
| `TECHNICAL_ARCHITECTURE.md` | The hybrid architecture (client-side wasm plus precomputed native results), the wasm wrapper API, frontend stack, store, worker protocol, equation term model, performance budget, CI. |
| `DESIGN_SPECIFICATION.md` | Page structure, every panel and its states, the growing equation explorer, the particle views, the two independently toggleable graphs, design system and accessibility. |
| `IMPLEMENTATION_ROADMAP.md` | Phases 0 to 8 with objectives, files, dependencies, acceptance criteria, tests, scientific checks, commit messages and conditions for proceeding. |
| `TESTING_AND_VALIDATION.md` | Existing v1.0 suite, tolerance policy, new tests per phase, browser self-test, Phase 2 verification cases, verification report, validation protocol. |
| `AUTONOMOUS_EXECUTION.md` | Pre-authorised actions and their limits, a suggested permission allow list, the phase loop, git and scientific safety, boundaries, subagent roles. |
| `COMMIT_AND_DEPLOYMENT.md` | Branches, commit identity, milestone messages, pre-commit checklist, pull request, GitHub Pages, releases. |
| `GITHUB_REFERENCES.md` | Every reference repository with licence and verdict, the Outram Park section, a do-not-install list and the research notes. |
| `RAYSTORAGE_INTEGRATION.md` | Engineering practices taken from NUS-H2 and the raystorage archive, with file paths, priorities, risks and the scientific independence rules. |

## Owner decisions

Web Phase 6 (decay, source, inventory, Arrhenius, temperature histories) waits for the owner. To unblock it, the owner fills in the "Owner choice" and "Date" columns of the table in `PROGRESS.md` for every row D1 to D14 and pushes the change. The options are explained in `docs/development/phase2_review_package.md` Section 7.

## Quick reference

| Item | Value |
|---|---|
| Work branch | `feat/strato-interactive-web-demo` |
| Commit identity | `weewzr <288510855+weewzr@users.noreply.github.com>` |
| Rust checks | `cargo test --all-targets`; `cargo run --release --example verify` |
| Wrapper checks | `cargo test --manifest-path crates/strato-wasm/Cargo.toml` |
| Web checks | in `web/`: `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`, `npx playwright test` |
| Labels | `COMPUTED v1.0`, `COMPUTED v2.0`, `PRECOMPUTED`, `ILLUSTRATIVE`, `MEASURED` |
| Pinned references | Outram Park `0efd055`, NUSH2 `b4817b3`, raystorage `3a25c4a` |
| Licence | AGPL-3.0-only |

| Phase | Commit message |
|---|---|
| 0 | `docs(prompt): phase 0 - audit and architecture confirmation` |
| 1 | `feat(web): phase 1 - dashboard foundation` |
| 2 | `feat(web): phase 2 - interactive equation explorer` |
| 3 | `feat(web): phase 3 - rust solver integration` |
| 4 | `feat(web): phase 4 - triso particle visualisation` |
| 5 | `feat(web): phase 5 - scientific comparison graphs` |
| 6 | `feat(physics): phase 6 - verified model extensions` (gated) |
| 7 | `feat(validation): phase 7 - literature comparison` |
| 8 | `feat(web): phase 8 - deployment and polish` |

## Status of this package

Written on 2026-11-10 from the repository at commit `8c44fb7` and from research into the reference repositories. A full line-by-line audit of the repository is Phase 0 of the roadmap; the session doing Phase 0 corrects any fact here that turns out wrong.
