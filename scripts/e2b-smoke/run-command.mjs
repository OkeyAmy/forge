import { Sandbox } from 'e2b'

const template = process.env.E2B_TEMPLATE || undefined
const timeoutMs = Number(process.env.E2B_TIMEOUT_MS || '120000')
const requestTimeoutMs = Number(process.env.E2B_REQUEST_TIMEOUT_MS || '60000')
const attempts = Number(process.env.E2B_SMOKE_ATTEMPTS || '3')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

async function retry(label, operation) {
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

let sandbox
try {
  sandbox = await retry('sandbox create', () =>
    Sandbox.create(template ? { template, requestTimeoutMs } : { requestTimeoutMs }),
  )
  const result = await retry('sandbox command', () =>
    sandbox.commands.run('echo forge-e2b-live', {
      timeoutMs,
      requestTimeoutMs,
    }),
  )

  const stdout = String(result.stdout || '').trim()
  if (stdout !== 'forge-e2b-live') {
    console.error(`unexpected stdout: ${JSON.stringify(result.stdout)}`)
    process.exitCode = 1
  } else {
    console.log(`sandbox=${sandbox.sandboxId}`)
    console.log(`stdout=${stdout}`)
  }
} finally {
  if (sandbox) {
    await sandbox.kill()
  }
}
