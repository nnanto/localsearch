use directories::ProjectDirs;
use std::path::PathBuf;

/// Configuration for localsearch project directories
pub struct LocalSearchDirs {
    project_dirs: Option<ProjectDirs>,
}

impl LocalSearchDirs {
    /// Create a new LocalSearchDirs instance
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("com", "localsearch", "localsearch");
        Self { project_dirs }
    }

    /// Get the default cache directory for embeddings
    pub fn default_cache_dir(&self) -> PathBuf {
        match &self.project_dirs {
            Some(dirs) => dirs.cache_dir().to_path_buf(),
            None => {
                // Fallback to current directory if ProjectDirs fails
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".cache")
            }
        }
    }

    /// Get the default database directory
    pub fn default_db_dir(&self) -> PathBuf {
        match &self.project_dirs {
            Some(dirs) => dirs.data_dir().to_path_buf(),
            None => {
                // Fallback to current directory if ProjectDirs fails
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            }
        }
    }

    /// Get the default database file path
    pub fn default_db_path(&self) -> PathBuf {
        self.default_db_dir().join("localsearch.db")
    }

    /// Ensure the cache directory exists
    pub fn ensure_cache_dir(&self) -> std::io::Result<PathBuf> {
        let cache_dir = self.default_cache_dir();
        std::fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    /// Ensure the database directory exists
    pub fn ensure_db_dir(&self) -> std::io::Result<PathBuf> {
        let db_dir = self.default_db_dir();
        std::fs::create_dir_all(&db_dir)?;
        Ok(db_dir)
    }
}

impl Default for LocalSearchDirs {
    fn default() -> Self {
        Self::new()
    }
}