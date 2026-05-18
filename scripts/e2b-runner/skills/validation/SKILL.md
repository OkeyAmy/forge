# Validation Skill

Use this during planning and after implementation.

## Responsibility

Choose and run checks that prove the change is reasonable for the repository.

## Process

1. Prefer repo-declared commands from `package.json`, `Cargo.toml`,
   `pyproject.toml`, CI config, or repo skill files.
2. Match checks to the changed surface.
3. Run focused checks before broad checks when time is limited.
4. Report commands, exit codes, and failures honestly.
5. If no check exists, say that explicitly.
6. Never run commands for unrelated ecosystems. For example, do not run
   `cargo test --workspace` without `Cargo.toml`, and do not run Python tools
   without a Python project manifest.
7. Do not install test frameworks just to create validation. Use existing
   project scripts unless the issue explicitly requires new test infrastructure.

## Output

- Recommended checks during planning.
- Actual command results after implementation.
- Residual risk when checks could not run.

## Failure Handling

Never claim verification passed unless the command ran and exited successfully.
