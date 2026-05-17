"use client";

import { useInView } from "@/hooks/useInView";
import { GITHUB_APP_INSTALL_URL } from "@/lib/links";
import { CheckCircle2, Copy, Github, GitPullRequestArrow, Settings2, Tag } from "lucide-react";

const INSTALL_STEPS = [
  {
    icon: Github,
    title: "Install the GitHub App",
    description:
      "Choose your account or organization, then select the repositories Forge is allowed to work on.",
  },
  {
    icon: Tag,
    title: "Tag an issue for Forge",
    description:
      "Add the `forge` label or comment `/forge plan`. Forge inspects the repo in E2B and replies with a plan.",
  },
  {
    icon: CheckCircle2,
    title: "Approve before changes",
    description:
      "Comment `/forge feedback ...` to revise the plan, or `/forge approve` when it looks right. Forge then edits, validates, pushes a branch, and opens a PR.",
  },
  {
    icon: GitPullRequestArrow,
    title: "Review and merge",
    description:
      "The PR includes changed files, validation output, and risk notes so maintainers stay in control.",
  },
];

const COMMANDS = [
  { label: "Plan from an issue", value: "/forge plan" },
  { label: "Revise the plan", value: "/forge feedback use X instead" },
  { label: "Approve implementation", value: "/forge approve" },
  { label: "Review a pull request", value: "/forge review" },
];

export function Setup() {
  const { ref, inView } = useInView();

  return (
    <section id="setup" className="py-24 px-4">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-12 lg:grid-cols-[0.9fr_1.1fr] lg:items-start">
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-xs font-mono text-muted">
              <Settings2 className="h-3.5 w-3.5 text-accent-blue" />
              Production path
            </div>
            <h2 className="mt-5 text-3xl font-bold sm:text-4xl">
              Connect once. Work from GitHub issues.
            </h2>
            <p className="mt-4 text-muted leading-relaxed">
              Users do not need to clone Forge, configure E2B, or run a terminal.
              Your production deployment handles the agent runtime. They only
              install the GitHub App and use issue comments.
            </p>

            <a
              href={GITHUB_APP_INSTALL_URL}
              className="mt-8 inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-white px-6 text-sm font-semibold text-black transition hover:bg-white/90"
            >
              <Github className="h-4 w-4" />
              Add Forge to a repository
            </a>

            <div className="mt-8 rounded-lg border border-white/10 bg-white/[0.025] p-4">
              <p className="text-sm font-semibold text-white">Repo-level guidance</p>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                Add <code className="rounded bg-code-bg px-1.5 py-0.5 font-mono text-accent-blue">SKILL.md</code>,{" "}
                <code className="rounded bg-code-bg px-1.5 py-0.5 font-mono text-accent-blue">.forge/SKILL.md</code>, or{" "}
                <code className="rounded bg-code-bg px-1.5 py-0.5 font-mono text-accent-blue">.github/forge/SKILL.md</code>{" "}
                when a repo needs custom setup, validation, or review rules.
              </p>
            </div>
          </div>

          <div ref={ref} className="grid gap-4">
            {INSTALL_STEPS.map((step, index) => {
              const Icon = step.icon;
              return (
                <div
                  key={step.title}
                  style={{
                    opacity: inView ? 1 : 0,
                    transform: inView ? "translateY(0)" : "translateY(18px)",
                    transition: `opacity 0.45s ease ${index * 0.07}s, transform 0.45s ease ${index * 0.07}s`,
                  }}
                  className="grid grid-cols-[2.75rem_1fr] gap-4 rounded-lg border border-white/8 bg-white/[0.025] p-4"
                >
                  <div className="flex h-11 w-11 items-center justify-center rounded-lg bg-white text-black">
                    <Icon className="h-5 w-5" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-white">{step.title}</h3>
                    <p className="mt-1 text-sm leading-relaxed text-muted">{step.description}</p>
                  </div>
                </div>
              );
            })}

            <div className="rounded-lg border border-white/8 bg-[#111318] p-4">
              <p className="mb-3 text-sm font-semibold text-white">Commands users need</p>
              <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
                {COMMANDS.map((command) => (
                  <div key={command.value} className="rounded-md border border-white/8 bg-black/24 p-3">
                    <div className="flex items-center gap-2 text-xs text-muted">
                      <Copy className="h-3.5 w-3.5" />
                      {command.label}
                    </div>
                    <code className="mt-2 block font-mono text-sm text-accent-blue">
                      {command.value}
                    </code>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
