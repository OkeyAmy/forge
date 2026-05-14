# Forge GitHub App Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Forge's webhook receiver into a durable GitHub App runtime that can plan issue work, gate execution on `/forge approve`, review PRs natively, and run approved work inside E2B.

**Architecture:** Keep the public API small: GitHub webhooks enter Forge, become durable jobs, then a background worker processes jobs and posts results back through GitHub App installation tokens. E2B is the only sandbox boundary for public execution; legacy direct run endpoints stay disabled by default.

**Tech Stack:** Rust, Axum, Tokio, file-backed JSON persistence, GitHub App REST API, E2B SDK bridge scripts, existing Forge model abstractions.

---

### Task 1: Durable Job Store

**Files:**
- Modify: `crates/forge-types/src/public_workflow.rs`
- Create: `crates/forge-api/src/job_store.rs`
- Modify: `crates/forge-api/src/main.rs`

- [ ] Add job records to `forge-types` with enough GitHub context to resume after restart.
- [ ] Add a file-backed JSON store that creates its parent directory, appends jobs atomically, and updates job state by id.
- [ ] Add tests proving jobs survive reload and latest issue plan lookup works.

### Task 2: Background Worker Queue

**Files:**
- Create: `crates/forge-api/src/workflow.rs`
- Modify: `crates/forge-api/src/routes/github_app.rs`
- Modify: `crates/forge-api/src/main.rs`

- [ ] Start a Tokio worker from `main` with a bounded channel and shared job store.
- [ ] On accepted GitHub commands, persist a job and enqueue it instead of only posting placeholder comments.
- [ ] Mark jobs `running`, `waiting_for_approval`, `branch_pushed`, or `failed` from worker outcomes.

### Task 3: Real `/forge plan` and `/forge approve`

**Files:**
- Modify: `crates/forge-api/src/workflow.rs`
- Modify: `crates/forge-api/src/github_app_auth.rs`
- Modify: `crates/forge-types/src/public_workflow.rs`

- [ ] `/forge plan` posts a concrete plan comment from the issue body, repo branch, and configured checks.
- [ ] `/forge approve` finds the latest waiting issue plan and starts an E2B execution job.
- [ ] Failed missing configuration is posted back to GitHub and recorded in the job store.

### Task 4: E2B Runner and Branch Push

**Files:**
- Create: `scripts/e2b-runner/package.json`
- Create: `scripts/e2b-runner/run-job.mjs`
- Modify: `crates/forge-api/src/workflow.rs`
- Modify: `.env.production.example`
- Modify: `docs/deployment/vultr-github-app.md`

- [ ] Runner creates an E2B sandbox, clones the installed repository with an installation token, checks out `forge/issue-N`, runs Forge's configured commands, captures `git diff --name-only`, and pushes the branch.
- [ ] Worker invokes the runner with JSON input, parses JSON output, and posts the branch-ready comment.
- [ ] Tests cover command construction and output parsing without fake GitHub/E2B claims.

### Task 5: Native PR Review

**Files:**
- Modify: `crates/forge-api/src/github_app_auth.rs`
- Modify: `crates/forge-api/src/workflow.rs`
- Modify: `crates/forge-types/src/public_workflow.rs`

- [ ] `/forge review` fetches PR changed files through GitHub App auth.
- [ ] Forge produces a review comment with risks, suggested fixes, and test gaps using native code, not PR-Agent.
- [ ] PR commands remain rejected on issues and issue commands remain rejected on PRs.

### Task 6: Install/Setup Landing Page

**Files:**
- Create: `crates/forge-api/src/routes/setup.rs`
- Modify: `crates/forge-api/src/main.rs`
- Modify: `.env.production.example`

- [ ] Public root page explains the GitHub App install flow and links to `GITHUB_APP_PUBLIC_URL` when configured.
- [ ] Health and webhook routes remain machine-friendly.

### Task 7: Verification and Push

**Files:**
- Modify: all touched files

- [ ] Run focused tests for job store, webhook, and workflow behavior.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo test --workspace`; E2B live test runs only when the real key is present.
- [ ] Commit using the Lore commit protocol and push the branch.
