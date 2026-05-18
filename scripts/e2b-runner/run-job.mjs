import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Sandbox } from 'e2b'

const input = JSON.parse(readFileSync(0, 'utf8'))
const scriptDir = dirname(fileURLToPath(import.meta.url))
const skillsDir = join(scriptDir, 'skills')
const timeoutMs = Number(process.env.E2B_TIMEOUT_MS || '900000')
const requestTimeoutMs = Number(process.env.E2B_REQUEST_TIMEOUT_MS || '60000')
const template = process.env.E2B_TEMPLATE || undefined
const attempts = Number(process.env.E2B_RUNNER_ATTEMPTS || '3')

const shellQuote = (value) => `'${String(value).replaceAll("'", "'\\''")}'`
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

const retry = async (label, operation) => {
  let lastError
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      return await operation()
    } catch (error) {
      lastError = error
      if (attempt < attempts) {
        console.error(`${label} failed on attempt ${attempt}/${attempts}: ${error?.message || error}`)
        await sleep(1000 * attempt)
      }
    }
  }
  throw lastError
}

const authenticatedCloneUrl = (cloneUrl, token) => {
  const url = new URL(cloneUrl)
  url.username = 'x-access-token'
  url.password = token
  return url.toString()
}

const truncate = (value, max = 6000) => {
  const text = String(value || '')
  return text.length > max ? `${text.slice(0, max)}\n[truncated]` : text
}

const readWorkflowSkill = (name) => {
  const path = join(skillsDir, name, 'SKILL.md')
  return existsSync(path) ? readFileSync(path, 'utf8').trim() : ''
}

const workflowSkillPack = (names) => names
  .map((name) => {
    const skill = readWorkflowSkill(name)
    return skill ? `## ${name}/SKILL.md\n${skill}` : ''
  })
  .filter(Boolean)
  .join('\n\n---\n\n')

const run = async (sandbox, command, cwd = undefined) => {
  const result = await retry(`command ${command}`, () =>
    sandbox.commands.run(command, { cwd, timeoutMs, requestTimeoutMs }),
  )
  const exitCode = Number(result.exitCode ?? 0)
  return {
    command,
    exit_code: exitCode,
    passed: exitCode === 0,
    stdout: String(result.stdout || ''),
    stderr: String(result.stderr || ''),
  }
}

const runForObservation = async (sandbox, command, cwd = undefined) => {
  const marker = '__FORGE_EXIT_CODE__'
  const wrapped = [
    'set +e',
    command,
    'status=$?',
    `printf '\\n${marker}:%s\\n' "$status"`,
    'exit 0',
  ].join('\n')
  const result = await retry(`model command ${command}`, () =>
    sandbox.commands.run(wrapped, { cwd, timeoutMs, requestTimeoutMs }),
  )
  const rawStdout = String(result.stdout || '')
  const match = rawStdout.match(new RegExp(`\\n${marker}:(\\d+)\\n?$`))
  const exitCode = match ? Number(match[1]) : Number(result.exitCode ?? 0)
  const stdout = match ? rawStdout.slice(0, match.index) : rawStdout
  return {
    command,
    exit_code: exitCode,
    passed: exitCode === 0,
    stdout,
    stderr: String(result.stderr || ''),
  }
}

const changedFilesInRepo = async (sandbox, repoDir) => {
  const result = await run(sandbox, 'git diff --name-only', repoDir)
  return result.stdout
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
}

const commandProgram = (command) => {
  const match = String(command || '').trim().match(/^([A-Za-z0-9._/-]+)/)
  return match ? match[1] : ''
}

const validationCheckSkipReason = async (sandbox, command, repoDir) => {
  const program = commandProgram(command)
  if (!program) return 'empty validation command'

  const manifestChecks = [
    { pattern: /^cargo\b/, manifest: 'Cargo.toml', label: 'Rust Cargo manifest' },
    { pattern: /^pnpm\b/, manifest: 'package.json', label: 'Node package manifest' },
    { pattern: /^npm\b/, manifest: 'package.json', label: 'Node package manifest' },
    { pattern: /^yarn\b/, manifest: 'package.json', label: 'Node package manifest' },
    { pattern: /^bun\b/, manifest: 'package.json', label: 'Node package manifest' },
    { pattern: /^python\b|^python3\b|^pytest\b|^pip\b/, manifest: 'pyproject.toml requirements.txt setup.py', label: 'Python project manifest' },
  ]
  for (const check of manifestChecks) {
    if (!check.pattern.test(command)) continue
    const expression = check.manifest
      .split(' ')
      .map((file) => `[ -f ${shellQuote(file)} ]`)
      .join(' || ')
    const result = await runForObservation(sandbox, expression, repoDir)
    if (result.exit_code !== 0) {
      return `${check.label} was not found in the repository`
    }
  }

  const binaryCheck = await runForObservation(sandbox, `command -v ${shellQuote(program)}`, repoDir)
  if (binaryCheck.exit_code !== 0) {
    return `${program} is not installed in the E2B template`
  }
  return ''
}

const skippedValidationCheck = (command, reason) => ({
  command,
  exit_code: 0,
  passed: true,
  stdout: `Forge skipped this validation command: ${reason}.`,
  stderr: '',
  skipped: true,
  skip_reason: reason,
})

const callModel = async (modelConfig, messages) => {
  const baseUrl = String(modelConfig.base_url).replace(/\/$/, '')
  const response = await fetch(`${baseUrl}/chat/completions`, {
    method: 'POST',
    headers: {
      authorization: `Bearer ${modelConfig.api_key}`,
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      model: modelConfig.model,
      messages,
      temperature: 0.1,
      max_tokens: 4096,
    }),
  })
  if (!response.ok) {
    throw new Error(`model request failed ${response.status}: ${await response.text()}`)
  }
  const json = await response.json()
  const content = json?.choices?.[0]?.message?.content
  if (!content) {
    throw new Error('model response did not include choices[0].message.content')
  }
  return content
}

const extractOutermostJson = (text) => {
  const start = text.indexOf('{')
  if (start === -1) return ''
  let depth = 0
  let inString = false
  let escaped = false
  for (let i = start; i < text.length; i++) {
    const ch = text[i]
    if (escaped) { escaped = false; continue }
    if (ch === '\\' && inString) { escaped = true; continue }
    if (ch === '"') { inString = !inString; continue }
    if (inString) continue
    if (ch === '{') depth++
    else if (ch === '}') {
      depth--
      if (depth === 0) return text.slice(start, i + 1)
    }
  }
  return ''
}

const parseModelJson = (content) => {
  const text = String(content || '').trim()
  // Prefer fenced code blocks (```json ... ``` or ``` ... ```)
  const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i)
  const candidate = fenced
    ? fenced[1].trim()
    : extractOutermostJson(text)
  if (!candidate) {
    throw new Error('model response did not include a JSON object')
  }
  try {
    return JSON.parse(candidate)
  } catch (error) {
    try {
      return JSON.parse(escapeInvalidJsonEscapes(candidate))
    } catch {
      throw error
    }
  }
}

const escapeInvalidJsonEscapes = (value) => {
  let output = ''
  for (let i = 0; i < value.length; i++) {
    const ch = value[i]
    if (ch !== '\\') {
      output += ch
      continue
    }
    const next = value[i + 1]
    if (!next) {
      output += '\\\\'
      continue
    }
    if ('"\\/bfnrtu'.includes(next)) {
      output += ch
    } else {
      output += '\\\\'
    }
  }
  return output
}

const callModelJson = async (modelConfig, messages, schemaDescription) => {
  const content = await callModel(modelConfig, messages)
  try {
    return { parsed: parseModelJson(content), content }
  } catch (error) {
    const repaired = await callModel(modelConfig, [
      {
        role: 'system',
        content: 'You repair malformed model output into valid JSON. Return ONLY valid JSON. No markdown, no explanation.',
      },
      {
        role: 'user',
        content: [
          `The previous response was not valid JSON: ${error?.message || String(error)}`,
          '',
          'Expected JSON shape:',
          schemaDescription,
          '',
          'Rules:',
          '- Return one JSON object only.',
          '- Put shell commands in string values.',
          '- Escape backslashes as \\\\ inside JSON strings.',
          '- Do not use markdown fences.',
          '',
          'Malformed response:',
          truncate(content, 12000),
        ].join('\n'),
      },
    ])
    try {
      return { parsed: parseModelJson(repaired), content: repaired }
    } catch (repairError) {
      throw new Error(
        `model returned invalid JSON after repair: ${repairError?.message || String(repairError)}; original parse error: ${error?.message || String(error)}`,
      )
    }
  }
}

const modelActionFromJson = (parsed) => {
  return {
    done: Boolean(parsed.done),
    commands: Array.isArray(parsed.commands) ? parsed.commands.map(String).filter(Boolean).slice(0, 5) : [],
    notes: String(parsed.notes || ''),
  }
}

const callActionJson = async (modelConfig, messages, schemaDescription, step) => {
  let lastError
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    try {
      return await callModelJson(modelConfig, messages, schemaDescription)
    } catch (error) {
      lastError = error
      messages.push({
        role: 'user',
        content: [
          `Your previous action for step ${step} was not usable JSON: ${error?.message || String(error)}`,
          'Return only this exact JSON shape:',
          schemaDescription,
          'Use valid JSON escaping. Backslashes inside shell commands must be doubled.',
        ].join('\n'),
      })
    }
  }
  throw lastError
}

const shouldSkipModelCommand = (command) => {
  const normalized = String(command || '').trim().replace(/\s+/g, ' ')
  return [
    /^git checkout (-b|-B)\b/,
    /^git switch (-c|-C)\b/,
    /^git branch\b/,
    /^git commit\b/,
    /^git push\b/,
    /^git remote\b/,
    /^(pnpm|npm|yarn|bun) (add|install|i)\b/,
  ].some((pattern) => pattern.test(normalized))
}

const skippedModelCommand = (command) => ({
  command,
  exit_code: 0,
  passed: true,
  stdout: [
    'Forge skipped this command because branch control, commits, pushes, remotes, and dependency installation are managed by the runner.',
    'Continue by inspecting files, editing the repository, and running existing validation commands.',
  ].join('\n'),
  stderr: '',
})

const runCodebaseExploration = async (sandbox, repoDir, input) => {
  // Exploration commands that run inside the sandbox to understand the codebase
  const explorationCommands = [
    { name: 'file_tree', cmd: 'find . -maxdepth 3 -type f | grep -v node_modules | grep -v \\.git | grep -v __pycache__ | grep -v target/ | head -150' },
    { name: 'package_json', cmd: 'cat package.json 2>/dev/null || echo "NO_PACKAGE_JSON"' },
    { name: 'cargo_toml', cmd: 'cat Cargo.toml 2>/dev/null || echo "NO_CARGO_TOML"' },
    { name: 'requirements', cmd: 'cat requirements.txt 2>/dev/null; cat pyproject.toml 2>/dev/null; echo "---"; cat setup.py 2>/dev/null || echo "NO_PYTHON_DEPS"' },
    { name: 'docker_config', cmd: 'cat Dockerfile 2>/dev/null; echo "---"; cat docker-compose.yml 2>/dev/null || echo "NO_DOCKER"' },
    { name: 'test_files', cmd: 'find . -type f \\( -name "*.test.*" -o -name "*.spec.*" -o -name "*_test.*" -o -name "test_*" \\) | grep -v node_modules | grep -v \\.git | head -30' },
    { name: 'readme', cmd: 'head -80 README.md 2>/dev/null || echo "NO_README"' },
    { name: 'skill_md', cmd: 'cat SKILL.md 2>/dev/null || cat .forge/SKILL.md 2>/dev/null || cat .github/forge/SKILL.md 2>/dev/null || echo "NO_SKILL_MD"' },
    { name: 'src_structure', cmd: 'ls -la src/ 2>/dev/null; echo "---"; ls -la lib/ 2>/dev/null; echo "---"; ls -la crates/ 2>/dev/null || echo "NO_SRC_LIB_CRATES"' },
    { name: 'config_files', cmd: 'find . -maxdepth 2 -type f \\( -name "*.json" -o -name "*.yaml" -o -name "*.yml" -o -name "*.toml" -o -name "*.env*" -o -name ".eslintrc*" -o -name ".prettierrc*" -o -name "tsconfig*" \\) | grep -v node_modules | grep -v \\.git | head -20' },
    { name: 'entry_points', cmd: 'cat package.json 2>/dev/null | grep -A5 "\"scripts\"" || echo "NO_NPM_SCRIPTS"; echo "---"; cat Cargo.toml 2>/dev/null | grep -A2 "\\[\\[bin\\]\\]" || echo "NO_RUST_BINS"' },
  ]

  const results = {}
  for (const { name, cmd } of explorationCommands) {
    try {
      const result = await sandbox.commands.run(cmd, { cwd: repoDir, timeoutMs: 15000, requestTimeoutMs: 10000 })
      results[name] = String(result.stdout || '').trim().slice(0, 4000)
    } catch (err) {
      results[name] = `[error: ${err?.message || String(err)}]`
    }
  }

  // Use model to synthesize exploration into a structured summary
  if (input.model) {
    const planningSkills = workflowSkillPack(['issue-intake', 'repository-inspection', 'planning', 'validation', 'github-communication'])
    const summaryPrompt = [
      'You are Forge planning a real GitHub issue workflow. Use the workflow skills and repository exploration to produce a maintainer-readable engineering plan.',
      '',
      `Repository: ${input.repository.owner}/${input.repository.name}`,
      `Default branch: ${input.repository.default_branch}`,
      `Issue #${input.issue.number}: ${input.issue.title}`,
      input.issue.body || 'No issue body was provided.',
      '',
      '## Forge Workflow Skills',
      planningSkills,
      '',
      '## Exploration Data',
      ...Object.entries(results).map(([name, data]) => `### ${name}\n${data}`),
      '',
      'Return strict JSON only:',
      '{',
      '  "language": "primary language detected",',
      '  "framework": "framework/library if detected, else null",',
      '  "structure_summary": "2-3 sentence summary of the codebase architecture",',
      '  "key_directories": ["list of important directories and their purpose"],',
      '  "test_setup": "how tests are run, or null if no tests found",',
      '  "build_system": "how the project builds",',
      '  "recommended_checks": ["commands Forge should run for this issue, using scripts/config actually present in the repo"],',
      '  "entry_points": ["main entry points of the application"],',
      '  "notable_patterns": ["notable patterns or conventions observed"],',
      '  "implementation_plan": ["short ordered engineering steps for this issue"],',
      '  "risk_level": "low|medium|high",',
      '  "risk_notes": "specific risk note for this issue",',
      '  "skill_instructions": "summary of SKILL.md guidance if present, else null"',
      '}',
    ].join('\n')

    try {
      const { parsed } = await callModelJson(
        input.model,
        [
          { role: 'system', content: 'You are a codebase analysis tool. Return ONLY valid JSON. No markdown, no explanation.' },
          { role: 'user', content: summaryPrompt },
        ],
        '{"language": string, "framework": string|null, "structure_summary": string, "key_directories": string[], "test_setup": string|null, "build_system": string, "recommended_checks": string[], "entry_points": string[], "notable_patterns": string[], "implementation_plan": string[], "risk_level": "low|medium|high", "risk_notes": string, "skill_instructions": string|null}',
      )
      results.synthesized_summary = parsed
    } catch (err) {
      results.synthesized_summary = { error: `Model synthesis failed: ${err?.message || String(err)}` }
    }
  }

  return results
}

const runAutonomousEdit = async (sandbox, repoDir, issuePath, input) => {
  if (!input.model) {
    throw new Error('FORGE_E2B_WORK_COMMAND or FORGE_MODEL/FORGE_BASE_URL/FORGE_API_KEY is required for E2B execution')
  }

  const maxSteps = Number(input.max_steps || 6)
  const implementationSkills = workflowSkillPack(['approval', 'implementation', 'validation', 'review', 'pull-request', 'github-communication'])
  const messages = [
    {
      role: 'system',
      content: [
        'You are Forge running inside an E2B sandbox.',
        'You are not a generic coding chatbot. You are executing a professional software engineering pipeline.',
        'Follow the Forge workflow skills below for how to inspect, implement, validate, review, and prepare a PR.',
        '',
        implementationSkills,
        '',
        'You may inspect and edit the cloned repository only through shell commands.',
        'Return strict JSON only: {"done": boolean, "commands": ["shell command"], "notes": "short reason"}.',
        'Use commands to inspect files, edit code, and run focused tests.',
        'Do not create, checkout, commit, push, or rename git branches. Forge already checked out the implementation branch and will commit, push, and open the PR after your edits.',
        'Do not install new dependencies or package managers. Use the tools already present in the repository and sandbox.',
        'If the repository contains SKILL.md, .forge/SKILL.md, or .github/forge/SKILL.md, read it first and follow its repo-specific instructions.',
        'Do not print secrets or environment variables.',
        'Set done=true after the repository has the intended code changes. Do not keep exploring once a focused diff exists.',
      ].join('\n'),
    },
    {
      role: 'user',
      content: [
        `Repository: ${input.repository.owner}/${input.repository.name}`,
        `Default branch: ${input.repository.default_branch}`,
        `Issue #${input.issue.number}: ${input.issue.title}`,
        `Issue file path: ${issuePath}`,
        '',
        input.issue.body || 'No issue body was provided.',
        '',
        'Start by inspecting the repository and any Forge SKILL.md file, then make the smallest useful change.',
      ].join('\n'),
    },
  ]

  const checks = []
  for (let step = 1; step <= maxSteps; step += 1) {
    const { parsed, content } = await callActionJson(
      input.model,
      messages,
      '{"done": boolean, "commands": string[], "notes": string}',
      step,
    )
    const action = modelActionFromJson(parsed)
    if (action.done) {
      checks.push({ command: `forge-model-step-${step}`, exit_code: 0, passed: true, stdout: action.notes, stderr: '' })
      return checks
    }
    if (action.commands.length === 0) {
      throw new Error(`model step ${step} did not provide commands`)
    }

    const observations = []
    for (const command of action.commands) {
      const result = shouldSkipModelCommand(command)
        ? skippedModelCommand(command)
        : await runForObservation(sandbox, command, repoDir)
      checks.push(result)
      observations.push([
        `$ ${command}`,
        `exit=${result.exit_code}`,
        `stdout:\n${truncate(result.stdout)}`,
        `stderr:\n${truncate(result.stderr)}`,
      ].join('\n'))
    }

    const changedFiles = await changedFilesInRepo(sandbox, repoDir)
    if (changedFiles.length > 0) {
      checks.push({
        command: `forge-auto-finish-after-step-${step}`,
        exit_code: 0,
        passed: true,
        stdout: [
          'Forge detected repository changes and is moving to validation.',
          '',
          ...changedFiles.map((file) => `- ${file}`),
        ].join('\n'),
        stderr: '',
      })
      return checks
    }

    messages.push({ role: 'assistant', content })
    messages.push({
      role: 'user',
      content: [
        `Observation for step ${step}:`,
        observations.join('\n\n---\n\n'),
        '',
        'Continue with the next JSON action. If the fix is done or a focused diff exists, return done=true.',
      ].join('\n'),
    })
  }

  const changedFiles = await changedFilesInRepo(sandbox, repoDir)
  if (changedFiles.length > 0) {
    checks.push({
      command: 'forge-auto-finish-at-max-steps',
      exit_code: 0,
      passed: true,
      stdout: [
        `Forge reached the ${maxSteps} step limit after producing repository changes, so it is moving to validation instead of failing the run.`,
        '',
        ...changedFiles.map((file) => `- ${file}`),
      ].join('\n'),
      stderr: '',
    })
    return checks
  }

  throw new Error(`model did not finish within ${maxSteps} E2B steps`)
}

let sandbox
try {
  sandbox = await retry('sandbox create', () =>
    Sandbox.create(template ? { template, requestTimeoutMs } : { requestTimeoutMs }),
  )

  const repo = input.repository
  const issue = input.issue
  const branchName = input.branch_name
  const cloneUrl = authenticatedCloneUrl(repo.clone_url, input.installation_token)
  const repoDir = `/home/user/${repo.name}`

  await run(sandbox, 'git config --global user.name forge-app')
  await run(sandbox, 'git config --global user.email forge-app@users.noreply.github.com')
  await run(sandbox, `git clone ${shellQuote(cloneUrl)} ${shellQuote(repoDir)}`)
  await run(sandbox, `git checkout -B ${shellQuote(branchName)} origin/${shellQuote(repo.default_branch)}`, repoDir)

  const issueMarkdown = [
    `# Issue #${issue.number}: ${issue.title}`,
    '',
    issue.body || 'No issue body was provided.',
    '',
  ].join('\n')
  const issuePath = '/home/user/forge-issue.md'
  const issueBase64 = Buffer.from(issueMarkdown, 'utf8').toString('base64')
  await run(sandbox, `printf %s ${shellQuote(issueBase64)} | base64 -d > ${shellQuote(issuePath)}`)

  // Exploration mode: analyze codebase and return findings without making changes
  if (input.mode === 'explore') {
    const exploration = await runCodebaseExploration(sandbox, repoDir, input)
    const output = {
      mode: 'exploration',
      repository: `${repo.owner}/${repo.name}`,
      branch: repo.default_branch,
      exploration,
    }
    console.log(JSON.stringify(output))
    process.exitCode = 0
  } else {
    const checks = []
    if (input.work_command) {
      const command = [
        `FORGE_ISSUE_FILE=${shellQuote(issuePath)}`,
        `FORGE_ISSUE_NUMBER=${shellQuote(issue.number)}`,
        input.work_command,
      ].join(' ')
      checks.push(await run(sandbox, command, repoDir))
    } else {
      checks.push(...(await runAutonomousEdit(sandbox, repoDir, issuePath, input)))
    }

    const diffBeforeChecks = await run(sandbox, 'git diff --name-only', repoDir)
    const changedFiles = diffBeforeChecks.stdout
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean)

    const risks = []
    for (const check of input.checks || []) {
      const skipReason = await validationCheckSkipReason(sandbox, check, repoDir)
      if (skipReason) {
        checks.push(skippedValidationCheck(check, skipReason))
        risks.push(`Skipped validation command \`${check}\`: ${skipReason}.`)
      } else {
        checks.push(await runForObservation(sandbox, check, repoDir))
      }
    }

    if (changedFiles.length === 0) {
      risks.push('No code changes were produced in the E2B sandbox.')
    }
    for (const check of checks) {
      if (!check.passed) {
        risks.push(`Command failed: ${check.command}`)
      }
    }

    if (changedFiles.length > 0) {
      await run(sandbox, 'git add -A', repoDir)
      await run(sandbox, `git commit -m ${shellQuote(`Forge issue #${issue.number}`)}`, repoDir)
      await run(sandbox, `git push origin HEAD:${shellQuote(branchName)}`, repoDir)
    }

    const output = {
      branch_name: branchName,
      compare_url: `https://github.com/${repo.owner}/${repo.name}/compare/${repo.default_branch}...${branchName}`,
      changed_files: changedFiles,
      checks: checks.map(({ command, exit_code, passed }) => ({ command, exit_code, passed })),
      risks,
    }
    console.log(JSON.stringify(output))
  }
} catch (error) {
  console.error(error?.stack || String(error))
  process.exitCode = 1
} finally {
  if (sandbox) {
    await sandbox.kill()
  }
}
