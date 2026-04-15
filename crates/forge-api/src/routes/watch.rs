use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use forge_run::config::{AgentConfigSerde, EnvConfigSerde, ProblemStatementConfigSerde, RunConfig};
use forge_run::run_single::RunSingle;

// ---------------------------------------------------------------------------
// Shared state for the watch task
// ---------------------------------------------------------------------------

pub struct WatchTask {
    pub repo: String,
    pub label: String,
    pub handle: tokio::task::JoinHandle<()>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

pub type WatchState = Arc<Mutex<Option<WatchTask>>>;

pub fn new_watch_state() -> WatchState {
    Arc::new(Mutex::new(None))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WatchRequest {
    /// GitHub repository to watch (owner/repo)
    pub repo: String,
    /// Label that triggers the agent (default: "forge")
    #[serde(default = "default_label")]
    pub label: String,
    /// Seconds between polls (default: 60)
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// Model name
    pub model: Option<String>,
    /// Model base URL
    pub base_url: Option<String>,
    /// API key
    pub api_key: Option<String>,
    /// Docker sandbox image
    pub image: Option<String>,
    /// Max steps per issue
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    /// Output directory
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_label() -> String { "forge".to_string() }
fn default_interval() -> u64 { 60 }
fn default_max_steps() -> u32 { 100 }
fn default_output_dir() -> String { "trajectories".to_string() }

#[derive(Debug, Serialize)]
pub struct WatchStatusResponse {
    pub running: bool,
    pub repo: Option<String>,
    pub label: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    html_url: String,
}

// ---------------------------------------------------------------------------
// POST /api/watch — start watching
// ---------------------------------------------------------------------------

pub async fn start_handler(
    State(watch_state): State<WatchState>,
    Json(req): Json<WatchRequest>,
) -> Result<Json<WatchStatusResponse>, (StatusCode, String)> {
    let mut guard = watch_state.lock().await;

    if let Some(task) = guard.as_ref() {
        if !task.handle.is_finished() {
            return Err((
                StatusCode::CONFLICT,
                format!("Watch already running for {}. Stop it first.", task.repo),
            ));
        }
    }

    let repo = req.repo.clone();
    let label = req.label.clone();
    let started_at = chrono::Utc::now();

    let handle = tokio::spawn(watch_loop(req));

    *guard = Some(WatchTask {
        repo: repo.clone(),
        label: label.clone(),
        handle,
        started_at,
    });

    Ok(Json(WatchStatusResponse {
        running: true,
        repo: Some(repo),
        label: Some(label),
        started_at: Some(started_at),
    }))
}

// ---------------------------------------------------------------------------
// GET /api/watch — status
// ---------------------------------------------------------------------------

pub async fn status_handler(
    State(watch_state): State<WatchState>,
) -> Json<WatchStatusResponse> {
    let guard = watch_state.lock().await;

    match guard.as_ref() {
        None => Json(WatchStatusResponse {
            running: false,
            repo: None,
            label: None,
            started_at: None,
        }),
        Some(task) => Json(WatchStatusResponse {
            running: !task.handle.is_finished(),
            repo: Some(task.repo.clone()),
            label: Some(task.label.clone()),
            started_at: Some(task.started_at),
        }),
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/watch — stop watching
// ---------------------------------------------------------------------------

pub async fn stop_handler(
    State(watch_state): State<WatchState>,
) -> Result<Json<WatchStatusResponse>, (StatusCode, String)> {
    let mut guard = watch_state.lock().await;

    match guard.take() {
        None => Err((StatusCode::NOT_FOUND, "No watch task is running".to_string())),
        Some(task) => {
            task.handle.abort();
            // Cleared fields after stop — clients expect nulls, not the last task metadata.
            Ok(Json(WatchStatusResponse {
                running: false,
                repo: None,
                label: None,
                started_at: None,
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// Watch loop (runs in background task)
// ---------------------------------------------------------------------------

async fn watch_loop(req: WatchRequest) {
    let client = build_github_client();
    let state_path = format!("{}/watch_state.json", req.output_dir);

    if let Err(e) = tokio::fs::create_dir_all(&req.output_dir).await {
        tracing::error!("Cannot create output dir: {e}");
        return;
    }

    let mut processed: HashSet<u64> = load_processed(&state_path).await;

    loop {
        match fetch_issues(&client, &req.repo, &req.label).await {
            Ok(issues) => {
                let new: Vec<&GitHubIssue> =
                    issues.iter().filter(|i| !processed.contains(&i.number)).collect();

                if new.is_empty() {
                    tracing::debug!("No new issues. Next poll in {}s.", req.interval);
                } else {
                    tracing::info!("Found {} new issue(s).", new.len());
                    for issue in new {
                        tracing::info!("Processing #{}: {}", issue.number, issue.title);
                        let result = run_issue(
                            &issue.html_url,
                            req.model.as_deref(),
                            req.base_url.as_deref(),
                            req.api_key.as_deref(),
                            req.image.as_deref(),
                            &req.output_dir,
                            req.max_steps,
                        )
                        .await;

                        match result {
                            Ok(status) => tracing::info!("  Done — {status}"),
                            Err(e) => tracing::error!("  Failed: {e}"),
                        }

                        processed.insert(issue.number);
                        save_processed(&state_path, &processed).await;
                    }
                }
            }
            Err(e) => tracing::error!("GitHub API error: {e}"),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(req.interval)).await;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn run_issue(
    github_url: &str,
    model: Option<&str>,
    base_url: Option<&str>,
    api_key: Option<&str>,
    image: Option<&str>,
    output_dir: &str,
    max_steps: u32,
) -> Result<String, String> {
    let config = RunConfig {
        agent: AgentConfigSerde {
            model_name: model.map(|s| s.to_string()),
            base_url: base_url.map(|s| s.to_string()),
            api_key: api_key.map(|s| s.to_string()),
            max_steps: Some(max_steps),
            ..Default::default()
        },
        env: EnvConfigSerde {
            image: image.map(|s| s.to_string()),
            ..Default::default()
        },
        problem_statement: ProblemStatementConfigSerde::GithubIssue {
            url: github_url.to_string(),
        },
        output_dir: output_dir.to_string(),
    };

    let run = RunSingle::from_run_config(config).map_err(|e| e.to_string())?;
    let result = run.run().await.map_err(|e| e.to_string())?;
    Ok(result.info.exit_status.unwrap_or_else(|| "unknown".to_string()))
}

async fn fetch_issues(
    client: &reqwest::Client,
    repo: &str,
    label: &str,
) -> Result<Vec<GitHubIssue>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/issues?state=open&per_page=100&labels={}",
        repo, label
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API returned {status}: {body}"));
    }

    resp.json::<Vec<GitHubIssue>>()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))
}

fn build_github_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    headers.insert(reqwest::header::USER_AGENT, "Forge/0.1".parse().unwrap());
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            if let Ok(val) = format!("Bearer {}", token).parse() {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("http client")
}

async fn load_processed(path: &str) -> HashSet<u64> {
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .and_then(|s| serde_json::from_str::<HashSet<u64>>(&s).ok())
        .unwrap_or_default()
}

async fn save_processed(path: &str, processed: &HashSet<u64>) {
    if let Ok(json) = serde_json::to_string(processed) {
        let _ = tokio::fs::write(path, json).await;
    }
}
