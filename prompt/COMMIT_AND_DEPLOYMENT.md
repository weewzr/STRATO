# Commit and deployment

## 1 Branches

- `main`: protected by convention; only the owner merges into it.
- `feat/strato-interactive-web-demo`: the work branch, created from the latest `main`.
- Optional short-lived branches `feat/web-<topic>` cut from the work branch for subagents, merged locally back into the work branch by the coordinator.

## 2 Identity

Set once per clone, repo-local:

```
git config user.name "weewzr"
git config user.email "288510855+weewzr@users.noreply.github.com"
```

No personal email address in any commit. End each commit message with the Claude attribution trailer that the running session is configured to use.

## 3 Phase milestones

| Phase | Commit message |
|---|---|
| 0 | `docs(prompt): phase 0 - audit and architecture confirmation` |
| 1 | `feat(web): phase 1 - dashboard foundation` |
| 2 | `feat(web): phase 2 - interactive equation explorer` |
| 3 | `feat(web): phase 3 - rust solver integration` |
| 4 | `feat(web): phase 4 - triso particle visualisation` |
| 5 | `feat(web): phase 5 - scientific comparison graphs` |
| 6 | `feat(physics): phase 6 - verified model extensions` |
| 7 | `feat(validation): phase 7 - literature comparison` |
| 8 | `feat(web): phase 8 - deployment and polish` |

Name commits by the actual scope. Partial work uses an honest message (for example `feat(validation): phase 7 - measurement display, predictions pending phase 6`). Never commit unfinished functionality as complete. Intermediate commits inside a phase are fine (`wip(web): ...` is not allowed on the pushed branch; use a descriptive conventional message).

## 4 Pre-commit checklist

- [ ] Phase tests and the v1.0 suite pass (`prompt/TESTING_AND_VALIDATION.md`).
- [ ] Build succeeds (`cargo build --release`, wasm build, `npm run build`).
- [ ] `prompt/PROGRESS.md` updated.
- [ ] Privacy scan of the diff: no names other than weewzr / Wee Zhern Ray and Theodore; no course codes, group numbers, matric numbers, deadlines; no local absolute paths; no secrets or tokens; no personal email.
- [ ] Style: no em dash characters in prose files.
- [ ] Only intended files staged (`git status` reviewed); no build output (`target/`, `web/dist/`, `node_modules/`) committed.

## 5 Push

`git push origin feat/strato-interactive-web-demo`. No force-push, no history rewrite, no branch deletion. If push fails for lack of access, record a blocker and keep committing locally.

## 6 Pull request

When all achievable phases are done, open one PR from the work branch into `main` (`gh pr create`) with: phases completed, test and verification evidence, performance numbers, open blockers, owner decisions still pending. Never merge it (`gh pr merge` and the web merge button would stamp the account email on the merge commit). The owner merges locally.

## 7 GitHub Pages

- Workflow `.github/workflows/web.yml` builds `web/dist` with `base: '/STRATO/'` and uploads it with `actions/upload-pages-artifact`; a `deploy` job uses `actions/deploy-pages` with `permissions: pages: write, id-token: write`, `concurrency: pages`.
- Deploy only from `main`. Until the owner merges and enables Pages (Settings, Pages, Source: GitHub Actions), the work branch only uploads the build as a CI artefact for review.
- Stamp the commit hash into asset URLs (cache busting), as Outram Park and NUS-H2 do.
- Reproducible build: pinned toolchain and lock files; CI builds twice and compares the outputs apart from the stamp.

## 8 Releases

No tags or releases without the owner's explicit authorisation. Update `CHANGELOG.md` under `[Unreleased]` in each phase commit that changes user-visible behaviour.
