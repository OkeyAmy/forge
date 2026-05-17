# Approval Skill

Use this when Forge receives `/forge approve`.

## Responsibility

Ensure implementation starts only from a real waiting plan.

## Process

1. Find the latest waiting plan for the issue.
2. Reuse its branch, checks, and inspected context.
3. Mark the plan as approved before starting implementation.
4. If no waiting plan exists, ask the maintainer to run `/forge plan`.

## Output

- Approved plan state.
- Implementation branch.
- Validation commands inherited from the plan.

## Failure Handling

Do not implement from a stale, missing, failed, or simulated plan.
