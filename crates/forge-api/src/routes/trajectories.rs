use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use forge_types::trajectory::TrajFile;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Query / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DirQuery {
    #[serde(default = "default_dir")]
    pub dir: String,
}

fn default_dir() -> String {
    "trajectories".to_string()
}

#[derive(Debug, Serialize)]
pub struct TrajSummary {
    pub name: String,
    /// Always a string for API consumers (`""` when unknown or file unreadable).
    pub exit_status: String,
    pub has_submission: bool,
    pub steps: usize,
    pub model_stats: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TrajListResponse {
    pub directory: String,
    pub count: usize,
    pub trajectories: Vec<TrajSummary>,
}

// ---------------------------------------------------------------------------
// GET /api/trajectories?dir=trajectories
// ---------------------------------------------------------------------------

pub async fn list_handler(
    Query(params): Query<DirQuery>,
) -> Result<Json<TrajListResponse>, (StatusCode, String)> {
    let dir = std::path::PathBuf::from(&params.dir);

    if !dir.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Directory {:?} does not exist", dir),
        ));
    }

    let mut summaries = Vec::new();

    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("traj") {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            if let Ok(traj) = serde_json::from_str::<TrajFile>(&content) {
                let model_stats =
                    serde_json::to_value(&traj.info.model_stats).unwrap_or(serde_json::json!({}));
                summaries.push(TrajSummary {
                    name,
                    exit_status: traj.info.exit_status.unwrap_or_default(),
                    has_submission: traj.info.submission.is_some(),
                    steps: traj.trajectory.len(),
                    model_stats,
                });
                continue;
            }
        }

        // File exists but couldn't be parsed — include with empty exit_status
        summaries.push(TrajSummary {
            name,
            exit_status: String::new(),
            has_submission: false,
            steps: 0,
            model_stats: serde_json::json!({}),
        });
    }

    summaries.sort_by(|a, b| a.name.cmp(&b.name));
    let count = summaries.len();

    Ok(Json(TrajListResponse {
        directory: params.dir,
        count,
        trajectories: summaries,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/trajectories/:name?dir=trajectories
// ---------------------------------------------------------------------------

pub async fn get_handler(
    Path(name): Path<String>,
    Query(params): Query<DirQuery>,
) -> Result<Json<TrajFile>, (StatusCode, String)> {
    let safe_name = name.replace(['/', '\\', '\0'], "_");
    let path = std::path::PathBuf::from(&params.dir).join(&safe_name);

    let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("Cannot read {:?}: {e}", path),
        )
    })?;

    let traj: TrajFile = serde_json::from_str(&content)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Parse error: {e}")))?;

    Ok(Json(traj))
}
