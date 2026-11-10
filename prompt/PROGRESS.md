# Progress

Update this file at the end of every phase and every session. The next session resumes from here.

## Current state

| Item | Value |
|---|---|
| Current phase | Phase 0, not started |
| Work branch | `feat/strato-interactive-web-demo` (not yet created) |
| Latest commit | the commit that added `prompt/` on `main` (fill in its hash in Phase 0 from `git log -1 -- prompt/`) |
| STRATO version | 1.0.1 (licence AGPL-3.0-only since commit `8c44fb7`) |

## Completed milestones (before the web demo)

| Milestone | Reference |
|---|---|
| v1.0 Phase 1 baseline, verified | tag `v1.0.0` (commit `8c5d8a8`) |
| v1.0.0 mathematical documentation | tag `v1.0.1` (commit `e6d0f60`) |
| Phase 2 derivations, pseudocode, review package, parameter register | commit `eac2806` |
| Relicence to AGPL-3.0-only | commit `8c44fb7` |
| Web demo instruction package | the commit that added `prompt/` |

## Web demo phases

| Phase | Status | Commit | Tests run | Scientific checks |
|---|---|---|---|---|
| 0 Audit and architecture | not started | | | |
| 1 Frontend foundation | not started | | | |
| 2 Equation explorer | not started | | | |
| 3 v1.0 solver integration | not started | | | |
| 4 Particle visualisation | not started | | | |
| 5 Scientific graphs | not started | | | |
| 6 Phase 2 physics | BLOCKED (owner decisions) | | | |
| 7 Validation | not started | | | |
| 8 Polish and deployment | not started | | | |

## Known blockers

| Id | Blocker | Needed from |
|---|---|---|
| B1 | Owner decisions D1 to D14 pending; web Phase 6 and the prediction part of Phase 7 cannot start | owner |
| B2 | GitHub Pages must be enabled (Settings, Pages, Source: GitHub Actions) and deployment needs a merge into `main` | owner |
| B3 | Push access from the collaborator's environment to `weewzr/STRATO` must be confirmed (collaborator access or a fine-grained token chosen by the owner) | owner and collaborator |
| B4 | Open scientific items from the review package: CRP-6 gas constant and decay treatment; data gaps O1, O7, O8, O9; no stiff-regime staircase proof | owner, later phases |

## Remaining work

All phases 0 to 8.

## Next actions

1. Phase 0: create the work branch, run the v1.0 suite and the wasm check, confirm the facts in `prompt/`.
2. Phase 1.

## Owner decisions

Only the repository owner edits the "Owner choice" and "Date" columns. The agent reads them and never fills them in. Proposed defaults come from `docs/development/phase2_review_package.md` Section 7 and are not approved by being listed here.

| Id | Topic | Options | Proposed default | Owner choice | Date |
|---|---|---|---|---|---|
| D1 | Nuclides first | any of Cs-137, Sr-90, Kr-85, I-131, Ag-110m | Cs-137, Sr-90, Kr-85, I-131; Ag-110m once activation data exist | PENDING | |
| D2 | Yields | cumulative $y^C$ or independent $y^I$ | $y^C$ only, keep the precursor warning | PENDING | |
| D3 | Activation products | FUTURE, or equations without data | FUTURE | PENDING | |
| D4 | Diffusivity set and gas constant | TECDOC-978 FRG, Collin et al. (2016), Hales et al. (2021); SI or per-row $R_g$ | owner to choose set; SI with `FIXED_8_3143` option | PENDING | |
| D5 | Temperature histories | AGR-1 plans, CRP-6 schedules, or both; history format; reference temperature | highest temperature of the history | PENDING | |
| D6 | Decay time integration | Backward Euler with implicit decay (P21.58) or exponential split (P21.66) | Backward Euler; outside inventory by (P21.73) | PENDING | |
| D7 | Modal projection under piecewise temperature | in scope or not; accessors or a copy of shooting code; staircase as test helper | owner to choose | PENDING | |
| D8 | Conductance time level | option A (step average) or E (end of step) | option A | PENDING | |
| D9 | Source region | single-region particle or per-region source flag | owner to choose | PENDING | |
| D10 | Source input | fission rate density only, or burnup or power | fission rate density only | PENDING | |
| D11 | Source step average | exact average (A) only, or keep rectangle (R) | exact average only | PENDING | |
| D12 | Step limits | hard caps or warnings | owner to choose | PENDING | |
| D13 | Modal block R1 scope | kernel-uniform source and start profile only | kernel-uniform only, general projection FUTURE | PENDING | |
| D14 | Step planner | merged planner of pseudocode block F | accept | PENDING | |

## Pinned external references

| Repository | Commit |
|---|---|
| theodoreOnzGit/outram-park-backend | `0efd0553310db6088052fef8b79be69518913ede` (main, 2026-11-10) |
| weewzr/NUSH2 | `b4817b305de5bfada5da147802cbf3aeffb9f4c1` |
| weewzr/raystorage | `3a25c4a767b67faa3e797944ea9ac63f9265d4a1` |

## Session log

| Date | Session | Phases worked | Commits | Notes |
|---|---|---|---|---|
| 2026-11-10 | package author | none (instructions only) | the `prompt/` commit | package written; deep audit left to Phase 0 |
