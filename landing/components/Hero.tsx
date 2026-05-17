import { ArrowRight, CheckCircle2, Github, ShieldCheck } from "lucide-react";
import { GITHUB_APP_INSTALL_URL } from "@/lib/links";

export function Hero() {
  return (
    <section className="relative min-h-screen flex flex-col items-center justify-center px-4 overflow-hidden">
      <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
        <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-accent-blue/60 to-transparent" />
        <div className="absolute left-1/2 top-1/3 h-[520px] w-[520px] -translate-x-1/2 rounded-full bg-gradient-radial from-accent-blue/16 via-accent-purple/8 to-transparent animate-glow-pulse" />
      </div>

      <div className="relative w-full max-w-6xl pt-20 pb-16">
        <div className="grid gap-12 lg:grid-cols-[1.02fr_0.98fr] lg:items-center">
          <div>
            <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-emerald-400/25 bg-emerald-400/8 px-3 py-1 text-xs font-mono text-emerald-300">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-300" />
              GitHub App · E2B sandbox · PR workflow
            </div>

            <h1 className="max-w-4xl text-5xl font-bold leading-tight tracking-tight sm:text-6xl lg:text-7xl">
              Add Forge to GitHub. Ship from issues.
            </h1>

            <p className="mt-6 max-w-2xl text-lg leading-relaxed text-muted sm:text-xl">
              Install the GitHub App on a repo, mention Forge on an issue, review
              the plan, then approve execution. Forge inspects the code in E2B,
              pushes a branch, and opens a pull request for you to review.
            </p>

            <div className="mt-10 flex flex-col gap-3 sm:flex-row">
              <a
                href={GITHUB_APP_INSTALL_URL}
                className="group inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-white px-6 text-sm font-semibold text-black transition hover:bg-white/90"
              >
                <Github className="h-4 w-4" />
                Install GitHub App
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </a>
              <a
                href="#setup"
                className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg border border-white/12 px-6 text-sm font-semibold text-white/85 transition hover:border-white/25 hover:text-white"
              >
                See workflow
              </a>
            </div>

            <div className="mt-8 grid gap-3 text-sm text-white/72 sm:grid-cols-3">
              {["No user setup after install", "Approval before code changes", "Works through PRs"].map((item) => (
                <div key={item} className="flex items-center gap-2">
                  <CheckCircle2 className="h-4 w-4 text-emerald-300" />
                  <span>{item}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="rounded-lg border border-white/10 bg-[#111318]/85 shadow-2xl shadow-black/35">
            <div className="flex items-center justify-between border-b border-white/8 px-5 py-4">
              <div className="flex items-center gap-2 text-sm font-semibold">
                <ShieldCheck className="h-4 w-4 text-emerald-300" />
                Repository activation
              </div>
              <span className="rounded-full border border-emerald-400/20 bg-emerald-400/8 px-2 py-0.5 text-xs text-emerald-300">
                Ready
              </span>
            </div>
            <div className="space-y-0 p-5">
              {[
                ["1", "Install Forge", "Choose the repositories Forge can access."],
                ["2", "Create an issue", "Add the `forge` label or comment `/forge plan`."],
                ["3", "Revise or approve", "Use `/forge feedback ...` or `/forge approve`."],
                ["4", "Review the PR", "Branch, checks, risks, and PR link stay in GitHub."],
              ].map(([num, title, body], index, list) => (
                <div key={title} className="grid grid-cols-[2rem_1fr] gap-4">
                  <div className="flex flex-col items-center">
                    <span className="flex h-8 w-8 items-center justify-center rounded-full bg-white text-xs font-bold text-black">
                      {num}
                    </span>
                    {index < list.length - 1 && <span className="h-10 w-px bg-white/10" />}
                  </div>
                  <div className="pb-5">
                    <h2 className="text-sm font-semibold text-white">{title}</h2>
                    <p className="mt-1 text-sm leading-relaxed text-muted">{body}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
