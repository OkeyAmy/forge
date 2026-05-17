# Planning Skill

Use this after issue intake and repository inspection.

## Responsibility

Create an approval-ready engineering plan that a maintainer can understand.

## Process

1. Explain the codebase context in plain language, not raw JSON.
2. Connect the issue request to the files or areas likely to change.
3. Propose ordered implementation steps.
4. List validation commands that fit the repository.
5. State risk using concrete repo and issue facts.
6. Wait for `/forge approve` before making changes when approval is required.

## Output

- Codebase context.
- What the issue needs.
- Proposed implementation path.
- Checks Forge will run.
- Risk.
- Branch name.
- Approval instruction.

## Failure Handling

If the model, sandbox, clone, or inspection fails, report failure and do not
create a waiting approval plan.
