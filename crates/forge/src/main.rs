use clap::{Parser, Subcommand};
use forge_run::config::{ProblemStatementConfigSerde, RunConfig};
use forge_run::run_single::RunSingle;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(
    name = "forge",
    version = "0.1.0",
    about = "Forge — autonomous AI software-engineering agent",
    long_about = "Forge clones a repository into an isolated Docker sandbox and autonomously \
                  fixes GitHub issues using any OpenAI-compatible model API."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List open issues on a GitHub repository
    ListIssues(ListIssuesArgs),
    /// Run the agent on a single issue or problem
    Run(RunArgs),
    /// Watch a repository and automatically fix issues as they are labelled
    Watch(WatchArgs),
    /// Display trajectory statistics
    QuickStats(QuickStatsArgs),
}

// ---------------------------------------------------------------------------
// list-issues
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
struct ListIssuesArgs {
    /// GitHub repository to scan (owner/repo)
    #[arg(long)]
    repo: String,

    /// Filter by label (e.g. "bug", "help wanted")
    #[arg(long)]
    label: Option<String>,

    /// Show up to this many issues (default: 30)
    #[arg(long, default_value_t = 30)]
    limit: u32,
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
struct RunArgs {
    /// Path to YAML config file
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    /// GitHub repository (owner/repo) — pair with --issue
    #[arg(long)]
    repo: Option<String>,

    /// GitHub issue number — pair with --repo
    #[arg(long)]
    issue: Option<u64>,

    /// Full GitHub issue URL (alternative to --repo + --issue)
    #[arg(long)]
    github_url: Option<String>,

    /// Problem statement as plain text
    #[arg(long)]
    problem_text: Option<String>,

    /// Path to a file containing the problem statement
    #[arg(long)]
    problem_file: Option<std::path::PathBuf>,

    /// Model name
    #[arg(long, env = "FORGE_MODEL")]
    model: Option<String>,

    /// Model base URL (OpenAI-compatible)
    #[arg(long, env = "FORGE_BASE_URL")]
    base_url: Option<String>,

    /// API key
    #[arg(long, env = "FORGE_API_KEY")]
    api_key: Option<String>,

    /// Docker sandbox image
    #[arg(long, default_value = "forge-sandbox:latest")]
    image: String,

    /// Output directory for trajectory files
    #[arg(long, default_value = "trajectories")]
    output_dir: String,

    /// Maximum agent steps before giving up
    #[arg(long, default_value_t = 100)]
    max_steps: u32,
}

// ---------------------------------------------------------------------------
// watch
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
struct WatchArgs {
    /// GitHub repository to watch (owner/repo)
    #[arg(long, env = "FORGE_WATCH_REPO")]
    repo: String,

    /// Only process issues that carry this label
    #[arg(long, env = "FORGE_WATCH_LABEL", default_value = "forge")]
    label: String,

    /// Seconds between polls
    #[arg(long, env = "FORGE_WATCH_INTERVAL", default_value_t = 60)]
    interval: u64,

    /// Model name
    #[arg(long, env = "FORGE_MODEL")]
    model: Option<String>,

    /// Model base URL (OpenAI-compatible)
    #[arg(long, env = "FORGE_BASE_URL")]
    base_url: Option<String>,

    /// API key
    #[arg(long, env = "FORGE_API_KEY")]
    api_key: Option<String>,

    /// Docker sandbox image
    #[arg(long, env = "FORGE_SANDBOX_IMAGE", default_value = "forge-sandbox:latest")]
    image: String,

    /// Output directory for trajectory files
    #[arg(long, default_value = "trajectories")]
    output_dir: String,

    /// Maximum agent steps per issue
    #[arg(long, default_value_t = 100)]
    max_steps: u32,
}

// ---------------------------------------------------------------------------
// quick-stats
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
struct QuickStatsArgs {
    /// Directory to scan for trajectory files
    #[arg(default_value = "trajectories")]
    directory: std::path::PathBuf,
}

// ---------------------------------------------------------------------------
// GitHub API types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    html_url: String,
    labels: Vec<GitHubLabel>,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

// Watch state — tracks which issues have already been processed.
#[derive(Debug, Default, Serialize, Deserialize)]
struct WatchState {
    processed: std::collections::HashSet<u64>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "forge=info"
                    .parse()
                    .expect("'forge=info' is a valid tracing directive"),
            ),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::ListIssues(args) => list_issues_command(args).await,
        Commands::Run(args) => run_command(args).await,
        Commands::Watch(args) => watch_command(args).await,
        Commands::QuickStats(args) => quick_stats_command(args).await,
    }
}

// ---------------------------------------------------------------------------
// list-issues command
// ---------------------------------------------------------------------------

async fn list_issues_command(args: ListIssuesArgs) {
    let client = github_client();
    let issues = match fetch_issues(&client, &args.repo, args.label.as_deref(), args.limit).await {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error fetching issues: {e}");
            std::process::exit(1);
        }
    };

    if issues.is_empty() {
        let filter = args
            .label
            .as_deref()
            .map(|l| format!(" with label '{l}'"))
            .unwrap_or_default();
        println!("No open issues found on {}{filter}.", args.repo);
        return;
    }

    println!("Open issues on {} ({} shown):\n", args.repo, issues.len());
    println!("{:<6}  {}", "#", "Title");
    println!("{}", "-".repeat(72));
    for issue in &issues {
        let labels: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();
        let label_str = if labels.is_empty() {
            String::new()
        } else {
            format!("  [{}]", labels.join(", "))
        };
        println!("#{:<5}  {}{}", issue.number, issue.title, label_str);
    }

    println!();
    println!("To fix an issue, run:");
    println!("  forge run --repo {} --issue <number>", args.repo);
}

// ---------------------------------------------------------------------------
// run command
// ---------------------------------------------------------------------------

async fn run_command(args: RunArgs) {
    let mut config = if let Some(ref cfg_path) = args.config {
        match RunConfig::from_yaml_file(cfg_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {e}");
                std::process::exit(1);
            }
        }
    } else {
        RunConfig::default()
    };

    if let Some(model) = args.model {
        config.agent.model_name = Some(model);
    }
    if let Some(url) = args.base_url {
        config.agent.base_url = Some(url);
    }
    if let Some(key) = args.api_key {
        config.agent.api_key = Some(key);
    }
    if args.image != "forge-sandbox:latest" || config.env.image.is_none() {
        config.env.image = Some(args.image);
    }
    config.output_dir = args.output_dir;
    config.agent.max_steps = Some(args.max_steps);

    // Resolve problem statement: --repo+--issue  >  --github-url  >  text/file  >  config
    if let (Some(repo), Some(number)) = (args.repo, args.issue) {
        let url = format!("https://github.com/{}/issues/{}", repo, number);
        config.problem_statement = ProblemStatementConfigSerde::GithubIssue { url };
    } else if let Some(url) = args.github_url {
        config.problem_statement = ProblemStatementConfigSerde::GithubIssue { url };
    } else if let Some(text) = args.problem_text {
        config.problem_statement = ProblemStatementConfigSerde::Text { text };
    } else if let Some(path) = args.problem_file {
        config.problem_statement = ProblemStatementConfigSerde::TextFile { path };
    }

    let run = match RunSingle::from_run_config(config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1);
        }
    };

    match run.run().await {
        Ok(result) => {
            let exit = result
                .info
                .exit_status
                .unwrap_or_else(|| "unknown".to_string());
            println!("Run complete. Exit status: {exit}");
            if let Some(sub) = result.info.submission {
                let preview: String = sub.chars().take(200).collect();
                println!("Patch preview:\n{preview}");
            }
        }
        Err(e) => {
            eprintln!("Run failed: {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// watch command
// ---------------------------------------------------------------------------

async fn watch_command(args: WatchArgs) {
    println!("Watching {} for issues labelled '{}'", args.repo, args.label);
    println!("Polling every {}s. Press Ctrl+C to stop.\n", args.interval);

    let client = github_client();
    let state_path = format!("{}/watch_state.json", args.output_dir);

    // Create output directory if needed
    if let Err(e) = tokio::fs::create_dir_all(&args.output_dir).await {
        eprintln!("Cannot create output dir: {e}");
        std::process::exit(1);
    }

    let mut state = load_watch_state(&state_path).await;

    loop {
        match fetch_issues(&client, &args.repo, Some(&args.label), 100).await {
            Ok(issues) => {
                let new_issues: Vec<&GitHubIssue> = issues
                    .iter()
                    .filter(|i| !state.processed.contains(&i.number))
                    .collect();

                if new_issues.is_empty() {
                    println!("No new issues to process. Next check in {}s.", args.interval);
                } else {
                    println!("Found {} new issue(s) to process.", new_issues.len());
                    for issue in new_issues {
                        println!("\nProcessing #{}: {}", issue.number, issue.title);
                        let result = run_single_issue(
                            &issue.html_url,
                            args.model.as_deref(),
                            args.base_url.as_deref(),
                            args.api_key.as_deref(),
                            &args.image,
                            &args.output_dir,
                            args.max_steps,
                        )
                        .await;

                        match result {
                            Ok(exit) => println!("  Done — status: {exit}"),
                            Err(e) => eprintln!("  Failed: {e}"),
                        }

                        // Mark processed regardless of outcome so we don't retry forever.
                        state.processed.insert(issue.number);
                        save_watch_state(&state_path, &state).await;
                    }
                }
            }
            Err(e) => eprintln!("GitHub API error: {e}"),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(args.interval)).await;
    }
}

// ---------------------------------------------------------------------------
// quick-stats command
// ---------------------------------------------------------------------------

async fn quick_stats_command(args: QuickStatsArgs) {
    use forge_types::trajectory::TrajFile;

    let dir = args.directory.clone();
    if !dir.exists() {
        eprintln!("Directory {:?} does not exist", dir);
        return;
    }

    let mut total = 0usize;
    let mut submitted = 0usize;
    let mut forfeited = 0usize;
    let mut errors = 0usize;
    let mut step_limit = 0usize;

    if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("traj") {
                total += 1;
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(traj) = serde_json::from_str::<TrajFile>(&content) {
                        match traj.info.exit_status.as_deref() {
                            Some("submitted") => submitted += 1,
                            Some("forfeited") => forfeited += 1,
                            Some("error") => errors += 1,
                            Some("step_limit_reached") => step_limit += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    println!("Trajectory stats for {:?}:", dir);
    println!("  Total:             {total}");
    println!("  Submitted:         {submitted}");
    println!("  Forfeited:         {forfeited}");
    println!("  Errors:            {errors}");
    println!("  Step limit:        {step_limit}");
    println!(
        "  Other:             {}",
        total.saturating_sub(submitted + forfeited + errors + step_limit)
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn github_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        "application/vnd.github.v3+json".parse().unwrap(),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        "Forge/0.1".parse().unwrap(),
    );
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("failed to build HTTP client")
}

async fn fetch_issues(
    client: &reqwest::Client,
    repo: &str,
    label: Option<&str>,
    limit: u32,
) -> Result<Vec<GitHubIssue>, String> {
    let per_page = limit.min(100);
    let mut url = format!(
        "https://api.github.com/repos/{}/issues?state=open&per_page={}",
        repo, per_page
    );
    if let Some(l) = label {
        url.push_str(&format!("&labels={}", l));
    }

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

async fn run_single_issue(
    github_url: &str,
    model: Option<&str>,
    base_url: Option<&str>,
    api_key: Option<&str>,
    image: &str,
    output_dir: &str,
    max_steps: u32,
) -> Result<String, String> {
    let mut config = RunConfig::default();
    config.agent.model_name = model.map(|s| s.to_string());
    config.agent.base_url = base_url.map(|s| s.to_string());
    config.agent.api_key = api_key.map(|s| s.to_string());
    config.agent.max_steps = Some(max_steps);
    config.env.image = Some(image.to_string());
    config.output_dir = output_dir.to_string();
    config.problem_statement = ProblemStatementConfigSerde::GithubIssue {
        url: github_url.to_string(),
    };

    let run = RunSingle::from_run_config(config).map_err(|e| e.to_string())?;
    let result = run.run().await.map_err(|e| e.to_string())?;
    Ok(result
        .info
        .exit_status
        .unwrap_or_else(|| "unknown".to_string()))
}

async fn load_watch_state(path: &str) -> WatchState {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        WatchState::default()
    }
}

async fn save_watch_state(path: &str, state: &WatchState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = tokio::fs::write(path, json).await;
    }
}
