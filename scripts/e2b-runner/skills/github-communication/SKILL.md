# GitHub Communication Skill

Use this whenever Forge posts to an issue or pull request thread.

## Responsibility

Communicate like a clear engineering teammate.

## Process

1. Use human-readable Markdown.
2. Do not dump raw JSON, stack traces, or provider responses into normal plans.
3. Keep status comments short and specific.
4. In failure comments, include the real failure and the next useful action.
5. In plan comments, make approval state obvious.
6. In completion comments, include changed files, checks, risks, and PR link.

## Output

- Maintainer-readable GitHub comments.
- No simulated fallback plans.
- No secret values.

## Failure Handling

If a comment cannot be posted, keep the job state accurate so operators can
recover without misleading the user.
