use anyhow::{Result, anyhow};
use git2::{DiffOptions, Repository};
use std::path::{Path, PathBuf};
use std::str;

/// Trait defining the Git operations required by the engine.
/// This abstraction allows for easier testing and decoupling from specific git implementations.
pub trait GitClient {
    /// Returns the list of files currently staged in the index.
    fn get_staged_files(&self) -> Result<Vec<PathBuf>>;

    /// Reads the content of a file as it exists in the staging area (index).
    fn read_staged_file_content(&self, path: &Path) -> Result<String>;

    /// Stages a file (adds it to the index).
    fn stage_file(&self, path: &Path) -> Result<()>;

    /// Returns the root path of the repository.
    fn get_repo_root(&self) -> PathBuf;

    /// Returns the .git directory path
    fn get_git_dir(&self) -> PathBuf;

    /// Checks if a file exists in the working directory.
    fn file_exists(&self, path: &Path) -> bool;

    /// Read file from working directory
    fn read_working_file(&self, path: &Path) -> Result<String>;

    /// Write file to working directory
    fn write_working_file(&self, path: &Path, content: &str) -> Result<()>;

    /// Get all tracked files (for "all" pattern processing)
    fn get_tracked_files(&self) -> Result<Vec<String>>;
}

/// Concrete implementation of GitClient using the git2 crate.
pub struct Git2Client {
    repo: Repository,
}

impl Git2Client {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }
}

impl GitClient for Git2Client {
    fn get_staged_files(&self) -> Result<Vec<PathBuf>> {
        let index = self.repo.index()?;
        let mut staged_files = Vec::new();
        let mut options = DiffOptions::new();

        // Method 1: Try diff from HEAD to index (for existing repo with commits)
        if let Ok(head) = self.repo.head() {
            if let Ok(head_tree) = head.peel_to_tree() {
                let diff = self.repo.diff_tree_to_index(
                    Some(&head_tree),
                    Some(&index),
                    Some(&mut options),
                )?;

                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path() {
                        staged_files.push(path.to_path_buf());
                    }
                }
            }
        } else {
            // Method 2: For the initial commit, diff against an empty tree.
            let empty_tree = self.repo.treebuilder(None)?.write()?;
            let empty_tree_obj = self.repo.find_tree(empty_tree)?;
            let diff = self.repo.diff_tree_to_index(
                Some(&empty_tree_obj),
                Some(&index),
                Some(&mut options),
            )?;

            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path() {
                    staged_files.push(path.to_path_buf());
                }
            }
        }

        // Method 3: Fallback. Directly iterate through the index.
        if staged_files.is_empty() {
            let entry_count = index.len();
            for i in 0..entry_count {
                if let Some(entry) = index.get(i) {
                    let path = PathBuf::from(str::from_utf8(&entry.path)?);
                    staged_files.push(path);
                }
            }
        }

        Ok(staged_files)
    }

    fn read_staged_file_content(&self, path: &Path) -> Result<String> {
        let index = self.repo.index()?;
        let entry = index
            .get_path(path, 0)
            .ok_or_else(|| anyhow!("Failed to get staged file entry for {}", path.display()))?;
        let blob = self.repo.find_blob(entry.id)?;
        let content = str::from_utf8(blob.content())?;
        Ok(content.to_string())
    }

    fn stage_file(&self, path: &Path) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(path)?;
        index.write()?;
        Ok(())
    }

    fn get_repo_root(&self) -> PathBuf {
        self.repo
            .path()
            .parent()
            .unwrap_or(self.repo.path())
            .to_path_buf()
    }

    fn get_git_dir(&self) -> PathBuf {
        self.repo.path().to_path_buf()
    }

    fn file_exists(&self, path: &Path) -> bool {
        // Check relative to repo root (repo.path() is .git usually, so parent is root)
        // actually git2 repo.path() returns the .git directory path.
        let root = self.get_repo_root();
        root.join(path).exists()
    }

    fn read_working_file(&self, path: &Path) -> Result<String> {
        let root = self.get_repo_root();
        let content = std::fs::read_to_string(root.join(path))?;
        Ok(content)
    }

    fn write_working_file(&self, path: &Path, content: &str) -> Result<()> {
        let root = self.get_repo_root();
        std::fs::write(root.join(path), content)?;
        Ok(())
    }

    fn get_tracked_files(&self) -> Result<Vec<String>> {
        let index = self.repo.index()?;
        let mut files = Vec::new();
        for i in 0..index.len() {
            if let Some(entry) = index.get(i)
                && let Ok(path_str) = str::from_utf8(&entry.path)
            {
                files.push(path_str.to_string());
            }
        }
        Ok(files)
    }
}
