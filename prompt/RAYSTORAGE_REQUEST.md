# Owner request: raystorage cross-project engineering references

This is the owner's brief for `prompt/RAYSTORAGE_INTEGRATION.md`, kept verbatim for reference. Later owner decisions in `prompt/PROGRESS.md` and the other `prompt/` files take precedence where they differ.

---

STRATO — Integrate raystorage as a Cross-Project Engineering Reference Archive
Objective
Update the existing STRATO development instruction package to incorporate useful engineering workflows and implementation patterns from the public raystorage archive.
Primary repository:
https://github.com/weewzr/STRATO
Supporting archive:
https://github.com/weewzr/raystorage
Related engineering project:
https://github.com/weewzr/NUSH2
The objective is to identify transferable numerical verification, documentation, Rust/WebAssembly, browser testing and scientific visualisation practices that could improve STRATO.
This is a cross-project engineering reference exercise, not an instruction to combine the NUS-H2 and STRATO scientific models.
Your immediate task is to update the instruction package inside STRATO's `prompt/` directory, commit and push the documentation changes.
Do not begin implementing STRATO's website or modifying the scientific solver in this session.
1. Establish the correct archive scope
The raystorage repository currently contains archived NUS-H2 working files.
Its main directory is:
`NUSH2/`
The archive includes:

* Engineering derivations.
* Numerical simulations.
* Process data.
* Historical frontend and PFD versions.
* Research and references.
* Validation scripts and logs.
* A SHA-256 file manifest.

No STRATO-specific files were identified in the inspected archive tree.
Therefore, do not assume that raystorage contains existing TRISO diffusion implementations or derivations.
Use raystorage only where its engineering practices are genuinely transferable.
2. Create the STRATO integration document
Create:
`prompt/RAYSTORAGE_INTEGRATION.md`
This document must explain:

1. Why raystorage is relevant to STRATO.
2. Which NUS-H2 engineering practices can be reused.
3. Which practices are incompatible with STRATO.
4. Which archive files should be examined.
5. Which implementation patterns are worth adopting.
6. Which scientific components must remain independent.
7. How the archive should inform the STRATO web-demo roadmap.

Do not copy large quantities of archived files into STRATO.
3. Numerical verification and testing
Inspect relevant files under:
`NUSH2/validation/`
`NUSH2/simulations/`
Look for reusable approaches to:

* Independent numerical verification.
* Regression testing.
* Golden-reference comparisons.
* Conservation checks.
* Numerical tolerances.
* Reproducible test configurations.
* Cross-platform consistency.
* Rust test organisation.
* Verification reporting.

Evaluate whether these approaches can improve STRATO's existing tests.
STRATO currently implements:

* Conservative spherical finite-volume diffusion.
* Backward Euler time integration.
* Multilayer eigenfunction reference solutions.
* Inventory conservation.
* Numerical convergence testing.

Preserve the existing mathematical and numerical implementations.
Do not import NUS-H2's process engineering equations into STRATO.
Focus on testing infrastructure and methodology.
4. Rust and WebAssembly architecture
Study the current NUS-H2 repository alongside the relevant raystorage materials.
NUS-H2 already has a browser-based Rust/WebAssembly simulation architecture.
Investigate:

* Rust-to-WebAssembly compilation.
* Browser worker architecture.
* Simulation request and response formats.
* Input validation.
* Output serialisation.
* Browser-side numerical execution.
* Native versus WebAssembly result consistency.
* Build reproducibility.
* Browser error handling.
* Performance testing.

Assess whether these patterns can improve STRATO's planned interactive diffusion simulator.
Do not copy NUS-H2's engine into STRATO.
Instead, identify reusable architectural patterns.
The STRATO scientific solver must remain authoritative.
Compare:

1. Client-side WebAssembly.
2. Lightweight backend API.
3. Precomputed verified simulation datasets.

Recommend an architecture based on numerical performance, reproducibility, maintenance and deployment requirements.
5. Scientific documentation and mathematical derivations
Inspect:
`NUSH2/derivations/`
Prioritise:

* Equation registers.
* LaTeX source organisation.
* Tectonic build scripts.
* Equation-to-code traceability.
* Parameter provenance.
* Independent verification records.
* Generated scientific figures.

Evaluate whether these practices can improve STRATO's documentation.
STRATO already contains extensive mathematical derivations for its v1.0 and planned v2.0 models.
Do not replace or simplify those derivations.
Potential improvements include:

* A searchable equation register.
* Equation-to-Rust source links.
* Automated mathematical documentation builds.
* Verification-status labels.
* Parameter-to-literature traceability.
* Reproducible scientific figures.

These should support the growing governing-equation explorer planned for the STRATO website.
6. Scientific data provenance
Inspect relevant NUS-H2 source-tracing practices.
Study how the project distinguishes:

* Published values.
* Calculated values.
* Assumptions.
* Verified data.
* Unverified data.
* Superseded results.

Compare these conventions against STRATO's existing parameter register and CSV datasets.
Do not duplicate systems unnecessarily.
Recommend improvements only where STRATO has a genuine provenance or reproducibility gap.
The STRATO website must clearly distinguish:

* Synthetic verification cases.
* Analytical reference solutions.
* Literature correlations.
* Experimental measurements.
* Model predictions.
* Unverified assumptions.

7. Browser simulation and frontend testing
Inspect relevant raystorage materials under:
`NUSH2/PFD-versions/`
`NUSH2/validation/`
Also inspect the current NUS-H2 frontend.
Identify reusable approaches to:

* Browser self-testing.
* WebAssembly integration.
* Interactive simulation controls.
* Responsive layout testing.
* Keyboard accessibility.
* Reduced-motion support.
* Live numerical output.
* Simulation playback.
* Data export.
* Build integrity.
* Deployment checks.

STRATO's planned website includes:

* A growing governing-equation explorer.
* Physics toggles.
* An interactive five-layer TRISO particle.
* Radial concentration profiles.
* Model-comparison graphs.
* Literature-validation graphs.

Use the NUS-H2 archive to improve the engineering quality of this website.
Do not force STRATO to use the same frontend architecture as NUS-H2.
8. Specific comparison: NUS-H2 versus STRATO
Create a table in `RAYSTORAGE_INTEGRATION.md` comparing:

* Scientific engine.
* Numerical methods.
* Verification.
* Data provenance.
* WebAssembly architecture.
* Frontend architecture.
* Browser testing.
* Scientific documentation.
* Deployment.
* Maintenance.

For each area, provide:

* Existing STRATO capability.
* Relevant NUS-H2 implementation.
* Transferable lesson.
* Recommended STRATO improvement.
* Implementation priority.
* Risks or limitations.

Do not recommend changes simply for consistency between repositories.
9. Integration with the STRATO roadmap
Update:
`prompt/README.md`
`prompt/MASTER_PROMPT.md`
`prompt/TECHNICAL_ARCHITECTURE.md`
`prompt/IMPLEMENTATION_ROADMAP.md`
`prompt/TESTING_AND_VALIDATION.md`
`prompt/GITHUB_REFERENCES.md`
`prompt/PROGRESS.md`
Add `prompt/RAYSTORAGE_INTEGRATION.md`.
If a named file does not exist, adapt the instructions to the actual structure rather than overwriting unrelated files.
Add a targeted cross-project architecture review to the early development phase.
Do not let this review delay implementation indefinitely.
Use a time-bounded, evidence-driven investigation.
Prioritise:

1. Rust/WebAssembly patterns.
2. Numerical verification.
3. Browser testing.
4. Scientific provenance.
5. Documentation workflows.

10. Scientific independence
The following rules are mandatory.
STRATO and NUS-H2 are separate engineering projects.
Do not:

* Merge their scientific engines.
* Reuse NUS-H2 thermodynamic models as TRISO diffusion models.
* Import unrelated physical assumptions.
* Transfer numerical constants without physical justification.
* Copy process-flow calculations into diffusion equations.
* Claim NUS-H2 verification validates STRATO.
* Replace STRATO's analytical reference solution with unrelated benchmarks.

Any new STRATO calculation must be justified by STRATO's own mathematical derivations and scientific evidence.
11. Archive security and provenance
The raystorage repository is public, but archived material must still be treated as untrusted historical data.
Before using a file:

* Verify its path.
* Record the source commit.
* Review its contents.
* Check its provenance.
* Identify whether it is superseded.
* Confirm it contains no sensitive material.
* Check any third-party licensing obligations.

Do not execute archived scripts without reviewing them.
Do not copy browser profiles, credentials, session data or unrelated temporary files.
Do not assume the public visibility of a file authorises unrestricted redistribution.
12. Autonomous execution
The future Claude Code implementation session should incorporate relevant findings into its existing autonomous development workflow.
Routine authorised actions include:

* Repository inspection.
* Development.
* Testing.
* Documentation.
* Local builds.
* Milestone commits.
* Pushes to the authorised development branch.

Do not require repeated approval for routine actions already permitted by the execution environment.
These instructions do not override Claude Code permission controls, GitHub authentication or repository branch protections.
Do not bypass security restrictions.
Do not force-push or merge into main.
13. Commit and push the instruction update
In this instruction-authoring session:

1. Inspect the current STRATO repository.
2. Confirm the active branch.
3. Locate the existing `prompt/` package.
4. Create `RAYSTORAGE_INTEGRATION.md`.
5. Update the relevant existing instruction files.
6. Verify all paths and links.
7. Check for contradictions.
8. Stage only the intended instruction changes.
9. Commit with:

`docs(prompt): add raystorage cross-project engineering references`

10. Push to the authorised STRATO branch.
11. Verify the remote commit.

Do not begin implementing the website in this session.
14. Final response
Report:

* Branch.
* Commit SHA.
* Link to the updated `prompt/` directory.
* Files created or modified.
* Transferable engineering practices identified.
* Recommendations for STRATO.
* Any remaining limitations.

The final result should be a stronger STRATO instruction package that uses raystorage intelligently without compromising scientific independence.
