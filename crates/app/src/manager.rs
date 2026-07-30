use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use directories::BaseDirs;
use indoc::indoc;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use tokio::{
    fs,
    sync::{Mutex, RwLock},
};
use uuid::Uuid;

use crate::models::Project;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct ProjectHandle {
    pub project: Project,
    pub pool: SqlitePool,
    path: PathBuf,
}

#[derive(Clone)]
pub struct ProjectManager {
    root: Arc<PathBuf>,
    projects: Arc<RwLock<BTreeMap<String, ProjectHandle>>>,
    operations: Arc<Mutex<()>>,
}

impl ProjectManager {
    pub async fn open(root: Option<PathBuf>) -> Result<Self> {
        let base = match root {
            Some(root) => root,
            None => default_storage_base()?,
        };
        let root = base.join("asapi-storage").join("projects");
        fs::create_dir_all(&root)
            .await
            .with_context(|| format!("failed to create project directory {}", root.display()))?;

        let manager = Self {
            root: Arc::new(root),
            projects: Arc::new(RwLock::new(BTreeMap::new())),
            operations: Arc::new(Mutex::new(())),
        };
        manager.load_existing().await?;
        if manager.projects.read().await.is_empty() {
            manager.create("Default").await?;
        }
        Ok(manager)
    }

    pub async fn list(&self) -> Vec<Project> {
        self.projects
            .read()
            .await
            .values()
            .map(|handle| handle.project.clone())
            .collect()
    }

    pub async fn get(&self, id: &str) -> Result<ProjectHandle> {
        self.projects
            .read()
            .await
            .get(id)
            .cloned()
            .with_context(|| format!("project '{id}' was not found"))
    }

    pub async fn create(&self, name: &str) -> Result<Project> {
        let _operation = self.operations.lock().await;
        let name = validated_project_name(name)?;
        {
            let projects = self.projects.read().await;
            if projects
                .values()
                .any(|handle| handle.project.name.eq_ignore_ascii_case(&name))
            {
                bail!("a project named '{name}' already exists");
            }
        }

        let id = Uuid::new_v4().to_string();
        let path = self.root.join(format!("{id}.sqlite3"));
        let pool = open_pool(&path).await?;
        MIGRATOR
            .run(&pool)
            .await
            .context("failed to migrate project database")?;
        let created_at = chrono::Utc::now().to_rfc3339();
        sqlx::query(indoc! {r#"
            INSERT INTO project (singleton, id, name, created_at)
            VALUES (1, ?, ?, ?)
        "#})
        .bind(&id)
        .bind(&name)
        .bind(&created_at)
        .execute(&pool)
        .await
        .context("failed to initialize project")?;

        let project = Project {
            id: id.clone(),
            name,
            created_at,
        };
        self.projects.write().await.insert(
            id,
            ProjectHandle {
                project: project.clone(),
                pool,
                path,
            },
        );
        Ok(project)
    }

    pub async fn rename(&self, id: &str, name: &str) -> Result<Project> {
        let _operation = self.operations.lock().await;
        let name = validated_project_name(name)?;
        let handle = self.get(id).await?;
        {
            let projects = self.projects.read().await;
            if projects.values().any(|candidate| {
                candidate.project.id != id && candidate.project.name.eq_ignore_ascii_case(&name)
            }) {
                bail!("a project named '{name}' already exists");
            }
        }
        sqlx::query(indoc! {r#"
            UPDATE project
            SET name = ?
            WHERE singleton = 1
        "#})
        .bind(&name)
        .execute(&handle.pool)
        .await
        .context("failed to rename project")?;

        let mut projects = self.projects.write().await;
        let stored = projects
            .get_mut(id)
            .with_context(|| format!("project '{id}' was not found"))?;
        stored.project.name = name;
        Ok(stored.project.clone())
    }

    pub async fn delete(&self, id: &str) -> Result<Project> {
        let _operation = self.operations.lock().await;
        {
            let projects = self.projects.read().await;
            if projects.len() == 1 {
                bail!("the last project cannot be deleted");
            }
        }
        let handle = self
            .projects
            .write()
            .await
            .remove(id)
            .with_context(|| format!("project '{id}' was not found"))?;
        handle.pool.close().await;
        remove_database_files(&handle.path).await?;
        Ok(handle.project)
    }

    async fn load_existing(&self) -> Result<()> {
        let mut entries = fs::read_dir(self.root.as_ref())
            .await
            .with_context(|| format!("failed to scan {}", self.root.display()))?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("sqlite3") {
                continue;
            }
            let pool = open_pool(&path).await?;
            MIGRATOR
                .run(&pool)
                .await
                .with_context(|| format!("failed to migrate {}", path.display()))?;
            let project = sqlx::query_as::<_, Project>(indoc! {r#"
                SELECT id, name, created_at
                FROM project
                WHERE singleton = 1
            "#})
            .fetch_optional(&pool)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;
            if let Some(project) = project {
                self.projects.write().await.insert(
                    project.id.clone(),
                    ProjectHandle {
                        project,
                        pool,
                        path,
                    },
                );
            } else {
                pool.close().await;
            }
        }
        Ok(())
    }
}

fn default_storage_base() -> Result<PathBuf> {
    let dirs =
        BaseDirs::new().context("could not determine the operating system home directory")?;
    Ok(dirs.home_dir().join(".local").join("share"))
}

fn validated_project_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        bail!("project name must not be empty");
    }
    if name.chars().count() > 80 {
        bail!("project name must be at most 80 characters");
    }
    Ok(name.to_string())
}

async fn open_pool(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));
    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .with_context(|| format!("failed to open {}", path.display()))
}

async fn remove_database_files(path: &Path) -> Result<()> {
    for candidate in [
        path.to_path_buf(),
        PathBuf::from(format!("{}-wal", path.display())),
        PathBuf::from(format!("{}-shm", path.display())),
    ] {
        match fs::remove_file(&candidate).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to remove {}", candidate.display()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn storage_uses_the_documented_layout_and_creates_default() {
        let temporary = tempfile::tempdir().unwrap();
        let manager = ProjectManager::open(Some(temporary.path().to_path_buf()))
            .await
            .unwrap();
        let projects = manager.list().await;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Default");

        let storage = temporary.path().join("asapi-storage").join("projects");
        let database = std::fs::read_dir(storage)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().and_then(|value| value.to_str()) == Some("sqlite3"))
            .unwrap();
        assert_eq!(
            database.extension().and_then(|value| value.to_str()),
            Some("sqlite3")
        );
    }

    #[tokio::test]
    async fn projects_are_isolated_databases_and_last_project_is_protected() {
        let temporary = tempfile::tempdir().unwrap();
        let manager = ProjectManager::open(Some(temporary.path().to_path_buf()))
            .await
            .unwrap();
        let default = manager.list().await.remove(0);
        let second = manager.create("Competitors").await.unwrap();
        manager.rename(&second.id, "Launch").await.unwrap();
        assert_eq!(
            manager.get(&second.id).await.unwrap().project.name,
            "Launch"
        );

        manager.delete(&default.id).await.unwrap();
        assert!(manager.delete(&second.id).await.is_err());
        assert_eq!(manager.list().await.len(), 1);
    }
}
