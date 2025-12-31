use git2::Repository;
use git_selective_ignore::core::config::ConfigManager;
use git_selective_ignore::core::engine::IgnoreEngine;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn setup_test_repo() -> (TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    (dir, repo)
}

#[test]
fn test_core_workflow() {
    let (td, repo) = setup_test_repo();
    let repo_root = td.path().to_path_buf();

    // 1. Setup Config
    let mut config_manager = ConfigManager::new_at(repo_root.clone()).unwrap();
    config_manager.initialize().unwrap();
    
    // Add a pattern
    config_manager.add_pattern(
        "test.txt".to_string(), 
        "line-regex".to_string(), 
        "/SECRET/".to_string()
    ).unwrap();

    // 2. Create a file with "secret" content
    let test_file_path = repo_root.join("test.txt");
    fs::write(&test_file_path, "line 1\nSECRET_KEY = \"12345\"\nline 3\n").unwrap();

    // 3. Stage the file
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("test.txt")).unwrap();
    index.write().unwrap();

    // 4. Run Pre-Commit
    let mut engine = IgnoreEngine::new(config_manager).unwrap();
    engine.process_pre_commit(false).unwrap();

    // 5. Verify staged content is cleaned
    // Refresh the repository to ensure we have the latest state
    let mut index = repo.index().unwrap();
    index.read(false).unwrap(); // Force reload from disk
    let entry = index.get_path(Path::new("test.txt"), 0).expect("File should be in index");
    let blob = repo.find_blob(entry.id).unwrap();
    let staged_content = std::str::from_utf8(blob.content()).unwrap();
    
    assert!(!staged_content.contains("SECRET_KEY"), "Staged content should not contain secret");
    assert!(staged_content.contains("line 1"), "Staged content should contain line 1");
    assert!(staged_content.contains("line 3"), "Staged content should contain line 3");

    // 6. Run Post-Commit
    engine.process_post_commit().unwrap();

    // 7. Verify working directory content is restored
    let working_content = fs::read_to_string(&test_file_path).unwrap();
    assert!(working_content.contains("SECRET_KEY"), "Working directory content should be restored");
}

#[test]
fn test_verify_command() {
    let (td, repo) = setup_test_repo();
    let repo_root = td.path().to_path_buf();

    let mut config_manager = ConfigManager::new_at(repo_root.clone()).unwrap();
    config_manager.initialize().unwrap();
    config_manager.add_pattern(
        "test.txt".to_string(), 
        "line-regex".to_string(), 
        "/FORBIDDEN/".to_string()
    ).unwrap();

    let test_file_path = repo_root.join("test.txt");
    fs::write(&test_file_path, "FORBIDDEN CONTENT\n").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("test.txt")).unwrap();
    index.write().unwrap();

    let mut engine = IgnoreEngine::new(config_manager).unwrap();
    
    // Verify should fail because forbidden content is staged
    let result = engine.verify_staging();
    assert!(result.is_err(), "Verify should fail when forbidden content is staged");
}
