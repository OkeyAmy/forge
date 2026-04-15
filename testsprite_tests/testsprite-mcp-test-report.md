# TestSprite MCP — Forge API Test Report

## 1️⃣ Document Metadata

- **Project name:** SWEagent (`forge-api` crate)
- **Date:** 2026-04-15
- **Execution:** TestSprite MCP `testsprite_generate_code_and_execute` plus CLI  `node …/testsprite-mcp/dist/index.js generateCodeAndExecute` (tunnel to `http://localhost:5000`)
- **Config:** [`testsprite_tests/tmp/config.json`](tmp/config.json) — `type: backend`, `localEndpoint: http://localhost:5000`
- **Prepared by:** TestSprite cloud run + local report completion

---

## 2️⃣ Requirement Validation Summary

### R1 — Liveness (`GET /health`)

| ID | Case | Result |
|----|------|--------|
| TC008 | JSON `status: ok`, non-empty `version` | Passed |

### R2 — Stats (`GET /api/stats`)

| ID | Case | Result |
|----|------|--------|
| TC006 | Aggregates for existing / missing dirs | Passed |

### R3 — GitHub issues (`GET /api/issues`)

| ID | Case | Result |
|----|------|--------|
| TC007 | List issues; PRs excluded server-side | Passed |

### R4 — Trajectory list (`GET /api/trajectories`)

| ID | Case | Result |
|----|------|--------|
| TC004 | Metadata shape; `exit_status` string; 404 for bad dir | Passed |

### R5 — Batch runs (`POST /api/run/batch`)

| ID | Case | Result |
|----|------|--------|
| TC002 | Valid batch200; malformed body **400** (manual JSON parse) | Passed |

### R6 — Watch (`POST` / `GET` / `DELETE /api/watch`)

| ID | Case | Result |
|----|------|--------|
| TC003 | Start, 409 on double start, GET status, DELETE with cleared fields | Passed |

### R7 — Single run (`POST /api/run`)

| ID | Case | Result |
|----|------|--------|
| TC001 | Expects **200** + `exit_status: submitted` for placeholder GitHub URL | **Failed** — returns **500** when Docker / model / real repo are unavailable (~70 ms). Behaviour is expected without a full Forge runtime. |

### R8 — Trajectory by name (`GET /api/trajectories/{name}`)

| ID | Case | Result |
|----|------|--------|
| TC005 | Generated test calls **`POST /api/run` first** to create a `.traj`, then GET detail | **Failed** — same root cause as TC001 (`/api/run` 500). Listing an **existing** `.traj` would pass; the generated script depends on a successful run. |

---

## 3️⃣ Coverage & Matching Metrics

- **Tests run:** 8  
- **Passed:** 6 (**75%**)  
- **Failed:** 2 (**25%**)

| Area | Total | Passed | Failed |
|------|-------|--------|--------|
| Health | 1 | 1 | 0 |
| Stats | 1 | 1 | 0 |
| Issues | 1 | 1 | 0 |
| Trajectories list | 1 | 1 | 0 |
| Batch | 1 | 1 | 0 |
| Watch | 1 | 1 | 0 |
| Single run | 1 | 0 | 1 |
| Trajectory detail | 1 | 0 | 1 |

---

## 4️⃣ Key Gaps / Risks

1. **TestSprite MCP workflow (what “proper” use means):**  
   - Call **`testsprite_generate_code_and_execute`** in the IDE (or equivalent MCP).  
   - When the tool returns `next_action`, run **`node …/testsprite-mcp/dist/index.js generateCodeAndExecute`** from the project root so the cloud runner can reach your app via the tunnel.  
   - Keep **`forge-api` running** on the port in `testsprite_tests/tmp/config.json` (here **5000**) for the whole run.

2. **TC001 / TC005 and `/api/run`:** Failures are **environment / integration**, not missing routes. Passing without Docker + valid model + real issue is unrealistic unless TestSprite generates tests that **list existing `.traj` files** instead of creating them via `/api/run`.

3. **Regenerated tests overwrite local edits:** Each `generateCodeAndExecute` run may replace `TC00*.py`. Stabilise behaviour in **Rust** (already done for batch400, issues PR filter, watch DELETE shape, trajectory `exit_status` string) and adjust **PRD / code_summary** so the next generation avoids “always200 on `/api/run`” when you only want smoke tests.

4. **Secrets:** Do not commit API keys from `tmp/config.json`; rotate anything that was ever pasted into TestSprite env fields.

---

## Raw artifacts

- [`tmp/raw_report.md`](tmp/raw_report.md)  
- [`tmp/test_results.json`](tmp/test_results.json)  
- Dashboard links per case in `raw_report.md`
