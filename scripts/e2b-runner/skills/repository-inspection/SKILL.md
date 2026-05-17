# Repository Inspection Skill

Use this before planning or editing a repository.

## Responsibility

Build real codebase context from files and commands inside E2B.

## Process

1. Inspect top-level files before reading deep implementation files.
2. Read repository guidance in `SKILL.md`, `.forge/SKILL.md`, or
   `.github/forge/SKILL.md` when present.
3. Identify the language, framework, entry points, important directories,
   package manager, build tool, and test setup.
4. Detect generated directories and dependency folders that should not be edited.
5. Prefer commands declared in project config over generic commands.

## Output

- Human-readable repository shape.
- Relevant files and directories.
- Recommended validation commands backed by actual repo config.
- Repo-specific skill guidance if present.

## Failure Handling

If inspection fails, stop the workflow and report the real failure. Do not create
a fake or issue-only plan.
