"use client";

import { useInView } from "@/hooks/useInView";
import { Terminal, Zap } from "lucide-react";
import { useState } from "react";

function CodeBlock({ code, language = "bash" }: { code: string; language?: string }) {
  return (
    <div className="rounded-lg overflow-hidden border border-white/10 bg-code-bg">
      <div className="flex items-center gap-2 px-4 py-2 bg-white/[0.03] border-b border-white/5">
        <Terminal className="w-3.5 h-3.5 text-muted" />
        <span className="text-xs text-muted font-mono">{language}</span>
      </div>
      <pre className="p-4 text-sm font-mono text-white/80 overflow-x-auto leading-relaxed">
        <code>{code}</code>
      </pre>
    </div>
  );
}

const CONSUMER_STEPS = [
  {
    label: "Prerequisites",
    description: "Only Docker is required. No Rust, no compiling, no build tools.",
    code: `# That's it — just Docker.\ndocker --version`,
    language: "bash",
  },
  {
    label: "Configure your model",
    description:
      "Copy the example config and fill in your model credentials. Any OpenAI-compatible API works — OpenAI, Gemini, local models, and more.",
    code: `cp .env.example .env\n\n# Open .env and set:\nFORGE_MODEL=your-model-name\nFORGE_BASE_URL=https://your-provider.example.com/v1\nFORGE_API_KEY=your-api-key`,
    language: ".env",
  },
  {
    label: "See what issues are open",
    description:
      "Forge scans the repo and lists open issues. You choose which one to fix — nothing is hardcoded.",
    code: `docker compose run --rm list-issues --repo owner/repo\n\n# Output:\n# #3   Add input validation to signup form  [bug]\n# #7   Dark mode flicker on page load\n# #12  Upgrade to Node 20`,
    language: "bash",
  },
  {
    label: "Fix an issue",
    description: "Pick an issue number from the list. Forge clones the repo, works autonomously, and produces a verified git diff patch.",
    code: `# One-shot fix\ndocker compose run --rm forge run --repo owner/repo --issue 3\n\n# Or fix continuously — label any issue "forge"\n# and it gets picked up automatically:\ndocker compose up watch`,
    language: "bash",
  },
];

const DEV_STEPS = [
  {
    label: "Prerequisites",
    description: "Docker 24+ and Rust 1.82+ are required. Build the sandbox image once:",
    code: `docker build -f Dockerfile.sandbox -t forge-sandbox:latest .`,
    language: "bash",
  },
  {
    label: "Clone & build",
    description: "Clone the repository and compile the release binary:",
    code: `git clone https://github.com/OkeyAmy/forge && cd forge\ncargo build --release -p forge`,
    language: "bash",
  },
  {
    label: "Configure credentials",
    description: "Copy .env.example to .env and set your model credentials:",
    code: `cp .env.example .env\n\n# Edit .env:\nFORGE_MODEL=your-model-name\nFORGE_BASE_URL=https://your-provider.example.com/v1\nFORGE_API_KEY=your-api-key`,
    language: ".env",
  },
  {
    label: "List issues and run",
    description: "Discover open issues on any repo, then fix the one you choose:",
    code: `# See open issues\n./target/release/forge list-issues --repo owner/repo\n\n# Fix a specific one\n./target/release/forge run --repo owner/repo --issue 12\n\n# Or run continuously\n./target/release/forge watch --repo owner/repo --label forge`,
    language: "bash",
  },
];

export function Setup() {
  const { ref, inView } = useInView();
  const [mode, setMode] = useState<"consumer" | "dev">("consumer");
  const steps = mode === "consumer" ? CONSUMER_STEPS : DEV_STEPS;

  return (
    <section id="setup" className="py-24 px-4">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-3xl sm:text-4xl font-bold">
            Get{" "}
            <span className="bg-gradient-to-r from-accent-blue to-accent-purple bg-clip-text text-transparent">
              started
            </span>
          </h2>
          <p className="mt-4 text-muted max-w-xl mx-auto">
            You choose the issues. Forge does the work.
          </p>

          {/* Mode toggle */}
          <div className="mt-8 inline-flex items-center gap-1 p-1 rounded-xl bg-white/[0.04] border border-white/10">
            <button
              onClick={() => setMode("consumer")}
              className={`flex items-center gap-2 px-5 py-2 rounded-lg text-sm font-medium transition-all ${
                mode === "consumer"
                  ? "bg-gradient-to-r from-accent-blue to-accent-purple text-white shadow"
                  : "text-muted hover:text-white"
              }`}
            >
              <Zap className="w-3.5 h-3.5" />
              Docker (recommended)
            </button>
            <button
              onClick={() => setMode("dev")}
              className={`flex items-center gap-2 px-5 py-2 rounded-lg text-sm font-medium transition-all ${
                mode === "dev"
                  ? "bg-gradient-to-r from-accent-blue to-accent-purple text-white shadow"
                  : "text-muted hover:text-white"
              }`}
            >
              <Terminal className="w-3.5 h-3.5" />
              Build from source
            </button>
          </div>

          {mode === "consumer" && (
            <p className="mt-4 text-xs text-muted/60 font-mono">
              No Rust · No compiling · Just Docker + .env
            </p>
          )}
        </div>

        <div ref={ref} className="space-y-10">
          {steps.map((step, i) => (
            <div
              key={`${mode}-${step.label}`}
              style={{
                opacity: inView ? 1 : 0,
                transform: inView ? "translateY(0)" : "translateY(20px)",
                transition: `opacity 0.5s ease ${i * 0.1}s, transform 0.5s ease ${i * 0.1}s`,
              }}
              className="flex gap-6"
            >
              <div className="flex flex-col items-center">
                <span className="w-8 h-8 rounded-full bg-gradient-to-br from-accent-blue to-accent-purple flex items-center justify-center text-sm font-bold text-white flex-shrink-0">
                  {i + 1}
                </span>
                {i < steps.length - 1 && (
                  <div className="w-px flex-1 mt-2 bg-gradient-to-b from-accent-blue/30 to-transparent" />
                )}
              </div>
              <div className="flex-1 pb-2">
                <h3 className="font-semibold text-white mb-1">{step.label}</h3>
                <p className="text-sm text-muted mb-3">{step.description}</p>
                <CodeBlock code={step.code} language={step.language} />
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
