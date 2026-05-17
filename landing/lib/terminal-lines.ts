export interface TerminalLine {
  type: "cmd" | "output" | "blank";
  text: string;
}

export const TERMINAL_LINES: TerminalLine[] = [
  { type: "cmd",    text: "github webhook: issues.labeled forge" },
  { type: "output", text: "→ Received issue #42 from acme/web" },
  { type: "output", text: "→ Starting E2B inspection sandbox" },
  { type: "output", text: "→ Cloning repository with GitHub App token" },
  { type: "blank",  text: "" },
  { type: "output", text: "[inspection] reading package.json, README.md, SKILL.md" },
  { type: "output", text: "[inspection] detected Vite + React + pnpm" },
  { type: "blank",  text: "" },
  { type: "output", text: "[planning] posting plan to GitHub issue thread" },
  { type: "output", text: "waiting_for_approval: /forge feedback ... or /forge approve" },
  { type: "cmd",    text: "/forge feedback keep the change limited to README.md" },
  { type: "output", text: "replanning: maintainer feedback received" },
  { type: "blank",  text: "" },
  { type: "cmd",    text: "/forge approve" },
  { type: "output", text: "→ Starting implementation sandbox" },
  { type: "output", text: "```bash" },
  { type: "output", text: "pnpm lint && pnpm build" },
  { type: "output", text: "```" },
  { type: "output", text: "checks passed" },
  { type: "blank",  text: "" },
  { type: "output", text: "Pushing branch and opening pull request..." },
  { type: "output", text: "Branch pushed: forge/issue-42" },
  { type: "output", text: "PR opened: acme/web#108" },
];
