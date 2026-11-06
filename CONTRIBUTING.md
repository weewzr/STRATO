# Contributing to STRATO

STRATO is research software in which every equation, parameter and result has to be traceable to a derivation or a published source. Please keep to the rules below.

## Before opening a pull request

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --release
cargo run --release --example verify
```

All four must pass. Continuous integration runs the same commands.

## Rules for scientific changes

1. **Derive, then write pseudocode, then code.** A new equation goes into `docs/mathematics/` before it goes into `src/`.
2. **Every physical parameter needs a source.** Record the value, unit, material, temperature range, citation (APA 7) and page, table or equation number in the parameter register. Mark a value `NOT VERIFIED` if you cannot confirm it. Never fill a gap with a guess.
3. **Keep synthetic and literature inputs apart.** Synthetic cases go in `inputs/synthetic/`, literature cases in `inputs/literature/`.
4. **Do not move the Phase 1 results.** `tests/phase1_regression.rs` must keep passing when the new physics is switched off. If a change is meant to alter v1.0 numbers, explain why in the pull request and in `CHANGELOG.md`.
5. **Verification is not validation.** Agreement between the two solvers shows the equations are solved correctly. It does not show the physics is right.
6. **Each new capability gets a test** against an exact or independently computed answer, with a stated tolerance.

## Commit messages

Use conventional commits, for example `feat: add Arrhenius diffusivity model`, `test: add inventory conservation check`, `docs: derive modal source projection`.
