use axum::body::Bytes;
use axum::http::StatusCode;
use axum::Json;
use forge_run::config::{
    AgentConfigSerde, EnvConfigSerde, ProblemStatementConfigSerde, RunConfig,
};
use forge_run::run_batch::RunBatch;
use forge_run::run_single::RunSingle;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Shared request/response types
// ---------------------------------------------------------------------------

/// Full agent configuration — every field mirrors AgentConfigSerde.
#[derive(Debug, Default, Deserialize)]
pub struct AgentOptions {
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    /// Parser type: "xml" | "function_calling" | ...
    pub parser_type: Option<String>,
    /// Max agent steps before giving up (default: 100)
    pub max_steps: Option<u32>,
    /// Max times the agent may retry a malformed reply
    pub max_requeries: Option<u32>,
    /// Override the system prompt template
    pub system_template: Option<String>,
    /// Override the per-instance prompt template
    pub instance_template: Option<String>,
}

/// Full sandbox environment configuration — every field mirrors EnvConfigSerde.
#[derive(Debug, Default, Deserialize)]
pub struct EnvOptions {
    /// Docker image for the sandbox (default: "forge-sandbox:latest")
    pub image: Option<String>,
    /// Custom container name
    pub container_name: Option<String>,
    /// Path inside the container where the repo is cloned
    pub repo_path: Option<String>,
    /// Command timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Commands run once after the container starts
    pub startup_commands: Option<Vec<String>>,
    /// Extra environment variables injected into the container as (KEY, VALUE) pairs
    pub env_vars: Option<Vec<(String, String)>>,
    /// Base commit SHA to reset to before running the agent
    pub base_commit: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub exit_status: Option<String>,
    pub has_submission: bool,
    /// First 500 chars of the patch (full patch is in the trajectory file)
    pub submission_preview: Option<String>,
    /// Total agent steps taken
    pub steps: usize,
    /// Token / cost stats returned by the model
    pub model_stats: serde_json::Value,
    /// Path of the trajectory file written to disk
    pub trajectory_file: Option<String>,
}

// ---------------------------------------------------------------------------
// POST /api/run
// ---------------------------------------------------------------------------

/// Request body for POST /api/run.
///
/// Problem source — supply exactly one of:
///   - `github_url`
///   - `repo` + `issue`
///   - `problem_text`
///
/// All agent / env fields are optional; unset values fall back to env vars.
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    // ── problem source ──────────────────────────────────────────────────
    /// Full GitHub issue URL: "https://github.com/owner/repo/issues/42"
    pub github_url: Option<String>,
    /// owner/repo — pair with `issue`
    pub repo: Option<String>,
    /// Issue number — pair with `repo`
    pub issue: Option<u64>,
    /// Plain-text problem statement
    pub problem_text: Option<String>,

    // ── output ──────────────────────────────────────────────────────────
    /// Directory for trajectory output (default: "trajectories")
    pub output_dir: Option<String>,

    // ── full config overrides ────────────────────────────────────────────
    #[serde(default)]
    pub agent: AgentOptions,
    #[serde(default)]
    pub env: EnvOptions,
}

pub async fn run_handler(
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, (StatusCode, String)> {
    let output_dir = req.output_dir.clone().unwrap_or_else(|| "trajectories".to_string());
    let config = build_run_config(req, &output_dir)?;

    let run = RunSingle::from_run_config(config)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let result = run
        .run()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let steps = result.trajectory.len();
    let model_stats = serde_json::to_value(&result.info.model_stats).unwrap_or(serde_json::json!({}));

    let trajectory_file = result.info.extra.get("trajectory_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let submission_preview = result.info.submission.as_deref().map(|s| {
        let chars: String = s.chars().take(500).collect();
        if s.chars().count() > 500 { format!("{chars}…") } else { chars }
    });

    Ok(Json(RunResponse {
        exit_status: result.info.exit_status,
        has_submission: result.info.submission.is_some(),
        submission_preview,
        steps,
        model_stats,
        trajectory_file,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/run/batch
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BatchRunItem {
    pub github_url: Option<String>,
    pub repo: Option<String>,
    pub issue: Option<u64>,
    pub problem_text: Option<String>,
    #[serde(default)]
    pub agent: AgentOptions,
    #[serde(default)]
    pub env: EnvOptions,
}

#[derive(Debug, Deserialize)]
pub struct BatchRunRequest {
    pub items: Vec<BatchRunItem>,
    /// Directory for all trajectory output (default: "trajectories")
    pub output_dir: Option<String>,
    /// Max concurrent runs (default: 4)
    pub workers: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BatchRunResult {
    pub instance_id: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchRunResponse {
    pub output_dir: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<BatchRunResult>,
}

pub async fn batch_handler(
    body: Bytes,
) -> Result<Json<BatchRunResponse>, (StatusCode, String)> {
    let req: BatchRunRequest = serde_json::from_slice(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid request body: {e}"),
        )
    })?;
    let output_dir = req.output_dir.unwrap_or_else(|| "trajectories".to_string());
    let workers = req.workers.unwrap_or(4);

    let mut configs = Vec::with_capacity(req.items.len());
    for item in req.items {
        let cfg = build_run_config(
            RunRequest {
                github_url: item.github_url,
                repo: item.repo,
                issue: item.issue,
                problem_text: item.problem_text,
                output_dir: None,
                agent: item.agent,
                env: item.env,
            },
            &output_dir,
        )?;
        configs.push(cfg);
    }

    let batch = RunBatch::new(configs, PathBuf::from(&output_dir), workers);
    let raw = batch.run().await;

    let results: Vec<BatchRunResult> = raw
        .into_iter()
        .map(|(id, res)| BatchRunResult {
            instance_id: id,
            success: res.is_ok(),
            error: res.err().map(|e| e.to_string()),
        })
        .collect();

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.len() - succeeded;

    Ok(Json(BatchRunResponse {
        output_dir,
        total: results.len(),
        succeeded,
        failed,
        results,
    }))
}

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

pub fn build_run_config(
    req: RunRequest,
    output_dir: &str,
) -> Result<RunConfig, (StatusCode, String)> {
    let problem_statement = match (req.github_url, req.repo, req.issue, req.problem_text) {
        (Some(url), _, _, _) => ProblemStatementConfigSerde::GithubIssue { url },
        (_, Some(repo), Some(issue), _) => {
            let url = format!("https://github.com/{}/issues/{}", repo, issue);
            ProblemStatementConfigSerde::GithubIssue { url }
        }
        (_, _, _, Some(text)) => ProblemStatementConfigSerde::Text { text },
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Provide github_url, repo+issue, or problem_text".to_string(),
            ))
        }
    };

    Ok(RunConfig {
        agent: AgentConfigSerde {
            model_name: req.agent.model,
            base_url: req.agent.base_url,
            api_key: req.agent.api_key,
            parser_type: req.agent.parser_type,
            max_steps: req.agent.max_steps,
            max_requeries: req.agent.max_requeries,
            system_template: req.agent.system_template,
            instance_template: req.agent.instance_template,
        },
        env: EnvConfigSerde {
            image: req.env.image,
            container_name: req.env.container_name,
            repo_path: req.env.repo_path,
            timeout_secs: req.env.timeout_secs,
            startup_commands: req.env.startup_commands,
            env_vars: req.env.env_vars,
            base_commit: req.env.base_commit,
        },
        problem_statement,
        output_dir: output_dir.to_string(),
    })
}
