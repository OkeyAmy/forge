# Forge — Production Examples

Three ways to run Forge against real GitHub issues. **No Rust installation required** — just Docker and a `.env` file.

## Issues created for testing

| # | Title | Method |
|---|---|---|
| [#27](https://github.com/OkeyAmy/Axioschat-Onboard/issues/27) | Add `isValidWalletAddress()` | Method 1 — one-shot |
| [#28](https://github.com/OkeyAmy/Axioschat-Onboard/issues/28) | Add `formatTokenAmount()` | Method 2 — config file |
| [#29](https://github.com/OkeyAmy/Axioschat-Onboard/issues/29) | Add `truncateAddress()` *(labelled `forge`)* | Method 3 — watch |

---

## Quick start

```bash
# 1. Copy the shared env file into whichever method you want to test
cp .env.example method-1-oneshot/.env
# Fill in: FORGE_API_KEY, GITHUB_TOKEN, DOCKER_GID

# 2. Find your Docker group GID
getent group docker | cut -d: -f3

# 3. Run
cd method-1-oneshot && docker compose run --rm forge
```

---

## Methods

### Method 1 — One-shot (`method-1-oneshot/`)

Fixes issue #27. Single command, exits when done.

```bash
cd method-1-oneshot
cp ../.env.example .env   # fill in your keys
docker compose run --rm forge
```

Override issue inline:
```bash
docker compose run --rm forge run --repo OkeyAmy/Axioschat-Onboard --issue 28
```

---

### Method 2 — YAML config file (`method-2-config-file/`)

Fixes issue #28. All parameters (model, prompts, max steps) live in a versioned YAML file.

```bash
cd method-2-config-file
cp ../.env.example .env   # fill in your keys
docker compose run --rm forge
```

Edit `forge-issue28.yaml` to change model, system prompt, max steps, etc.

---

### Method 3 — Watch mode (`method-3-watch/`)

Runs continuously. Labels any issue `forge` on GitHub and Forge picks it up automatically.
Issue #29 is already labelled — it will be fixed within `FORGE_WATCH_INTERVAL` seconds.

```bash
cd method-3-watch
cp ../.env.example .env   # fill in your keys
docker compose up watch   # runs until you Ctrl+C
```

---

## Shared trajectories

All methods write trajectory files to `examples/trajectories/`.
View stats across all runs:

```bash
# From any method directory:
docker compose run --rm quick-stats
```

---

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `FORGE_MODEL` | Yes | Model name (e.g. `gpt-4o`, `models/gemini-2.0-flash-001`) |
| `FORGE_BASE_URL` | Yes | OpenAI-compatible API base URL |
| `FORGE_API_KEY` | Yes | API key |
| `GITHUB_TOKEN` | Recommended | Raises rate limit; required for private repos |
| `DOCKER_GID` | Yes | Host Docker group GID — `getent group docker \| cut -d: -f3` |
| `FORGE_REPO` | Method 1 | `owner/repo` to fix |
| `FORGE_ISSUE` | Method 1 | Issue number to fix |
| `FORGE_WATCH_REPO` | Method 3 | Repo to watch |
| `FORGE_WATCH_LABEL` | Method 3 | Label to watch for (default: `forge`) |
| `FORGE_WATCH_INTERVAL` | Method 3 | Poll interval in seconds (default: `60`) |
