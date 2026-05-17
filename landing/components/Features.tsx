"use client";

import { Bot, GitBranch, GitPullRequestArrow, LockKeyhole, MessageSquareText, Workflow } from "lucide-react";
import { useInView } from "@/hooks/useInView";

const FEATURES = [
  {
    icon: Workflow,
    title: "GitHub App Workflow",
    description: "Users install Forge on repos and work entirely from GitHub issues and pull requests.",
  },
  {
    icon: LockKeyhole,
    title: "E2B Execution",
    description: "Repository inspection, implementation, and validation run in isolated E2B environments.",
  },
  {
    icon: MessageSquareText,
    title: "Approval-first Plans",
    description: "Forge posts a readable engineering plan and waits for `/forge approve` before editing.",
  },
  {
    icon: Bot,
    title: "Pipeline Skills",
    description: "Issue intake, inspection, planning, implementation, validation, review, and PR handoff are separate agent skills.",
  },
  {
    icon: GitBranch,
    title: "Repo-level SKILL.md",
    description: "Projects can teach Forge their setup, test commands, boundaries, and review rules.",
  },
  {
    icon: GitPullRequestArrow,
    title: "Pull Request Handoff",
    description: "Forge pushes `forge/issue-{N}`, opens a PR, and reports checks, changed files, and risks.",
  },
];

export function Features() {
  const { ref, inView } = useInView();

  return (
    <section id="features" className="py-24 px-4">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-3xl sm:text-4xl font-bold">
            What{" "}
            <span className="bg-gradient-to-r from-accent-blue to-accent-purple bg-clip-text text-transparent">
              Forge
            </span>{" "}
            does
          </h2>
          <p className="mt-4 text-muted max-w-xl mx-auto">
            The production system is built around GitHub, E2B, approval, and PR review.
          </p>
        </div>

        <div
          ref={ref}
          className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6"
        >
          {FEATURES.map((feature, i) => {
            const Icon = feature.icon;
            return (
              <div
                key={feature.title}
                style={{
                  opacity: inView ? 1 : 0,
                  transform: inView ? "translateY(0)" : "translateY(24px)",
                  transition: `opacity 0.5s ease ${i * 0.08}s, transform 0.5s ease ${i * 0.08}s`,
                }}
                className="p-6 rounded-lg border border-white/5 bg-white/[0.02] hover:border-accent-blue/30 hover:bg-white/[0.04] transition-all duration-300"
              >
                <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-accent-blue/20 to-accent-purple/20 flex items-center justify-center mb-4">
                  <Icon className="w-5 h-5 text-accent-blue" />
                </div>
                <h3 className="font-semibold text-white mb-2">{feature.title}</h3>
                <p className="text-sm text-muted leading-relaxed">{feature.description}</p>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
