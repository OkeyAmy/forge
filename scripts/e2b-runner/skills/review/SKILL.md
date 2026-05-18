# Review Skill

Use this before branch handoff and for `/forge review`.

## Responsibility

Review the diff like a professional software engineer before asking humans to
merge it.

## Process

1. Inspect changed files and the diff.
2. Check for incorrect scope, broken contracts, missing validation, security
   issues, and maintainability risks.
3. Confirm the implementation still matches the issue.
4. Reject placeholder tests, fake assertions, unrelated ecosystems, and invented
   test infrastructure.
5. Prefer concrete findings over generic advice.

## Output

- Summary of the change.
- Risks or review notes.
- Test gaps.
- Merge readiness.

## Failure Handling

If the diff cannot be reviewed, say so and include the command or API failure.
