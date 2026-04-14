# Method 2 — YAML config file

More control: define the model, sandbox, system prompt, and issue in a single YAML file.
Good for teams who want reproducible, version-controlled runs.

**Issue being fixed:** [#28 — Add formatTokenAmount()](https://github.com/OkeyAmy/Axioschat-Onboard/issues/28)

## Setup

```bash
cp ../. env.example .env
# Edit .env: set FORGE_API_KEY, GITHUB_TOKEN, DOCKER_GID

docker compose run --rm forge
```

## Or run with the binary directly

```bash
# If you have Forge built locally:
set -a && source .env && set +a
./../../target/release/forge run --config forge-issue28.yaml
```

## Key difference from Method 1

The YAML config lets you:
- Pin a specific model version
- Customise the system + instance prompt
- Set `max_steps` per run
- Choose `parser_type` (thought_action / action_only / function_calling)
- Commit the config file so every run is reproducible
