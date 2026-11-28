# Antigravity Development Tools & Free UI Resource Audit

This record documents the free-only UI design evaluation (Stage 3) and Antigravity development tool audit and setup (Stages 4–5) for **STRATO**.

## Zero-Cost Policy

All tools, extensions, MCP servers, and references evaluated or configured for STRATO are strictly zero-cost (`$0`) under open-source licenses or unauthenticated/free community tiers:

- No paid subscriptions, commercial plans, paid trials, usage-based billing, or payment credentials were added or activated.
- No credentials, OAuth tokens, API keys, or local username paths are stored in this repository.

---

## Stage 3 — Free Open-Source UI Resources Evaluated

| Candidate Repository | License / Stars | Evaluation Scope | Disposition & Outcome |
| :--- | :--- | :--- | :--- |
| `https://github.com/monet-design/monet-registry` | `None` (no SPDX license in GitHub repo metadata) / `427` stars | Inspected layout proportions and section registry patterns | **Visual reference only** — zero code copied due to missing explicit open-source license |
| `https://github.com/0050piyush/flowscope` | `MIT` / `1` star | Reviewed static HTML/CSS/JS UI pattern library (`144` screens, `27` patterns, `28` UI elements) | **Pattern reference only** — informed subtle spacing, pill toggle affordances, and equation card framing (`web/src/design/explore.css`); zero runtime dependency added |
| `https://github.com/INSANE0777/Awwwards-mcp` | `MIT` (`v1.22.1`) / `978` stars | Node `>=22.13.0` Playwright website-scraping MCP server | **Not installed** — unnecessary external web-scraping surface for STRATO's domain-specific scientific UI |

All Stage 3 UI improvements (`web/src/design/explore.css` and `web/src/graph/ExploreGraphs.tsx`) were authored in native repository CSS using existing light/dark colour tokens (`--bg`, `--surface`, `--accent-bg`, `--series-b`, `--series-c`, `--text`) while preserving `Method A: BTCS` (`#DC2626` red), `Method B: Sturm–Liouville` (`#2563EB` blue), and experimental black/grey marker styling.

---

## Stage 4 — Antigravity Development Tool Audit & Configuration

### Status Summary

| Tool | Source Repository | License / Version / Commit | Install Method | Config Location | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Conductor** | `https://github.com/gemini-cli-extensions/conductor` | `Apache-2.0` / `0.3.0` / `6e8f9a860bcdd6a2c423473c12e745200688c633` | Shallow Git clone into Antigravity global plugins directory | `~/.gemini/config/plugins/conductor` | **Installed and verified** |
| **Context7** | `https://github.com/upstash/context7` | `MIT` / MCP server `4.9.1` / `e8d84b8ebcc82866703f7ea86a61c7261dbc07d4` | Unauthenticated Streamable HTTP MCP endpoint | `~/.gemini/config/mcp_config.json` | **Installed and verified** |
| **CodeRabbit** | `https://github.com/coderabbitai/skills` | `MIT` / `1.3.5` / `f0c27e8b1c8f0bf9e51dd30d55ae4d1effef5071` | Antigravity Marketplace plugin (`antigravity-plugins-partners/plugin/coderabbit`) + optional local CLI | Antigravity Marketplace / Plugins Manager | **Pending user approval** |
| **Ralph Loop** | `https://github.com/burakkaygusuz/ralph-wiggum` | `Apache-2.0` LICENSE text (GitHub SPDX `NOASSERTION`) / `3.0.0` / `49461659082e493c8b077d2c28e517449259f361` | Not installed (POSIX bash Stop-hook plugin; `0` stars) | N/A | **Investigated but unsupported** |

---

### 1. Conductor (`gemini-cli-extensions/conductor`)

- **Purpose:** Structured specification, track planning, implementation, status tracking, review, and git-aware revert skills (`conductor-setup`, `conductor-new-track`, `conductor-implement`, `conductor-review`, `conductor-status`, `conductor-revert`) plus Antigravity modal prompt rules (`rules/conductor_antigravity.md`).
- **Installation & Verification:** Cloned at `6e8f9a860bcdd6a2c423473c12e745200688c633` (`v0.3.0`, `Apache-2.0`) into `~/.gemini/config/plugins/conductor`. Verified presence of `plugin.json`, `VERSION`, `LICENSE`, `rules/conductor_antigravity.md`, and all six `skills/*/SKILL.md` directories. Zero shell hooks or paid dependencies; no STRATO tracks were initialized.
- **Authentication & Free-Tier Limits:** None required (`$0`, local extension files only).
- **Activation & Reload Note:** Antigravity discovers new global plugin directories on session reload/restart. Run Conductor workflows only when starting an intentional spec-driven track (existing `docs/ANTIGRAVITY_PROGRESS.md` remains authoritative for STRATO).

### 2. Context7 (`upstash/context7`)

- **Purpose:** Live version-specific library documentation retrieval via MCP (`resolve-library-id`, `query-docs`).
- **Installation & Configuration:** Configured unauthenticated free Streamable HTTP transport in `~/.gemini/config/mcp_config.json`:
  ```json
  {
    "mcpServers": {
      "context7": {
        "url": "https://mcp.context7.com/mcp"
      }
    }
  }
  ```
- **Verification Results:** Live HTTP JSON-RPC (`protocolVersion: 2025-03-26`) against `https://mcp.context7.com/mcp` returned `HTTP 200 OK` (`Context7 v4.9.1`), resolving React (`/reactjs/react.dev`), TypeScript (`/microsoft/typescript`), and Vite (`/vitejs/vite`) IDs and returning live React `useState` documentation without an API key.
- **Authentication & Free-Tier Limits:** Works immediately at zero cost without an API key under Upstash's public rate limits; an optional free Context7/Upstash account key can raise personal rate limits later if desired, or the Marketplace plugin (`antigravity-plugins-partners/plugin/context7`) can be added from the UI for versioned Marketplace updates.

### 3. CodeRabbit (`coderabbitai/skills`)

- **Purpose:** AI pull-request and local diff review (`code-review`, `autofix`, `coderabbit-config`, `coderabbit-secure`, `coderabbit-setup`) backed by remote MCP (`https://mcp.coderabbit.ai/`) and optional `coderabbit` CLI.
- **Audit & Pending Status:** Audited `coderabbitai/skills` (`MIT`, `v1.3.5`, `f0c27e8b1c8f0bf9e51dd30d55ae4d1effef5071`), its `.antigravity-plugin/README.md`, `plugin.json`, and `mcp_config.json`. Left **Pending user approval** rather than forcing unauthenticated config or conflicting local plugin names so the official Marketplace plugin (`antigravity-plugins-partners/plugin/coderabbit`) can be installed and connected cleanly via OAuth consent in Antigravity (`CodeRabbit > Accounts`) without bypassing user consent, starting paid Pro trials, or enabling PR auto-merge.
- **Authentication & Free-Tier Limits:** CodeRabbit's remote MCP requires GitHub/CodeRabbit OAuth sign-in via Antigravity UI; free tier covers public/open-source GitHub repositories (local CLI reviews also require the optional `coderabbit` CLI and `coderabbit auth login`).

### 4. Ralph Loop (`burakkaygusuz/ralph-wiggum`)

- **Purpose:** Iterative loop plugin using a Claude/Cursor-style `Stop` hook (`hooks/hooks.json`) running `bash "${CLAUDE_PLUGIN_ROOT}/scripts/stop-hook.sh"` until `.ralph/fix_plan.md` items and `--test-cmd` pass.
- **Audit & Compatibility Finding (`Investigated but unsupported`):**
  1. `scripts/stop-hook.sh` explicitly documents that paths are not JSON-decoded, so a working directory containing Windows backslashes (`\`) fails to resolve.
  2. `scripts/lib/step.sh` executes agent-writable `TEST_COMMAND` strings from `.ralph/state.env` inside a turn-blocking bash `Stop` hook without interactive confirmation.
  3. Targets `${CLAUDE_PLUGIN_ROOT}` / `agy` CLI (`agy plugin install`), whereas this workstation uses native Windows PowerShell without `agy`.
  4. Low community provenance (`0` stars at audit time).
- **Recommended Conservative Free Alternative:** Use Antigravity's built-in `/goal` and `/schedule` commands natively supported on Windows with zero shell `Stop` hooks and zero cost.
