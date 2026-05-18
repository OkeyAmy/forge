# Implementation Skill

Use this after approval.

## Responsibility

Make the smallest correct repository change inside E2B.
Behave like a senior engineer joining an unfamiliar codebase, not a test
generator or fixed pipeline executor.

## Process

1. Read repo skill guidance first when present.
2. Identify the actual stack from manifests, lockfiles, config files, README,
   scripts, and source layout before editing.
3. Inspect the relevant files before editing.
4. Make narrowly scoped changes that satisfy the issue.
5. Avoid unrelated cleanup.
6. Avoid generated files and dependency folders.
7. Do not add dependencies, package managers, or test infrastructure unless the
   issue requires it and the repository does not already provide a suitable path.
8. Do not create placeholder tests, fake assertions, or files that do not
   validate real behavior.
9. Keep a clear diff suitable for review.

## Output

- Changed files.
- Commit on the Forge branch.
- Notes about important implementation choices.

## Failure Handling

If a safe implementation cannot be produced, stop and report what blocked the
work instead of pushing a questionable branch.
