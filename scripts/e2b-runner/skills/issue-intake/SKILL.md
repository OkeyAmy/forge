# Issue Intake Skill

Use this when Forge receives a GitHub issue, label event, or `/forge plan`.

## Responsibility

Turn the GitHub issue into an engineering task without inventing scope.

## Process

1. Read the issue title and body.
2. Identify the requested outcome, constraints, and explicit "do not" instructions.
3. Treat missing details as uncertainty, not permission to guess.
4. Prefer the smallest useful change that satisfies the issue.
5. If the issue asks for approval before implementation, stop at planning.

## Output

- A concise task statement.
- Any blockers or missing information.
- A branch name tied to the issue.

## Failure Handling

If the issue cannot be understood, explain what is missing in the GitHub thread
and do not create an implementation plan.
