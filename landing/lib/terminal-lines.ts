export interface TerminalLine {
  type: "cmd" | "output" | "blank";
  text: string;
}

export const TERMINAL_LINES: TerminalLine[] = [
  // Step 1: discover issues
  { type: "cmd",    text: "docker compose run --rm list-issues --repo owner/repo" },
  { type: "output", text: "Open issues on owner/repo:" },
  { type: "blank",  text: "" },
  { type: "output", text: "#3   Add input validation to signup form  [bug]" },
  { type: "output", text: "#7   Dark mode flicker on page load       [enhancement]" },
  { type: "output", text: "#12  Upgrade to Node 20" },
  { type: "blank",  text: "" },
  // Step 2: fix a chosen issue
  { type: "cmd",    text: "docker compose run --rm forge run --repo owner/repo --issue 3" },
  { type: "output", text: "→ Fetching issue #3 from GitHub..." },
  { type: "output", text: "→ Starting Docker sandbox..." },
  { type: "output", text: "→ Cloning repository..." },
  { type: "blank",  text: "" },
  { type: "output", text: "Working autonomously..." },
  { type: "output", text: "→ Explored repo structure" },
  { type: "output", text: "→ Located signup form handler" },
  { type: "output", text: "→ Added email & password validation" },
  { type: "output", text: "→ Submitted patch" },
  { type: "blank",  text: "" },
  { type: "output", text: "Run complete. Exit status: submitted" },
  { type: "output", text: "diff --git a/src/utils/validation.ts b/src/utils/validation.ts" },
  { type: "output", text: "+export function isValidEmail(email: string): boolean {" },
  { type: "output", text: "+  const re = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/;" },
  { type: "output", text: "+  return re.test(email);" },
  { type: "output", text: "+}" },
];
