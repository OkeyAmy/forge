use std::process::Stdio;

use forge_run::run_single::build_model;
use forge_types::public_workflow::{
    render_branch_ready_comment, render_plan_comment, ForgeCheckResult, ForgeCommand,
    ForgeGitHubEvent, ForgeJobState, ForgePlan, ForgeVerificationSummary, GitHubSubject,
};
use forge_types::{ForgeError, HistoryItem, MessageContent, Role};
use tokio::sync::mpsc;

use crate::github_app_auth::{GitHubAppClient, GitHubAppConfig, GitHubPullRequestFile};
use crate::job_store::FileJobStore;

#[derive(Clone)]
pub struct AppState {
    pub jobs: FileJobStore,
    pub queue: mpsc::Sender<String>,
}

pub fn start_worker(jobs: FileJobStore) -> mpsc::Sender<String> {
    let (tx, mut rx) = mpsc::channel::<String>(128);
    tokio::spawn(async move {
        while let Some(job_id) = rx.recv().await {
            if let Err(error) = process_job(&jobs, &job_id).await {
                tracing::error!(%job_id, %error, "Forge workflow job failed");
                let _ = jobs
                    .update(&job_id, |job| {
                        job.state = ForgeJobState::Failed;
                        job.error = Some(error.to_string());
                    })
                    .await;
            }
        }
    });
    tx
}

pub async fn process_job(jobs: &FileJobStore, job_id: &str) -> Result<(), ForgeError> {
    let job = jobs
        .get(job_id)
        .await?
        .ok_or_else(|| ForgeError::Config(format!("job {job_id} was not found")))?;

    match job.event.command {
        ForgeCommand::Plan => process_plan_job(jobs, job_id, &job.event).await,
        ForgeCommand::Approve => process_approve_job(jobs, job_id, &job.event).await,
        ForgeCommand::Review => process_review_job(jobs, job_id, &job.event).await,
        ForgeCommand::Status
        | ForgeCommand::Cancel
        | ForgeCommand::Improve
        | ForgeCommand::Ask { .. }
        | ForgeCommand::Fix => {
            jobs.update(job_id, |job| {
                job.state = ForgeJobState::NeedsInput;
                job.error =
                    Some("this Forge command is accepted but not implemented yet".to_string());
            })
            .await?;
            Ok(())
        }
    }
}

async fn process_plan_job(
    jobs: &FileJobStore,
    job_id: &str,
    event: &ForgeGitHubEvent,
) -> Result<(), ForgeError> {
    jobs.update(job_id, |job| job.state = ForgeJobState::Planning)
        .await?;
    post_comment_if_configured(
        event,
        "## Forge\n\nI am inspecting this repository in an E2B sandbox so I can produce a codebase-aware plan.",
    )
    .await?;
    let plan = match run_e2b_plan_job(event).await {
        Ok(plan) => plan,
        Err(error) => {
            tracing::warn!(%error, "E2B planning failed");
            post_comment_if_configured(
                event,
                &format!(
                    "## Forge Planning Failed\n\nForge could not inspect this repository in E2B, so I did not create an approval plan.\n\n### Error\n`{}`\n\nRun `/forge plan` again after the deployment or configuration issue is fixed.",
                    markdown_inline_code(&error.to_string())
                ),
            )
            .await?;
            jobs.update(job_id, |job| {
                job.state = ForgeJobState::Failed;
                job.error = Some(error.to_string());
            })
            .await?;
            return Ok(());
        }
    };
    post_comment_if_configured(event, &render_plan_comment(&plan)).await?;
    jobs.update(job_id, |job| {
        job.state = ForgeJobState::WaitingForApproval;
        job.branch_name = Some(plan.branch_name.clone());
        job.plan = Some(plan);
        job.error = None;
    })
    .await?;
    Ok(())
}

async fn process_approve_job(
    jobs: &FileJobStore,
    job_id: &str,
    event: &ForgeGitHubEvent,
) -> Result<(), ForgeError> {
    let GitHubSubject::Issue { number, .. } = &event.subject else {
        return Err(ForgeError::Config(
            "/forge approve requires an issue context".to_string(),
        ));
    };
    let Some(plan_job) = jobs
        .latest_waiting_issue_plan(&event.repository.full_name, *number)
        .await?
    else {
        let body =
            "## Forge\n\nI could not find a waiting plan for this issue. Run `/forge plan` first.";
        post_comment_if_configured(event, body).await?;
        jobs.update(job_id, |job| {
            job.state = ForgeJobState::NeedsInput;
            job.error = Some("no waiting issue plan found".to_string());
        })
        .await?;
        return Ok(());
    };

    let branch = plan_job
        .branch_name
        .clone()
        .unwrap_or_else(|| issue_branch_name(*number));
    jobs.update(&plan_job.id, |job| {
        job.state = ForgeJobState::Approved;
        job.error = None;
    })
    .await?;
    jobs.update(job_id, |job| {
        job.state = ForgeJobState::Running;
        job.branch_name = Some(branch.clone());
        job.plan = plan_job.plan.clone();
    })
    .await?;

    let output = run_e2b_job(event, &branch, plan_job.plan.as_ref()).await?;
    let pull_request = create_pull_request_for_output(event, &output).await?;
    let comment = render_branch_ready_comment(
        &output.branch_name,
        &output.compare_url,
        pull_request.as_ref().map(|pr| pr.html_url.as_str()),
        &output.changed_files,
        &ForgeVerificationSummary {
            commands: output
                .checks
                .iter()
                .map(|check| ForgeCheckResult {
                    command: check.command.clone(),
                    exit_code: check.exit_code,
                    passed: check.passed,
                })
                .collect(),
            risks: output.risks.clone(),
        },
    );
    post_comment_if_configured(event, &comment).await?;
    jobs.update(job_id, |job| {
        job.state = ForgeJobState::BranchPushed;
        job.pull_request_url = pull_request.as_ref().map(|pr| pr.html_url.clone());
        job.pull_request_number = pull_request.as_ref().map(|pr| pr.number);
    })
    .await?;
    Ok(())
}

async fn process_review_job(
    jobs: &FileJobStore,
    job_id: &str,
    event: &ForgeGitHubEvent,
) -> Result<(), ForgeError> {
    jobs.update(job_id, |job| job.state = ForgeJobState::Running)
        .await?;
    let files = fetch_pr_files_if_configured(event).await?;
    let body = build_model_pr_review_comment(event, &files)
        .await
        .unwrap_or_else(|_| {
            render_native_review_comment(event, &files).unwrap_or_else(|error| {
                format!("## Forge PR Review\n\nForge could not review this pull request: {error}")
            })
        });
    post_comment_if_configured(event, &body).await?;
    jobs.update(job_id, |job| job.state = ForgeJobState::BranchPushed)
        .await?;
    Ok(())
}

#[cfg(test)]
pub fn build_issue_plan(event: &ForgeGitHubEvent) -> Result<ForgePlan, ForgeError> {
    let GitHubSubject::Issue {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "cannot build an issue plan from a pull request event".to_string(),
        ));
    };
    let issue_body = body
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or("No issue body was provided.");

    Ok(ForgePlan {
        summary: format!("Issue #{number}: {title}\n\n{issue_body}"),
        proposed_change: format!(
            "Create `{}` from `{}` and make the smallest code change that satisfies the issue.",
            issue_branch_name(*number),
            event.repository.default_branch
        ),
        checks: configured_checks(),
        risk: "Medium: Forge will execute only after approval in an E2B sandbox, then report verification output before a PR is opened.".to_string(),
        branch_name: issue_branch_name(*number),
        codebase_context: None,
    })
}

pub fn render_native_review_comment(
    event: &ForgeGitHubEvent,
    files: &[GitHubPullRequestFile],
) -> Result<String, ForgeError> {
    let GitHubSubject::PullRequest {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "cannot build a PR review from an issue event".to_string(),
        ));
    };
    let body = body
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or("No pull request body was provided.");
    let changed_files = if files.is_empty() {
        "- Changed file data was not available in this environment.".to_string()
    } else {
        files
            .iter()
            .map(|file| {
                format!(
                    "- `{}`: {} (+{}, -{})",
                    file.filename, file.status, file.additions, file.deletions
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let patch_risks = files
        .iter()
        .filter(|file| {
            file.patch
                .as_ref()
                .is_some_and(|patch| patch.len() > 12_000)
        })
        .map(|file| {
            format!(
                "- `{}` has a large patch; review it manually for missed context.",
                file.filename
            )
        })
        .collect::<Vec<_>>();
    let risk_section = if patch_risks.is_empty() {
        "- No file-level risk flags from the fetched PR metadata.".to_string()
    } else {
        patch_risks.join("\n")
    };

    Ok(format!(
        "## Forge PR Review\n\n\
         ### Scope\nPR #{number}: {title}\n\n{body}\n\n\
         ### Changed Files\n{changed_files}\n\n\
         ### Risk Flags\n{risk_section}\n\n\
         ### Test Focus\n- Confirm the PR includes targeted tests for the changed behavior.\n- Confirm the default branch still builds after merge."
    ))
}

pub fn build_pr_review_prompt(
    event: &ForgeGitHubEvent,
    files: &[GitHubPullRequestFile],
) -> Result<String, ForgeError> {
    let GitHubSubject::PullRequest {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "cannot build a PR review prompt from an issue event".to_string(),
        ));
    };
    let file_context = files
        .iter()
        .map(|file| {
            format!(
                "File: {}\nStatus: {}\nAdditions: {}\nDeletions: {}\nPatch:\n{}\n",
                file.filename,
                file.status,
                file.additions,
                file.deletions,
                file.patch.as_deref().unwrap_or("[patch unavailable]")
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    Ok(format!(
        "Review this GitHub pull request as Forge.\n\n\
         Repository: {}\n\
         PR: #{number} {title}\n\
         PR body:\n{}\n\n\
         Changed files:\n{file_context}\n\n\
         Return markdown with these sections exactly:\n\
         ## Forge PR Review\n\
         ### Summary\n\
         ### Key Issues To Review\n\
         ### Suggested Fixes\n\
         ### Test Gaps\n\
         ### Merge Risk\n\n\
         Be specific. Do not invent files or behavior that are not in the diff.",
        event.repository.full_name,
        body.as_deref()
            .unwrap_or("No pull request body was provided.")
    ))
}

async fn build_model_pr_review_comment(
    event: &ForgeGitHubEvent,
    files: &[GitHubPullRequestFile],
) -> Result<String, ForgeError> {
    let model = build_model(None, None, None)?;
    let prompt = build_pr_review_prompt(event, files)?;
    let history = vec![
        HistoryItem {
            role: Role::System,
            content: MessageContent::Text(
                "You are Forge, a GitHub App that reviews pull requests from repository diffs."
                    .to_string(),
            ),
            ..Default::default()
        },
        HistoryItem {
            role: Role::User,
            content: MessageContent::Text(prompt),
            ..Default::default()
        },
    ];
    let output = model.query(&history).await?;
    Ok(output.message)
}

async fn fetch_pr_files_if_configured(
    event: &ForgeGitHubEvent,
) -> Result<Vec<GitHubPullRequestFile>, ForgeError> {
    let Some(installation_id) = event.installation_id else {
        return Ok(Vec::new());
    };
    let GitHubSubject::PullRequest { number, .. } = &event.subject else {
        return Ok(Vec::new());
    };
    let Ok(config) = GitHubAppConfig::from_env() else {
        return Ok(Vec::new());
    };
    GitHubAppClient::new(config)
        .list_pull_request_files(
            installation_id,
            &event.repository.owner,
            &event.repository.name,
            *number,
        )
        .await
}

async fn create_pull_request_for_output(
    event: &ForgeGitHubEvent,
    output: &E2bRunnerOutput,
) -> Result<Option<crate::github_app_auth::GitHubPullRequestResponse>, ForgeError> {
    if output.changed_files.is_empty() {
        return Ok(None);
    }
    let Some(installation_id) = event.installation_id else {
        return Ok(None);
    };
    let GitHubSubject::Issue { number, title, .. } = &event.subject else {
        return Ok(None);
    };
    let Ok(config) = GitHubAppConfig::from_env() else {
        return Ok(None);
    };
    let body = format!(
        "Forge implemented issue #{number} after maintainer approval.\n\n\
         ## Verification\n{}\n\n\
         ## Risks\n{}\n\n\
         Closes #{number}.",
        output
            .checks
            .iter()
            .map(|check| {
                let status = if check.passed { "passed" } else { "failed" };
                format!("- `{}`: {} (exit {})", check.command, status, check.exit_code)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        if output.risks.is_empty() {
            "- No known remaining risks.".to_string()
        } else {
            output
                .risks
                .iter()
                .map(|risk| format!("- {risk}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );
    let pr = GitHubAppClient::new(config)
        .create_pull_request(
            installation_id,
            &event.repository.owner,
            &event.repository.name,
            &format!("Forge: {title}"),
            &output.branch_name,
            &event.repository.default_branch,
            &body,
        )
        .await?;
    Ok(Some(pr))
}

fn configured_checks() -> Vec<String> {
    std::env::var("FORGE_PUBLIC_CHECKS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_else(|| vec!["cargo test --workspace".to_string()])
}

fn issue_branch_name(issue_number: u64) -> String {
    format!("forge/issue-{issue_number}")
}

fn markdown_inline_code(value: &str) -> String {
    value.replace('`', "'")
}

async fn post_comment_if_configured(
    event: &ForgeGitHubEvent,
    body: &str,
) -> Result<Option<u64>, ForgeError> {
    let Some(installation_id) = event.installation_id else {
        return Ok(None);
    };
    let Ok(config) = GitHubAppConfig::from_env() else {
        return Ok(None);
    };
    let client = GitHubAppClient::new(config);
    let number = match &event.subject {
        GitHubSubject::Issue { number, .. } | GitHubSubject::PullRequest { number, .. } => *number,
    };
    let comment = client
        .post_issue_comment(
            installation_id,
            &event.repository.owner,
            &event.repository.name,
            number,
            body,
        )
        .await?;
    Ok(Some(comment.id))
}

#[derive(Debug, serde::Serialize)]
struct E2bRunnerInput {
    mode: String,
    repository: E2bRepositoryInput,
    issue: E2bIssueInput,
    branch_name: String,
    installation_token: String,
    work_command: Option<String>,
    model: Option<E2bModelInput>,
    max_steps: u32,
    checks: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct E2bRepositoryInput {
    owner: String,
    name: String,
    clone_url: String,
    default_branch: String,
}

#[derive(Debug, serde::Serialize)]
struct E2bIssueInput {
    number: u64,
    title: String,
    body: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct E2bModelInput {
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct E2bRunnerOutput {
    pub branch_name: String,
    pub compare_url: String,
    pub changed_files: Vec<String>,
    pub checks: Vec<E2bCheckOutput>,
    pub risks: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct E2bPlanRunnerOutput {
    pub mode: String,
    pub repository: String,
    pub branch: String,
    pub exploration: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
pub struct E2bCheckOutput {
    pub command: String,
    pub exit_code: i32,
    pub passed: bool,
}

async fn run_e2b_plan_job(event: &ForgeGitHubEvent) -> Result<ForgePlan, ForgeError> {
    let Some(installation_id) = event.installation_id else {
        return Err(ForgeError::Config(
            "GitHub installation id is required for codebase inspection".to_string(),
        ));
    };
    let GitHubSubject::Issue {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "E2B planning requires an issue event".to_string(),
        ));
    };
    let branch_name = issue_branch_name(*number);
    let token = GitHubAppClient::new(GitHubAppConfig::from_env()?)
        .installation_token(installation_id)
        .await?;
    let input = E2bRunnerInput {
        mode: "explore".to_string(),
        repository: E2bRepositoryInput {
            owner: event.repository.owner.clone(),
            name: event.repository.name.clone(),
            clone_url: event.repository.clone_url.clone(),
            default_branch: event.repository.default_branch.clone(),
        },
        issue: E2bIssueInput {
            number: *number,
            title: title.clone(),
            body: body.clone(),
        },
        branch_name: branch_name.clone(),
        installation_token: token,
        work_command: None,
        model: Some(e2b_model_from_env().ok_or_else(|| {
            ForgeError::Config(
                "FORGE_MODEL, FORGE_BASE_URL, and FORGE_API_KEY are required for E2B planning"
                    .to_string(),
            )
        })?),
        max_steps: std::env::var("FORGE_E2B_MAX_STEPS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(6),
        checks: configured_checks(),
    };
    let stdout =
        run_e2b_runner_with_json(&serde_json::to_vec(&input).map_err(ForgeError::Json)?).await?;
    let inspection = parse_e2b_plan_runner_output(&stdout)?;
    build_issue_plan_from_inspection(event, &inspection)
}

fn build_issue_plan_from_inspection(
    event: &ForgeGitHubEvent,
    inspection: &E2bPlanRunnerOutput,
) -> Result<ForgePlan, ForgeError> {
    if inspection.mode != "exploration" {
        return Err(ForgeError::Environment(format!(
            "E2B planning returned unexpected mode `{}`",
            inspection.mode
        )));
    }
    let GitHubSubject::Issue {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "cannot build an issue plan from a pull request event".to_string(),
        ));
    };
    let summary = inspection
        .exploration
        .get("synthesized_summary")
        .map(|summary| {
            if let Some(error) = summary.get("error").and_then(|value| value.as_str()) {
                return Err(ForgeError::Environment(format!(
                    "E2B model synthesis failed: {error}"
                )));
            }
            serde_json::to_string_pretty(summary).map_err(ForgeError::Json)
        })
        .transpose()?
        .ok_or_else(|| {
            ForgeError::Environment(
                "E2B inspection did not return a model-synthesized codebase summary".to_string(),
            )
        })?;
    let issue_body = body
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or("No issue body was provided.");

    Ok(ForgePlan {
        summary: format!("Issue #{number}: {title}\n\n{issue_body}"),
        proposed_change: format!(
            "Use the inspected `{}` repository context to make the smallest code change that satisfies this issue, then run the checks listed below.",
            inspection.repository
        ),
        checks: configured_checks(),
        risk: format!(
            "Medium: Forge inspected `{}` at `{}` in E2B. Implementation still waits for maintainer approval and will run in a fresh E2B sandbox.",
            inspection.repository, inspection.branch
        ),
        branch_name: issue_branch_name(*number),
        codebase_context: Some(summary),
    })
}

async fn run_e2b_job(
    event: &ForgeGitHubEvent,
    branch_name: &str,
    plan: Option<&ForgePlan>,
) -> Result<E2bRunnerOutput, ForgeError> {
    let Some(installation_id) = event.installation_id else {
        return Err(ForgeError::Config(
            "GitHub installation id is required for branch push".to_string(),
        ));
    };
    let config = GitHubAppConfig::from_env()?;
    let token = GitHubAppClient::new(config)
        .installation_token(installation_id)
        .await?;
    let GitHubSubject::Issue {
        number,
        title,
        body,
        ..
    } = &event.subject
    else {
        return Err(ForgeError::Config(
            "E2B execution requires an issue event".to_string(),
        ));
    };

    let input = E2bRunnerInput {
        mode: "implement".to_string(),
        repository: E2bRepositoryInput {
            owner: event.repository.owner.clone(),
            name: event.repository.name.clone(),
            clone_url: event.repository.clone_url.clone(),
            default_branch: event.repository.default_branch.clone(),
        },
        issue: E2bIssueInput {
            number: *number,
            title: title.clone(),
            body: body.clone(),
        },
        branch_name: branch_name.to_string(),
        installation_token: token,
        work_command: std::env::var("FORGE_E2B_WORK_COMMAND")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        model: e2b_model_from_env(),
        max_steps: std::env::var("FORGE_E2B_MAX_STEPS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(6),
        checks: plan
            .map(|plan| plan.checks.clone())
            .unwrap_or_else(configured_checks),
    };

    let stdout =
        run_e2b_runner_with_json(&serde_json::to_vec(&input).map_err(ForgeError::Json)?).await?;
    parse_e2b_runner_output(&stdout)
}

async fn run_e2b_runner_with_json(input_json: &[u8]) -> Result<String, ForgeError> {
    let runner = std::env::var("FORGE_E2B_RUNNER")
        .unwrap_or_else(|_| "scripts/e2b-runner/run-job.mjs".to_string());
    let mut child = tokio::process::Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ForgeError::Io)?;
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(input_json).await.map_err(ForgeError::Io)?;
    }
    let output = child.wait_with_output().await.map_err(ForgeError::Io)?;
    if !output.status.success() {
        return Err(ForgeError::Environment(format!(
            "E2B runner failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| ForgeError::Environment(format!("E2B runner emitted invalid UTF-8: {e}")))
}

fn e2b_model_from_env() -> Option<E2bModelInput> {
    let base_url = std::env::var("FORGE_BASE_URL").ok()?;
    let api_key = std::env::var("FORGE_API_KEY").ok()?;
    let model = std::env::var("FORGE_MODEL").ok()?;
    if base_url.trim().is_empty() || api_key.trim().is_empty() || model.trim().is_empty() {
        return None;
    }
    Some(E2bModelInput {
        base_url,
        api_key,
        model,
    })
}

pub fn parse_e2b_runner_output(stdout: &str) -> Result<E2bRunnerOutput, ForgeError> {
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| ForgeError::Environment("E2B runner emitted no output".to_string()))?;
    serde_json::from_str(json_line).map_err(ForgeError::Json)
}

pub fn parse_e2b_plan_runner_output(stdout: &str) -> Result<E2bPlanRunnerOutput, ForgeError> {
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| ForgeError::Environment("E2B runner emitted no output".to_string()))?;
    serde_json::from_str(json_line).map_err(ForgeError::Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_types::public_workflow::{GitHubActor, GitHubRepositoryRef, GitHubSubject};

    fn issue_event(command: ForgeCommand) -> ForgeGitHubEvent {
        ForgeGitHubEvent {
            delivery_id: "delivery-1".to_string(),
            installation_id: None,
            repository: GitHubRepositoryRef {
                owner: "acme".to_string(),
                name: "app".to_string(),
                full_name: "acme/app".to_string(),
                default_branch: "main".to_string(),
                clone_url: "https://github.com/acme/app.git".to_string(),
            },
            actor: GitHubActor {
                login: "maintainer".to_string(),
            },
            subject: GitHubSubject::Issue {
                number: 9,
                title: "Fix login redirect".to_string(),
                body: Some("Users land on / after OAuth.".to_string()),
                html_url: "https://github.com/acme/app/issues/9".to_string(),
            },
            command,
        }
    }

    fn pr_event(command: ForgeCommand) -> ForgeGitHubEvent {
        let mut event = issue_event(command);
        event.subject = GitHubSubject::PullRequest {
            number: 4,
            title: "Change auth callback".to_string(),
            body: Some("Updates redirect handling.".to_string()),
            html_url: "https://github.com/acme/app/pull/4".to_string(),
        };
        event
    }

    #[test]
    fn issue_plan_uses_issue_context_and_default_branch() {
        let plan = build_issue_plan(&issue_event(ForgeCommand::Plan)).unwrap();

        assert!(plan.summary.contains("Fix login redirect"));
        assert!(plan.summary.contains("Users land on / after OAuth."));
        assert!(plan.proposed_change.contains("main"));
        assert_eq!(plan.branch_name, "forge/issue-9");
    }

    #[test]
    fn codebase_inspection_builds_approval_plan() {
        let inspection = E2bPlanRunnerOutput {
            mode: "exploration".to_string(),
            repository: "acme/app".to_string(),
            branch: "main".to_string(),
            exploration: serde_json::json!({
                "synthesized_summary": {
                    "language": "Rust",
                    "framework": "Axum",
                    "test_setup": "cargo test --workspace"
                }
            }),
        };

        let plan = build_issue_plan_from_inspection(&issue_event(ForgeCommand::Plan), &inspection)
            .unwrap();

        assert!(plan.codebase_context.unwrap().contains("Axum"));
        assert!(plan.proposed_change.contains("acme/app"));
        assert!(plan.risk.contains("main"));
        assert_eq!(plan.branch_name, "forge/issue-9");
    }

    #[test]
    fn codebase_inspection_rejects_model_synthesis_errors() {
        let inspection = E2bPlanRunnerOutput {
            mode: "exploration".to_string(),
            repository: "acme/app".to_string(),
            branch: "main".to_string(),
            exploration: serde_json::json!({
                "synthesized_summary": {
                    "error": "model request failed 400: API key expired"
                }
            }),
        };

        let error =
            build_issue_plan_from_inspection(&issue_event(ForgeCommand::Plan), &inspection)
                .unwrap_err()
                .to_string();

        assert!(error.contains("API key expired"));
    }

    #[test]
    fn parse_e2b_plan_output_uses_last_json_line() {
        let output = parse_e2b_plan_runner_output(
            "creating sandbox\n{\"mode\":\"exploration\",\"repository\":\"acme/app\",\"branch\":\"main\",\"exploration\":{\"synthesized_summary\":{\"language\":\"Rust\"}}}\n",
        )
        .unwrap();

        assert_eq!(output.mode, "exploration");
        assert_eq!(output.repository, "acme/app");
        assert_eq!(output.branch, "main");
        assert_eq!(
            output.exploration["synthesized_summary"]["language"],
            "Rust"
        );
    }

    #[test]
    fn native_review_rejects_issue_events() {
        let result = render_native_review_comment(&issue_event(ForgeCommand::Review), &[]);

        assert!(result.is_err());
    }

    #[test]
    fn native_review_includes_pull_request_context() {
        let review = render_native_review_comment(
            &pr_event(ForgeCommand::Review),
            &[GitHubPullRequestFile {
                filename: "src/auth.rs".to_string(),
                status: "modified".to_string(),
                additions: 8,
                deletions: 2,
                patch: Some("@@".to_string()),
            }],
        )
        .unwrap();

        assert!(review.contains("PR #4"));
        assert!(review.contains("Change auth callback"));
        assert!(review.contains("Updates redirect handling."));
        assert!(review.contains("src/auth.rs"));
        assert!(review.contains("+8, -2"));
    }

    #[test]
    fn pr_review_prompt_contains_diff_context() {
        let prompt = build_pr_review_prompt(
            &pr_event(ForgeCommand::Review),
            &[GitHubPullRequestFile {
                filename: "src/auth.rs".to_string(),
                status: "modified".to_string(),
                additions: 8,
                deletions: 2,
                patch: Some("@@ -1 +1 @@\n-old\n+new".to_string()),
            }],
        )
        .unwrap();

        assert!(prompt.contains("PR: #4 Change auth callback"));
        assert!(prompt.contains("File: src/auth.rs"));
        assert!(prompt.contains("@@ -1 +1 @@"));
        assert!(prompt.contains("Do not invent files"));
    }

    #[test]
    fn e2b_runner_output_parses_last_json_line() {
        let output = parse_e2b_runner_output(
            "log line\n{\"branch_name\":\"forge/issue-9\",\"compare_url\":\"https://github.com/acme/app/compare/main...forge/issue-9\",\"changed_files\":[\"src/lib.rs\"],\"checks\":[{\"command\":\"cargo test\",\"exit_code\":0,\"passed\":true}],\"risks\":[\"manual PR open required\"]}\n",
        )
        .unwrap();

        assert_eq!(output.branch_name, "forge/issue-9");
        assert_eq!(output.changed_files, vec!["src/lib.rs"]);
        assert_eq!(output.checks[0].command, "cargo test");
        assert!(output.checks[0].passed);
    }

    #[tokio::test]
    async fn approve_without_waiting_plan_records_needs_input() {
        let path = std::env::temp_dir().join(format!(
            "forge-workflow-approve-{}-{}.json",
            std::process::id(),
            issue_branch_name(9)
        ));
        let store = FileJobStore::new(&path);
        let job = store
            .insert_event(issue_event(ForgeCommand::Approve))
            .await
            .unwrap();

        process_job(&store, &job.id).await.unwrap();
        let updated = store.get(&job.id).await.unwrap().unwrap();

        assert_eq!(updated.state, ForgeJobState::NeedsInput);
        assert_eq!(
            updated.error.as_deref(),
            Some("no waiting issue plan found")
        );
        let _ = std::fs::remove_file(path);
    }
}
