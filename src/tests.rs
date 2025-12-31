#[cfg(test)]
mod tests {
    use crate::builders::patterns::IgnorePattern;
    use crate::core::config::{ConfigManager, SelectiveIgnoreConfig};
    use crate::core::engine::IgnoreEngine;
    use git2::Repository;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_test_repo() -> (tempfile::TempDir, Repository, PathBuf) {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let repo_path = dir.path().to_path_buf();
        (dir, repo, repo_path)
    }

    #[test]
    fn test_initialization() {
        let (_dir, _repo, repo_path) = setup_test_repo();
        std::env::set_current_dir(&repo_path).unwrap();

        let config_manager = ConfigManager::new().unwrap();
        config_manager.initialize().unwrap();

        let config_file = repo_path.join(".git").join("selective-ignore.toml");
        assert!(config_file.exists());
    }

    #[test]
    fn test_dry_run_pre_commit() {
        let (_dir, repo, repo_path) = setup_test_repo();
        std::env::set_current_dir(&repo_path).unwrap();

        let test_file = "test.txt";
        let file_path = repo_path.join(test_file);
        fs::write(&file_path, "line1\nIGNORE ME\nline3\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(test_file)).unwrap();
        index.write().unwrap();

        let mut config_manager = ConfigManager::new().unwrap();
        config_manager.initialize().unwrap();
        config_manager
            .add_pattern(
                test_file.to_string(),
                "line-regex".to_string(),
                "/IGNORE ME/".to_string(),
            )
            .unwrap();

        let mut engine = IgnoreEngine::new(config_manager).unwrap();

        // Dry run
        engine.process_pre_commit(true).unwrap();

        // Verify file content hasn't changed
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nIGNORE ME\nline3\n");
    }

    #[test]
    fn test_actual_pre_commit() {
        let (_dir, repo, repo_path) = setup_test_repo();
        std::env::set_current_dir(&repo_path).unwrap();

        let test_file = "test.txt";
        let file_path = repo_path.join(test_file);
        fs::write(&file_path, "line1\nIGNORE ME\nline3\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(test_file)).unwrap();
        index.write().unwrap();

        let mut config_manager = ConfigManager::new().unwrap();
        config_manager.initialize().unwrap();
        config_manager
            .add_pattern(
                test_file.to_string(),
                "line-regex".to_string(),
                "/IGNORE ME/".to_string(),
            )
            .unwrap();

        let mut engine = IgnoreEngine::new(config_manager).unwrap();

        // Actual run
        engine.process_pre_commit(false).unwrap();

        // Verify file content HAS changed in working directory
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nline3\n");
    }

    #[test]
    fn test_global_config_merge() {
        let (_dir, repo, repo_path) = setup_test_repo();
        std::env::set_current_dir(&repo_path).unwrap();

        // 1. Create a dummy global config
        let global_config_dir = repo_path.join("global_config");
        fs::create_dir_all(&global_config_dir).unwrap();
        let global_config_path = global_config_dir.join("config.toml");

        let mut global_config = SelectiveIgnoreConfig::default();
        global_config.files.insert(
            "all".to_string(),
            vec![
                IgnorePattern::new("line-regex".to_string(), "/GLOBAL/".to_string()).unwrap(),
            ],
        );

        let content = toml::to_string_pretty(&global_config).unwrap();
        fs::write(&global_config_path, content).unwrap();

        // 2. Setup local repo with a file and local pattern
        let test_file = "test.txt";
        let file_path = repo_path.join(test_file);
        fs::write(&file_path, "line1\nGLOBAL\nLOCAL\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(test_file)).unwrap();
        index.write().unwrap();

        let mut config_manager = ConfigManager::new().unwrap();
        config_manager.set_global_config_path(global_config_path);
        config_manager.initialize().unwrap();
        config_manager
            .add_pattern(
                test_file.to_string(),
                "line-regex".to_string(),
                "/LOCAL/".to_string(),
            )
            .unwrap();

        // 3. Run engine and verify both global and local patterns applied
        let mut engine = IgnoreEngine::new(config_manager).unwrap();
        engine.process_pre_commit(false).unwrap();

        let final_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(final_content, "line1\n");
    }
}
