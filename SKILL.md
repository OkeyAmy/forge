# Forge Repository Skill

Forge reads a repository skill file before planning or implementing work in E2B.
Projects can add one of these files:

- `SKILL.md`
- `.forge/SKILL.md`
- `.github/forge/SKILL.md`

Use the file to teach Forge how a human maintainer expects work to happen in that
repository.

## What Forge Looks For

Write short, concrete instructions:

- Project shape: important directories, entry points, generated files to avoid.
- Setup: package manager, install command, required services.
- Validation: the exact commands Forge should run for common changes.
- Coding rules: naming, formatting, architecture boundaries, test conventions.
- Review rules: what risks matter before opening a pull request.

## Example

```md
# Forge Skill

This is a Vite React app with API routes under `api/`.

## Before Editing

- Use `npm install` if dependencies are missing.
- Read `README.md`, `package.json`, and the touched feature files first.
- Do not edit generated assets in `dist/`.

## Validation

- For UI or TypeScript changes, run `npm run build`.
- For isolated component changes, run the nearest test if one exists.
- If no relevant test exists, explain that in the PR.

## Pull Requests

- Keep changes small.
- Mention commands run and any checks that could not be run.
```

## Forge Behavior

Forge still starts from the GitHub issue. The skill file is guidance, not a
replacement for the issue request. If the issue and skill conflict, Forge should
prefer the issue for product intent and the skill for repository mechanics.
