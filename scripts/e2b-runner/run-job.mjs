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

  const checks = []
  if (input.work_command) {
    const command = [
      `FORGE_ISSUE_FILE=${shellQuote(issuePath)}`,
      `FORGE_ISSUE_NUMBER=${shellQuote(issue.number)}`,
      input.work_command,
    ].join(' ')
    checks.push(await run(sandbox, command, repoDir))
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
} catch (error) {
  console.error(error?.stack || String(error))
  process.exitCode = 1
} finally {
  if (sandbox) {
    await sandbox.kill()
  }
}
