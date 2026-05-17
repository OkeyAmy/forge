use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use forge_types::public_workflow::{ForgeCommand, ForgeGitHubEvent, ForgeJob, ForgeJobState};
use forge_types::ForgeError;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct FileJobStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl FileJobStore {
    pub fn from_env() -> Self {
        let path = std::env::var("FORGE_JOB_STORE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".forge/jobs.json"));
        Self::new(path)
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn insert_event(&self, event: ForgeGitHubEvent) -> Result<ForgeJob, ForgeError> {
        let _guard = self.lock.lock().await;
        let mut jobs = self.read_all_unlocked().await?;
        let now = unix_now();
        let job = ForgeJob {
            id: job_id(&event, now),
            state: ForgeJobState::Received,
            event,
            created_at_unix_secs: now,
            updated_at_unix_secs: now,
            plan: None,
            branch_name: None,
            pull_request_url: None,
            pull_request_number: None,
            error: None,
        };
        jobs.push(job.clone());
        self.write_all_unlocked(&jobs).await?;
        Ok(job)
    }

    pub async fn all(&self) -> Result<Vec<ForgeJob>, ForgeError> {
        let _guard = self.lock.lock().await;
        self.read_all_unlocked().await
    }

    pub async fn get(&self, id: &str) -> Result<Option<ForgeJob>, ForgeError> {
        Ok(self.all().await?.into_iter().find(|job| job.id == id))
    }

    pub async fn find_by_delivery_id(
        &self,
        delivery_id: &str,
    ) -> Result<Option<ForgeJob>, ForgeError> {
        Ok(self
            .all()
            .await?
            .into_iter()
            .find(|job| job.event.delivery_id == delivery_id))
    }

    pub async fn update<F>(&self, id: &str, update: F) -> Result<ForgeJob, ForgeError>
    where
        F: FnOnce(&mut ForgeJob),
    {
        let _guard = self.lock.lock().await;
        let mut jobs = self.read_all_unlocked().await?;
        let job = jobs
            .iter_mut()
            .find(|job| job.id == id)
            .ok_or_else(|| ForgeError::Config(format!("job {id} was not found")))?;
        update(job);
        job.updated_at_unix_secs = unix_now();
        let updated = job.clone();
        self.write_all_unlocked(&jobs).await?;
        Ok(updated)
    }

    pub async fn latest_waiting_issue_plan(
        &self,
        repository_full_name: &str,
        issue_number: u64,
    ) -> Result<Option<ForgeJob>, ForgeError> {
        let jobs = self.all().await?;
        Ok(jobs.into_iter().rev().find(|job| {
            job.event.repository.full_name == repository_full_name
                && job.issue_number() == Some(issue_number)
                && job.event.command == ForgeCommand::Plan
                && job.state == ForgeJobState::WaitingForApproval
        }))
    }

    async fn read_all_unlocked(&self) -> Result<Vec<ForgeJob>, ForgeError> {
        if tokio::fs::metadata(&self.path).await.is_err() {
            return Ok(Vec::new());
        }
        let data = tokio::fs::read(&self.path).await.map_err(ForgeError::Io)?;
        if data.is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_slice(&data).map_err(ForgeError::Json)
    }

    async fn write_all_unlocked(&self, jobs: &[ForgeJob]) -> Result<(), ForgeError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(ForgeError::Io)?;
        }
        let tmp_path = tmp_path_for(&self.path);
        let data = serde_json::to_vec_pretty(jobs).map_err(ForgeError::Json)?;
        tokio::fs::write(&tmp_path, data)
            .await
            .map_err(ForgeError::Io)?;
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .map_err(ForgeError::Io)
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn job_id(event: &ForgeGitHubEvent, timestamp: u64) -> String {
    format!(
        "{}-{}-{timestamp}",
        event.delivery_id,
        command_slug(&event.command)
    )
}

fn command_slug(command: &ForgeCommand) -> &'static str {
    match command {
        ForgeCommand::Plan => "plan",
        ForgeCommand::Approve => "approve",
        ForgeCommand::Status => "status",
        ForgeCommand::Cancel => "cancel",
        ForgeCommand::Review => "review",
        ForgeCommand::Improve => "improve",
        ForgeCommand::Feedback { .. } => "feedback",
        ForgeCommand::Ask { .. } => "ask",
        ForgeCommand::Fix => "fix",
    }
}

fn tmp_path_for(path: &Path) -> PathBuf {
    path.with_extension(format!("tmp.{}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_types::public_workflow::{GitHubActor, GitHubRepositoryRef, GitHubSubject};

    fn test_store_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "forge-job-store-{name}-{}-{}.json",
            std::process::id(),
            unix_now()
        ))
    }

    fn issue_event(
        delivery_id: &str,
        command: ForgeCommand,
        issue_number: u64,
    ) -> ForgeGitHubEvent {
        ForgeGitHubEvent {
            delivery_id: delivery_id.to_string(),
            installation_id: Some(42),
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
                number: issue_number,
                title: "Bug".to_string(),
                body: Some("Fix it".to_string()),
                html_url: "https://github.com/acme/app/issues/7".to_string(),
            },
            command,
        }
    }

    #[tokio::test]
    async fn file_store_persists_jobs_across_reloads() {
        let path = test_store_path("reload");
        let store = FileJobStore::new(&path);
        let inserted = store
            .insert_event(issue_event("delivery-1", ForgeCommand::Plan, 7))
            .await
            .unwrap();

        let reloaded = FileJobStore::new(&path);
        let jobs = reloaded.all().await.unwrap();

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, inserted.id);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn latest_waiting_issue_plan_ignores_non_waiting_jobs() {
        let path = test_store_path("latest");
        let store = FileJobStore::new(&path);
        let first = store
            .insert_event(issue_event("delivery-1", ForgeCommand::Plan, 7))
            .await
            .unwrap();
        store
            .update(&first.id, |job| job.state = ForgeJobState::Failed)
            .await
            .unwrap();
        let second = store
            .insert_event(issue_event("delivery-2", ForgeCommand::Plan, 7))
            .await
            .unwrap();
        store
            .update(&second.id, |job| {
                job.state = ForgeJobState::WaitingForApproval
            })
            .await
            .unwrap();

        let found = store
            .latest_waiting_issue_plan("acme/app", 7)
            .await
            .unwrap();

        assert_eq!(found.unwrap().id, second.id);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn find_by_delivery_id_returns_existing_job() {
        let path = test_store_path("delivery");
        let store = FileJobStore::new(&path);
        let inserted = store
            .insert_event(issue_event("delivery-1", ForgeCommand::Plan, 7))
            .await
            .unwrap();

        let found = store.find_by_delivery_id("delivery-1").await.unwrap();
        let missing = store.find_by_delivery_id("delivery-2").await.unwrap();

        assert_eq!(found.unwrap().id, inserted.id);
        assert!(missing.is_none());
        let _ = std::fs::remove_file(path);
    }
}
