use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeCommand {
    Plan,
    Approve,
    Status,
    Cancel,
    Review,
    Improve,
    Feedback { message: String },
    Ask { question: String },
    Fix,
}

impl ForgeCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let rest = input.trim().strip_prefix("/forge")?.trim();
        if rest.is_empty() {
            return Some(Self::Plan);
        }

        let mut parts = rest.splitn(2, char::is_whitespace);
        let command = parts.next()?.to_ascii_lowercase();
        let arg = parts.next().unwrap_or("").trim().to_string();

        match command.as_str() {
            "plan" => Some(Self::Plan),
            "approve" => Some(Self::Approve),
            "status" => Some(Self::Status),
            "cancel" => Some(Self::Cancel),
            "review" => Some(Self::Review),
            "improve" => Some(Self::Improve),
            "feedback" if !arg.is_empty() => Some(Self::Feedback { message: arg }),
            "change" if !arg.is_empty() => Some(Self::Feedback { message: arg }),
            "ask" if !arg.is_empty() => Some(Self::Ask { question: arg }),
            "fix" => Some(Self::Fix),
            _ => None,
        }
    }

    pub fn requires_issue_context(&self) -> bool {
        matches!(self, Self::Plan | Self::Approve | Self::Feedback { .. })
    }

    pub fn requires_pr_context(&self) -> bool {
        matches!(
            self,
            Self::Review | Self::Improve | Self::Ask { .. } | Self::Fix
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeJobState {
    Received,
    Planning,
    WaitingForApproval,
    Approved,
    Queued,
    Running,
    Verifying,
    BranchPushed,
    Failed,
    Cancelled,
    NeedsInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRepositoryRef {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub default_branch: String,
    pub clone_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubActor {
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitHubSubject {
    Issue {
        number: u64,
        title: String,
        body: Option<String>,
        html_url: String,
    },
    PullRequest {
        number: u64,
        title: String,
        body: Option<String>,
        html_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeGitHubEvent {
    pub delivery_id: String,
    pub installation_id: Option<u64>,
    pub repository: GitHubRepositoryRef,
    pub actor: GitHubActor,
    pub subject: GitHubSubject,
    pub command: ForgeCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeJob {
    pub id: String,
    pub state: ForgeJobState,
    pub event: ForgeGitHubEvent,
    pub created_at_unix_secs: u64,
    pub updated_at_unix_secs: u64,
    pub plan: Option<ForgePlan>,
    pub branch_name: Option<String>,
    #[serde(default)]
    pub pull_request_url: Option<String>,
    #[serde(default)]
    pub pull_request_number: Option<u64>,
    pub error: Option<String>,
}

impl ForgeJob {
    pub fn issue_number(&self) -> Option<u64> {
        match &self.event.subject {
            GitHubSubject::Issue { number, .. } => Some(*number),
            GitHubSubject::PullRequest { .. } => None,
        }
    }

    pub fn pull_request_number(&self) -> Option<u64> {
        match &self.event.subject {
            GitHubSubject::Issue { .. } => None,
            GitHubSubject::PullRequest { number, .. } => Some(*number),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgePlan {
    pub summary: String,
    pub proposed_change: String,
    pub checks: Vec<String>,
    pub risk: String,
    pub branch_name: String,
    pub codebase_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeVerificationSummary {
    pub commands: Vec<ForgeCheckResult>,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeCheckResult {
    pub command: String,
    pub exit_code: i32,
    pub passed: bool,
}

pub fn render_plan_comment(plan: &ForgePlan) -> String {
    let checks = if plan.checks.is_empty() {
        "- No automatic checks configured.".to_string()
    } else {
        plan.checks
            .iter()
            .map(|check| format!("- `{check}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let context_section = plan
        .codebase_context
        .as_ref()
        .map(|ctx| format!("### Codebase Context\n{ctx}\n\n"))
        .unwrap_or_default();

    format!(
        "## Forge Plan\n\n\
         {context_section}\
         ### What I Think This Issue Needs\n{summary}\n\n\
         ### Proposed Change\n{proposed_change}\n\n\
         ### Setup And Checks I Will Run\n{checks}\n\n\
         ### Risk\n{risk}\n\n\
         ### Branch\n`{branch_name}`\n\n\
         ### Approve\nComment `/forge approve` to let Forge make changes in an E2B sandbox.",
        summary = plan.summary,
        proposed_change = plan.proposed_change,
        checks = checks,
        risk = plan.risk,
        branch_name = plan.branch_name
    )
}

pub fn render_plan_comment_with_feedback(plan: &ForgePlan) -> String {
    let mut rendered = render_plan_comment(plan);
    rendered.push_str(
        "\n\n### Give Feedback\nComment `/forge feedback <what should change>` and Forge will revise the plan before implementation.",
    );
    rendered
}

pub fn render_approval_started_comment(branch_name: &str) -> String {
    format!(
        "## Forge Approved\n\n\
         I found the approved plan and am starting a fresh E2B sandbox now.\n\n\
         ### What Happens Next\n\
         - Clone the repository with the GitHub App installation token.\n\
         - Check out `{branch_name}` from the default branch.\n\
         - Inspect the code and run the implementation pipeline.\n\
         - Run the planned validation commands.\n\
         - Push the branch and open a pull request if changes are produced.\n\n\
         I will report back in this thread with the branch, PR, checks, and risks."
    )
}

pub fn render_approval_failed_comment(error: &str) -> String {
    format!(
        "## Forge Implementation Failed\n\n\
         I found the approved plan, but the E2B implementation job failed before a branch or pull request was ready.\n\n\
         ### Error\n`{}`\n\n\
         Forge stopped this run so the implementation problem can be fixed without repeating unsafe work.",
        error.replace('`', "'")
    )
}

pub fn render_feedback_missing_plan_comment() -> String {
    "## Forge\n\nI received the feedback, but I could not find a waiting plan for this issue. Run `/forge plan` first.".to_string()
}

pub fn render_feedback_started_comment(message: &str) -> String {
    format!(
        "## Forge Feedback Received\n\n\
         I will revise the plan using this maintainer feedback before any code changes are made:\n\n\
         > {}\n\n\
         I am starting a fresh E2B inspection pass now.",
        message.replace('\n', "\n> ")
    )
}

pub fn render_feedback_failed_comment(error: &str) -> String {
    format!(
        "## Forge Plan Revision Failed\n\n\
         I received the feedback, but could not revise the plan in E2B.\n\n\
         ### Error\n`{}`\n\n\
         Comment `/forge feedback <message>` again after the deployment or configuration issue is fixed.",
        error.replace('`', "'")
    )
}

pub fn render_branch_ready_comment(
    branch_name: &str,
    compare_url: &str,
    pull_request_url: Option<&str>,
    changed_files: &[String],
    verification: &ForgeVerificationSummary,
) -> String {
    let files = if changed_files.is_empty() {
        "- No changed files reported.".to_string()
    } else {
        changed_files
            .iter()
            .map(|file| format!("- `{file}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let checks = if verification.commands.is_empty() {
        "- No verification commands were run.".to_string()
    } else {
        verification
            .commands
            .iter()
            .map(|check| {
                let status = if check.passed { "passed" } else { "failed" };
                format!(
                    "- `{}`: {} (exit {})",
                    check.command, status, check.exit_code
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let risks = if verification.risks.is_empty() {
        "- No known remaining risks.".to_string()
    } else {
        verification
            .risks
            .iter()
            .map(|risk| format!("- {risk}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let next_step = pull_request_url
        .map(|url| format!("Forge opened a pull request: {url}"))
        .unwrap_or_else(|| {
            format!("Open a pull request from `{branch_name}` when the diff looks right.")
        });

    format!(
        "## Forge Branch Ready\n\n\
         ### Branch\n`{branch_name}`\n\n\
         Compare: {compare_url}\n\n\
         ### Changed Files\n{files}\n\n\
         ### Verification\n{checks}\n\n\
         ### Risks And Notes\n{risks}\n\n\
         ### Next Step\n{next_step}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_forge_command_as_plan() {
        assert_eq!(ForgeCommand::parse("/forge"), Some(ForgeCommand::Plan));
    }

    #[test]
    fn parse_issue_commands() {
        assert_eq!(ForgeCommand::parse("/forge plan"), Some(ForgeCommand::Plan));
        assert_eq!(
            ForgeCommand::parse("/forge approve"),
            Some(ForgeCommand::Approve)
        );
        assert_eq!(
            ForgeCommand::parse("/forge feedback use the existing README section"),
            Some(ForgeCommand::Feedback {
                message: "use the existing README section".to_string()
            })
        );
        assert_eq!(
            ForgeCommand::parse("/forge change run npm build instead"),
            Some(ForgeCommand::Feedback {
                message: "run npm build instead".to_string()
            })
        );
        assert_eq!(
            ForgeCommand::parse("/forge status"),
            Some(ForgeCommand::Status)
        );
        assert_eq!(
            ForgeCommand::parse("/forge cancel"),
            Some(ForgeCommand::Cancel)
        );
    }

    #[test]
    fn parse_pr_commands() {
        assert_eq!(
            ForgeCommand::parse("/forge review"),
            Some(ForgeCommand::Review)
        );
        assert_eq!(
            ForgeCommand::parse("/forge improve"),
            Some(ForgeCommand::Improve)
        );
        assert_eq!(ForgeCommand::parse("/forge fix"), Some(ForgeCommand::Fix));
        assert_eq!(
            ForgeCommand::parse("/forge ask why did this fail?"),
            Some(ForgeCommand::Ask {
                question: "why did this fail?".to_string()
            })
        );
    }

    #[test]
    fn parse_rejects_unknown_or_empty_ask() {
        assert_eq!(ForgeCommand::parse("forge plan"), None);
        assert_eq!(ForgeCommand::parse("/forge deploy"), None);
        assert_eq!(ForgeCommand::parse("/forge ask"), None);
        assert_eq!(ForgeCommand::parse("/forge feedback"), None);
    }

    #[test]
    fn command_context_flags_are_stable() {
        assert!(ForgeCommand::Plan.requires_issue_context());
        assert!(ForgeCommand::Feedback {
            message: "revise plan".to_string()
        }
        .requires_issue_context());
        assert!(ForgeCommand::Review.requires_pr_context());
        assert!(!ForgeCommand::Status.requires_issue_context());
        assert!(!ForgeCommand::Cancel.requires_pr_context());
    }

    #[test]
    fn render_plan_comment_contains_approval_command() {
        let plan = ForgePlan {
            summary: "The login form accepts blank emails.".to_string(),
            proposed_change: "Add validation before submit.".to_string(),
            checks: vec!["npm test".to_string()],
            risk: "low".to_string(),
            branch_name: "forge/issue-12-login-validation".to_string(),
            codebase_context: Some("React + TypeScript project with Vite.".to_string()),
        };

        let rendered = render_plan_comment(&plan);

        assert!(rendered.contains("## Forge Plan"));
        assert!(rendered.contains("/forge approve"));
        assert!(rendered.contains("forge/issue-12-login-validation"));
    }

    #[test]
    fn render_branch_comment_contains_checks_and_compare_url() {
        let verification = ForgeVerificationSummary {
            commands: vec![ForgeCheckResult {
                command: "cargo test --workspace".to_string(),
                exit_code: 0,
                passed: true,
            }],
            risks: vec!["No browser test was run.".to_string()],
        };

        let rendered = render_branch_ready_comment(
            "forge/issue-8-fix-api",
            "https://github.com/acme/app/compare/main...forge/issue-8-fix-api",
            Some("https://github.com/acme/app/pull/19"),
            &["src/api.rs".to_string()],
            &verification,
        );

        assert!(rendered.contains("## Forge Branch Ready"));
        assert!(rendered.contains("cargo test --workspace"));
        assert!(
            rendered.contains("https://github.com/acme/app/compare/main...forge/issue-8-fix-api")
        );
        assert!(rendered.contains("https://github.com/acme/app/pull/19"));
    }
}
