# Forge — Product Requirements Document (TestSprite)

## Product Overview

Forge is an autonomous AI software-engineering agent that fixes GitHub issues using any
OpenAI-compatible language model. It clones a repository into an isolated Docker sandbox,
analyses the problem statement, and produces a git diff (patch) that resolves the issue.
The patch can be submitted automatically as a GitHub pull request.

`forge-api` is the HTTP server wrapper around all Forge capabilities. It exposes REST
endpoints so external tools (like TestSprite) can drive the full agent lifecycle — single
runs, batch runs, continuous watching, trajectory inspection, and stats — without invoking
the CLI directly.

---

## Core Goals

- Trigger single and batch Forge agent runs via HTTP.
- Continuously watch a GitHub repository and auto-fix labelled issues.
- Inspect individual trajectory files (full agent history + patch).
- Aggregate outcome statistics across trajectory directories.
- List open GitHub issues for a repository.
- Provide health / readiness probes for CI/CD integration.

---

## Key Features

1. **Single Run (POST /api/run)** — Full agent run with every RunConfig option exposed.
2. **Batch Run (POST /api/run/batch)** — Parallel multi-issue runs with configurable concurrency.
3. **Watch Mode (POST/GET/DELETE /api/watch)** — Start, inspect, and stop the continuous watcher.
4. **Trajectory List (GET /api/trajectories)** — List all `.traj` files with metadata.
5. **Trajectory Detail (GET /api/trajectories/:name)** — Full content of one trajectory file.
6. **Stats (GET /api/stats)** — Aggregate outcome counts across a trajectory directory.
7. **Issue Listing (GET /api/issues)** — Query open GitHub issues.
8. **Health Check (GET /health)** — Liveness / readiness probe.

---

## API Endpoints

### GET /health

**Purpose:** Liveness / readiness probe.

**Response (200 OK):**
```json
{ "status": "ok", "version": "0.1.0" }
```

---

### POST /api/run

**Purpose:** Run the Forge agent on a GitHub issue or plain-text problem. Blocks until the
agent finishes (configure HTTP client timeout ≥ 10 minutes).

**Request body (JSON):**
```json
{
  "github_url": "https://github.com/owner/repo/issues/42",
  "output_dir": "trajectories",
  "agent": {
    "model": "gpt-4o",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",
    "parser_type": "xml",
    "max_steps": 100,
    "max_requeries": 3,
    "system_template": null,
    "instance_template": null
  },
  "env": {
    "image": "akachiokey/forge-sandbox:latest",
    "container_name": null,
    "repo_path": "/repo",
    "timeout_secs": 120,
    "startup_commands": [],
    "env_vars": [["MY_VAR", "value"]],
    "base_commit": null
  }
}
```

Alternative problem sources (use one):
- `"github_url"` — full GitHub issue URL
- `"repo"` + `"issue"` — owner/repo + issue number
- `"problem_text"` — raw text problem statement

All `agent` and `env` fields are optional; unset values fall back to environment variables.

**Response (200 OK):**
```json
{
  "exit_status": "submitted",
  "has_submission": true,
  "submission_preview": "diff --git a/src/...",
  "steps": 23,
  "model_stats": { "total_cost": 0.042 },
  "trajectory_file": "trajectories/owner__repo-i42.traj"
}
```

**Possible `exit_status` values:**
| Value | Meaning |
|---|---|
| `submitted` | Agent produced a patch |
| `forfeited` | Agent gave up |
| `step_limit_reached` | Max steps exhausted |
| `error` | Agent hit a fatal error |

**Errors:** `400` bad input · `500` agent runtime failure

---

### POST /api/run/batch

**Purpose:** Run the agent on multiple issues in parallel.

**Request body (JSON):**
```json
{
  "output_dir": "trajectories",
  "workers": 4,
  "items": [
    { "github_url": "https://github.com/owner/repo/issues/1" },
    { "repo": "owner/repo", "issue": 2, "agent": { "max_steps": 50 } },
    { "problem_text": "Fix the off-by-one error in utils.py" }
  ]
}
```

Each item supports the same `agent` / `env` overrides as `/api/run`.

**Response (200 OK):**
```json
{
  "output_dir": "trajectories",
  "total": 3,
  "succeeded": 2,
  "failed": 1,
  "results": [
    { "instance_id": "https://github.com/owner/repo/issues/1", "success": true, "error": null },
    { "instance_id": "https://github.com/owner/repo/issues/2", "success": true, "error": null },
    { "instance_id": "text-Fix the off-by-one", "success": false, "error": "docker not found" }
  ]
}
```

---

### POST /api/watch

**Purpose:** Start the background watcher. The watcher polls GitHub every `interval` seconds
and runs the agent on any new issues carrying the given label.

Only one watch task runs at a time. Returns `409 Conflict` if already running.

**Request body (JSON):**
```json
{
  "repo": "owner/repo",
  "label": "forge",
  "interval": 60,
  "model": "gpt-4o",
  "base_url": "https://api.openai.com/v1",
  "api_key": "sk-...",
  "image": "akachiokey/forge-sandbox:latest",
  "max_steps": 100,
  "output_dir": "trajectories"
}
```

**Response (200 OK):**
```json
{
  "running": true,
  "repo": "owner/repo",
  "label": "forge",
  "started_at": "2026-04-15T10:00:00Z"
}
```

---

### GET /api/watch

**Purpose:** Query current watch task status.

**Response (200 OK):**
```json
{
  "running": true,
  "repo": "owner/repo",
  "label": "forge",
  "started_at": "2026-04-15T10:00:00Z"
}
```

Returns `running: false` with null fields when no watcher is active.

---

### DELETE /api/watch

**Purpose:** Stop the running watch task.

**Response (200 OK):** Same shape as GET /api/watch with `running: false`.

**Error:** `404` if no watch task is running.

---

### GET /api/trajectories

**Purpose:** List all `.traj` files in a directory with per-file metadata.

**Query parameters:**
| Param | Default | Description |
|---|---|---|
| `dir` | `"trajectories"` | Directory to scan |

**Response (200 OK):**
```json
{
  "directory": "trajectories",
  "count": 3,
  "trajectories": [
    {
      "name": "owner__repo-i42.traj",
      "exit_status": "submitted",
      "has_submission": true,
      "steps": 23,
      "model_stats": { "total_cost": 0.042 }
    }
  ]
}
```

**Error:** `404` if directory does not exist.

---

### GET /api/trajectories/:name

**Purpose:** Return the complete content of a single `.traj` file (trajectory steps, agent
info, model stats, and the full submission patch).

**Path parameter:** filename including `.traj` extension.

**Query parameters:**
| Param | Default | Description |
|---|---|---|
| `dir` | `"trajectories"` | Directory containing the file |

**Example:** `GET /api/trajectories/owner__repo-i42.traj?dir=trajectories`

**Response (200 OK):** Full `TrajFile` JSON — `trajectory`, `info`, `environment`.

**Errors:** `404` file not found · `422` parse error.

---

### GET /api/stats

**Purpose:** Aggregate outcome counts across a directory of `.traj` files.

**Query parameters:**
| Param | Default | Description |
|---|---|---|
| `dir` | `"trajectories"` | Directory to scan |

**Response (200 OK):**
```json
{
  "directory": "trajectories",
  "total": 10,
  "submitted": 7,
  "forfeited": 1,
  "errors": 1,
  "step_limit_reached": 1,
  "other": 0
}
```

---

### GET /api/issues

**Purpose:** List open GitHub issues for a repository.

**Query parameters:**
| Param | Required | Default | Description |
|---|---|---|---|
| `repo` | Yes | — | `owner/repo` format |
| `label` | No | — | Filter by label |
| `limit` | No | `30` | Max results (cap: 100) |

**Response (200 OK):**
```json
{
  "repo": "owner/repo",
  "count": 2,
  "issues": [
    {
      "number": 42,
      "title": "Fix validation bug",
      "url": "https://github.com/owner/repo/issues/42",
      "labels": ["bug"]
    }
  ]
}
```

---

## User Flow Summary

1. `GET /health` — confirm server is up.
2. `GET /api/issues?repo=owner/repo` — find a real issue to fix.
3. `POST /api/run` with `github_url` — run the agent on it.
4. Assert `exit_status == "submitted"` and `has_submission == true`.
5. `GET /api/trajectories` — confirm the traj file was created.
6. `GET /api/trajectories/:name` — inspect full agent history.
7. `GET /api/stats` — assert `submitted >= 1`.
8. `POST /api/watch` — start the watcher; `GET /api/watch` confirms it's running; `DELETE /api/watch` stops it.
9. `POST /api/run/batch` — run multiple issues in parallel and check `succeeded == total`.

---

## Validation Criteria

- `GET /health` → 200 with `{ "status": "ok" }`.
- `GET /api/issues` valid public repo → non-empty list.
- `GET /api/issues` invalid repo → 502 with descriptive error.
- `POST /api/run` missing problem source → 400.
- `POST /api/run` valid GitHub issue → 200 with `exit_status` set.
- `POST /api/run/batch` mixed items → correct `succeeded` / `failed` counts.
- `POST /api/watch` → 200 running; second POST → 409.
- `GET /api/watch` while running → `running: true`.
- `DELETE /api/watch` → 200 stopped; second DELETE → 404.
- `GET /api/trajectories` missing dir → 404.
- `GET /api/trajectories` after a run → at least one entry.
- `GET /api/trajectories/:name` valid → 200 with full traj.
- `GET /api/trajectories/:name` missing → 404.
- `GET /api/stats` valid dir → correct numeric totals.
- `GET /api/stats` missing dir → 404.

---

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `FORGE_MODEL` | Yes | Model name |
| `FORGE_BASE_URL` | Yes | OpenAI-compatible base URL |
| `FORGE_API_KEY` | Yes | API key |
| `GITHUB_TOKEN` | Recommended | PAT for issue reads and PR creation |
| `FORGE_API_PORT` | No (default: 5000) | Port the HTTP server binds to |
| `RUST_LOG` | No | Log filter e.g. `"forge_api=info"` |

---

## Tech Stack

- **Language:** Rust (2021 edition)
- **HTTP framework:** axum 0.8
- **Runtime:** tokio (async)
- **Serialization:** serde + serde_json
- **Docker orchestration:** bollard
- **Model client:** reqwest (OpenAI-compatible REST)
