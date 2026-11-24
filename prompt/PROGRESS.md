# Progress

Update this file at the end of every phase and every session. The next session resumes from here.

## Current state

| Item | Value |
|---|---|
| Current phase | Phase 0, in progress (policy and decisions recorded; baseline below) |
| Work branch | `feat/strato-interactive-web-demo`, created 2026-11-24 from `main` at `e3a558f` |
| Latest commit on `main` | `e3a558f` (2026-11-24). The last commit touching `prompt/` on `main` is `7fe4421` |
| STRATO version | 1.0.1 (licence AGPL-3.0-only since commit `8c44fb7`) |

## Completed milestones (before the web demo)

| Milestone | Reference |
|---|---|
| v1.0 Phase 1 baseline, verified | tag `v1.0.0` (commit `8c5d8a8`) |
| v1.0.0 mathematical documentation | tag `v1.0.1` (commit `e6d0f60`) |
| Phase 2 derivations, pseudocode, review package, parameter register | commit `eac2806` |
| Relicence to AGPL-3.0-only | commit `8c44fb7` |
| Web demo instruction package | commits `3a4319e`, `bc107b1` and `7fe4421` (2026-11-10) |
| Phase 2 open maths items O4, O10, O11, O12, R2.4, P2; equations to (P21.142) and (P22.142); decisions D15 and D16 added | commit `9802b2d` (2026-11-13) |

## Web demo phases

| Phase | Status | Commit | Tests run | Scientific checks |
|---|---|---|---|---|
| 0 Audit and architecture | not started | | | |
| 1 Frontend foundation | not started | | | |
| 2 Equation explorer | not started | | | |
| 3 v1.0 solver integration | not started | | | |
| 4 Particle visualisation | not started | | | |
| 5 Scientific graphs | not started | | | |
| 6.1 Phase 2.1 physics (decay, source, inventory) | not started; gate open (D1, D2, D3, D6, D9 to D16 decided 2026-11-24) | | | |
| 6.2 Phase 2.2 physics (temperature-dependent diffusivity) | not started; gate open (D4, D5, D7, D8 decided 2026-11-24; D5 sub-items open, see the table); starts after 6.1 passes | | | |
| 7 Validation | not started | | | |
| 8 Polish and deployment | not started | | | |

## Known blockers

| Id | Blocker | Needed from |
|---|---|---|
| B1 | RESOLVED 2026-11-24. Owner decisions D1 to D16 recorded below; the Phase 6 gate is split into 6.1 and 6.2. Remaining sub-items (D5 history format and reference temperature) are open questions for Phase 6.2, not a blocker for 6.1 | owner |
| B2 | OPEN. The Pages source must be switched from "Deploy from a branch" to "GitHub Actions" (Settings, Pages, Source). Until then the live site stays the Jekyll render of `main`. Merging into `main` is done by the gated procedure of `prompt/COMMIT_AND_DEPLOYMENT.md` Section 6 | owner |
| B3 | RESOLVED 2026-11-24. Pushing the work branch to `weewzr/STRATO` works from the development environment | none |
| B4 | Open scientific items from the review package: CRP-6 gas constant and decay treatment; data gaps O1, O7, O8, O9; no stiff-regime staircase proof | owner, later phases |

## Remaining work

Phase 0 (remaining audit items: public API against `prompt/TECHNICAL_ARCHITECTURE.md` Section 3, cross-project review) and phases 1 to 8.

## Next actions

1. Phase 0: confirm the public API in `src/` against `prompt/TECHNICAL_ARCHITECTURE.md` Section 3, confirm the remaining facts in `prompt/`, and do the time-bounded cross-project review of `prompt/RAYSTORAGE_INTEGRATION.md` Section 4 (files) and Section 8 (effect on the roadmap).
2. Phase 1.

## Corrections made on 2026-11-24 (Stage A audit)

- `prompt/TESTING_AND_VALIDATION.md` Section 1 said CI compares the synthetic-run outputs with `tests/golden/v1.0/`. It does not: CI only uploads them. The golden regression test asserts the time series, the final profile and the analytical series; `convergence_study.csv` and the two stdout files are asserted by no test. The text now says so.
- `prompt/SCIENTIFIC_REQUIREMENTS.md` gave the Phase 2 equation ranges as (P21.1) to (P21.92) and (P22.1) to (P22.59). Since `9802b2d` they run to (P21.142) and (P22.142) (counted from the `\tag` entries of `docs/mathematics/STRATO_Phase2_Derivations.md`).
- `prompt/README.md` said the package was written from `8c44fb7` without naming the package commits; it now names `3a4319e`, `bc107b1`, `7fe4421` and this update.
- This file had placeholder commits ("the commit that added `prompt/`"); they are filled in above.
- D15 and D16 (added to the review package on 2026-11-13) were missing from the decision table and from every gate reference; the gate now reads D1 to D16 everywhere.
- `prompt/IMPLEMENTATION_ROADMAP.md` Phase 4 and Phase 5 computed layer inventory shares and the difference curve in TypeScript; both now come from the Rust wrapper (M4).
- `prompt/COMMIT_AND_DEPLOYMENT.md` asked for an assistant attribution trailer; replaced by owner-only attribution (M2).
- Noted, not changed (frozen `src/`): `src/lib.rs` line 13 cites `docs/mathematics/STRATO_Mathematical_Derivations.md`, which does not exist.
- Noted, not changed (outside `prompt/`): nine scripts under `scripts/derivation_checks/` and `scripts/figures/` carry a local absolute virtual-environment path in their docstrings, against the package's own privacy rule.

## Owner decisions

Only the repository owner decides the "Owner choice" and "Date" columns. An agent never chooses them; it may write them only to record a choice the owner has made, naming the source. Proposed defaults come from `docs/development/phase2_review_package.md` Section 7 and are not approved by being listed here. The choices below were made by the owner at the Stage A checkpoint and recorded on the owner's instruction.

Gate split (owner, 2026-11-24): web Phase 6.1 (scientific Phase 2.1: decay, source, inventory) needs D1, D2, D3, D6, D9, D10, D11, D12, D13, D14, D15 and D16; web Phase 6.2 (scientific Phase 2.2: temperature-dependent diffusivity and temperature histories) also needs D4, D5, D7 and D8.

| Id | Topic | Options | Proposed default | Owner choice | Date | Source |
|---|---|---|---|---|---|---|
| D1 | Nuclides first | any of Cs-137, Sr-90, Kr-85, I-131, Ag-110m | Cs-137, Sr-90, Kr-85, I-131; Ag-110m once activation data exist | Cs-137 and Sr-90 first; Kr-85 second, with the precursor-delay caveat; I-131 deferred (no PyC or SiC diffusivity in the register); Ag-110m later | 2026-11-24 | owner, Stage A checkpoint |
| D2 | Yields | cumulative $y^C$ or independent $y^I$ | $y^C$ only, keep the precursor warning | cumulative yields $y^C$ only | 2026-11-24 | owner, Stage A checkpoint |
| D3 | Activation products | FUTURE, or equations without data | FUTURE | later; not in v2.0 (stay FUTURE) | 2026-11-24 | owner, Stage A checkpoint |
| D4 | Diffusivity set and gas constant | TECDOC-978 FRG, Collin et al. (2016), Hales et al. (2021); SI or per-row $R_g$ | owner to choose set; SI with `FIXED_8_3143` option | TECDOC-978 FRG two-branch diffusivity rows; SI gas constant by default with a selectable 8.3143 option (`FIXED_8_3143`); AGR-1 correction factors are never shipped as defaults | 2026-11-24 | owner, Stage A checkpoint |
| D5 | Temperature histories | AGR-1 plans, CRP-6 schedules, or both; history format; reference temperature | highest temperature of the history | ship both the reconstructed AGR-1 safety-test plans and the CRP-6 schedules, each labelled with its evidence class. Not decided by this choice: the history format (one kind with repeated times or two kinds) and the reference temperature of the effective time; the proposal stays a proposal until the owner settles them before Phase 6.2 code needs them | 2026-11-24 | owner, Stage A checkpoint |
| D6 | Decay time integration | Backward Euler with implicit decay (P21.58) or exponential split (P21.66) | Backward Euler; outside inventory by (P21.73) | unsplit Backward Euler with implicit decay (P21.58) | 2026-11-24 | owner, Stage A checkpoint |
| D7 | Modal projection under piecewise temperature | in scope or not; accessors or a copy of shooting code; staircase as test helper | owner to choose | block R3 in scope as a verification reference, through read-only accessors to the shooting layer states; the staircase only as a test helper | 2026-11-24 | owner, Stage A checkpoint |
| D8 | Conductance time level | option A (step average) or E (end of step) | option A | option A (step-average diffusivity) as default, option E selectable | 2026-11-24 | owner, Stage A checkpoint |
| D9 | Source region | single-region particle or per-region source flag | owner to choose | configurable region count 1 to 5, with a kernel-only source as default; the change to `Config::validate` is made under the full v1.0 regression suite | 2026-11-24 | owner, Stage A checkpoint |
| D10 | Source input | fission rate density only, or burnup or power | fission rate density only | fission rate density is the only source input | 2026-11-24 | owner, Stage A checkpoint |
| D11 | Source step average | exact average (A) only, or keep rectangle (R) | exact average only | exact step-average source (A) only | 2026-11-24 | owner, Stage A checkpoint |
| D12 | Step limits | hard caps or warnings | owner to choose | hard caps in the adaptive planner; warnings in FIXED mode | 2026-11-24 | owner, Stage A checkpoint |
| D13 | Modal block R1 scope | kernel-uniform source and start profile only | kernel-uniform only, general projection FUTURE | kernel-uniform source and start profile only | 2026-11-24 | owner, Stage A checkpoint |
| D14 | Step planner | merged planner of pseudocode block F | accept | the merged step planner of pseudocode block F | 2026-11-24 | owner, Stage A checkpoint |
| D15 | Integrator (review package item 15, added 2026-11-13) | Backward Euler only; Backward Euler with TR-BDF2; Backward Euler with the exponential integrator | Backward Euler only for v2.0 (Stage A recommendation) | Backward Euler as the only integrator | 2026-11-24 | owner, Stage A checkpoint |
| D16 | Proposed step changes of pseudocode Section 9.5 (review package item 16, added 2026-11-13) | any of the proposals from O4, O10, O11, O12, P2 and R2.4 | the F3 to F5 diffusion-step rule and the R2.4 step-3 correction; defer the rest (Stage A recommendation) | adopt only the F3 to F5 diffusion-step rule and the R2.4 step-3 correction; defer the rest (including R1.0 mode counting, R2.5 and H4b) | 2026-11-24 | owner, Stage A checkpoint |

### Owner policy decisions

These rank first in the authority order of `prompt/MASTER_PROMPT.md`, above every other file in `prompt/`. They were recorded on the owner's instruction.

| Id | Decision | Replaces | Source | Date |
|---|---|---|---|---|
| M1 | Merge policy. After all mandatory tests, the independent verification and every acceptance criterion pass, the coordinator merges `main` into the work branch with a normal commit if needed, re-runs CI, and then fast-forward pushes the work branch to `main` (`prompt/COMMIT_AND_DEPLOYMENT.md` Section 6). Never the GitHub merge button or `gh pr merge`, never force, never a history rewrite. Every scientific safeguard and every other rule of the package stays as it is. | every earlier statement that forbids merging into `main` or requires the final pull request to stay unmerged, including the verbatim records `prompt/ORIGINAL_REQUEST.md` ("Do not merge into `main` without explicit authorisation"; this row is that authorisation) and `prompt/RAYSTORAGE_REQUEST.md` ("Do not force-push or merge into main"; the force-push ban stands) | owner, Stage A checkpoint | 2026-11-24 |
| M2 | Commit attribution. Commits are attributed to the owner only; no assistant attribution of any kind in commit messages, pull requests or files | the attribution-trailer sentence of `prompt/COMMIT_AND_DEPLOYMENT.md` Section 2 | owner, Stage A checkpoint | 2026-11-24 |
| M3 | The `prompt/` folder is not published on the website (not in the Pages artefact, not linked from the site) | none (new) | owner, Stage A checkpoint | 2026-11-24 |
| M4 | Every quantity shown in the browser comes from the Rust solver through the WebAssembly wrapper or from verified precomputed output. No scientific equation or derived quantity is computed in JavaScript or TypeScript, including layer inventory shares and difference curves | the TypeScript computation of layer inventory shares (roadmap Phase 4) and of the difference curve (roadmap Phase 5) | owner, Stage A checkpoint | 2026-11-24 |
| M5 | The sentence in the repository `README.md` about the internal SNRSI comparison is frozen exactly as it stands: not deleted, extended, repeated or quoted anywhere else (not in the demo, the portfolio or any presentation). Until computed per-temperature comparisons exist, which the owner will ask for later, no experimental-agreement figure is written anywhere | none (new) | owner, Stage A checkpoint | 2026-11-24 |

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
