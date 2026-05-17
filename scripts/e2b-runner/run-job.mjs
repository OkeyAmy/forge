import { readFileSync } from 'node:fs'
import { Sandbox } from 'e2b'

const input = JSON.parse(readFileSync(0, 'utf8'))
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

const parseModelJson = (content) => {
  const text = String(content || '').trim()
  const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i)
  const candidate = fenced
    ? fenced[1].trim()
    : text.slice(text.indexOf('{'), text.lastIndexOf('}') + 1).trim()
  if (!candidate) {
    throw new Error('model response did not include a JSON object')
  }
  return JSON.parse(candidate)
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
          'Malformed response:',
          truncate(content, 12000),
        ].join('\n'),
      },
    ])
    return { parsed: parseModelJson(repaired), content: repaired }
  }
}

const modelActionFromJson = (parsed) => {
  return {
    done: Boolean(parsed.done),
    commands: Array.isArray(parsed.commands) ? parsed.commands.map(String).filter(Boolean).slice(0, 5) : [],
    notes: String(parsed.notes || ''),
  }
}

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
    const summaryPrompt = [
      'You are analyzing a codebase structure. Based on these exploration results, produce a concise summary.',
      '',
      `Repository: ${input.repository.owner}/${input.repository.name}`,
      `Default branch: ${input.repository.default_branch}`,
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
      '  "entry_points": ["main entry points of the application"],',
      '  "notable_patterns": ["notable patterns or conventions observed"]',
      '}',
    ].join('\n')

    try {
      const { parsed } = await callModelJson(
        input.model,
        [
          { role: 'system', content: 'You are a codebase analysis tool. Return ONLY valid JSON. No markdown, no explanation.' },
          { role: 'user', content: summaryPrompt },
        ],
        '{"language": string, "framework": string|null, "structure_summary": string, "key_directories": string[], "test_setup": string|null, "build_system": string, "entry_points": string[], "notable_patterns": string[]}',
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
  const messages = [
    {
      role: 'system',
      content: [
        'You are Forge running inside an E2B sandbox.',
        'You may inspect and edit the cloned repository only through shell commands.',
        'Return strict JSON only: {"done": boolean, "commands": ["shell command"], "notes": "short reason"}.',
        'Use commands to inspect files, edit code, and run focused tests.',
        'Do not print secrets or environment variables.',
        'Set done=true only after the repository has the intended code changes.',
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
        'Start by inspecting the repository, then make the smallest useful change.',
      ].join('\n'),
    },
  ]

  const checks = []
  for (let step = 1; step <= maxSteps; step += 1) {
    const { parsed, content } = await callModelJson(
      input.model,
      messages,
      '{"done": boolean, "commands": string[], "notes": string}',
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
      const result = await run(sandbox, command, repoDir)
      checks.push(result)
      observations.push([
        `$ ${command}`,
        `exit=${result.exit_code}`,
        `stdout:\n${truncate(result.stdout)}`,
        `stderr:\n${truncate(result.stderr)}`,
      ].join('\n'))
    }

    messages.push({ role: 'assistant', content })
    messages.push({
      role: 'user',
      content: [
        `Observation for step ${step}:`,
        observations.join('\n\n---\n\n'),
        '',
        'Continue with the next JSON action. If the fix is done, return done=true.',
      ].join('\n'),
    })
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

  for (const check of input.checks || []) {
    checks.push(await run(sandbox, check, repoDir))
  }

  const risks = []
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
