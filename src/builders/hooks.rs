use anyhow::Result;
use std::fs;
use std::path::Path;

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# Git Selective Ignore - Pre-commit Hook

# Check if git-selective-ignore is available
if ! command -v git-selective-ignore > /dev/null 2>&1; then
    echo "Warning: git-selective-ignore not found in PATH"
    exit 0
fi

# Process files before commit
git-selective-ignore pre-commit
if [ $? -ne 0 ]; then
    echo "Error: Failed to process selective ignore patterns"
    exit 1
fi
"#;

const POST_COMMIT_HOOK: &str = r#"#!/bin/sh
# Git Selective Ignore - Post-commit Hook

# Check if git-selective-ignore is available
if ! command -v git-selective-ignore > /dev/null 2>&1; then
    echo "Warning: git-selective-ignore not found in PATH"
    exit 0
fi

# Restore files after commit
git-selective-ignore post-commit
"#;

const POST_MERGE_HOOK: &str = r#"#!/bin/sh
# Git Selective Ignore - Post-merge Hook

# Check if git-selective-ignore is available
if ! command -v git-selective-ignore > /dev/null 2>&1; then
    echo "Warning: git-selective-ignore not found in PATH"
    exit 0
fi

# Restore files after merge
git-selective-ignore post-commit
"#;

const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
# Git Selective Ignore - Pre-push Hook

# Check if git-selective-ignore is available
if ! command -v git-selective-ignore > /dev/null 2>&1; then
    echo "Warning: git-selective-ignore not found in PATH"
    exit 0
fi

# Verify no ignored content is staged before pushing
git-selective-ignore verify
"#;
pub fn install_git_hooks(repo_root: &Path) -> Result<()> {
    let hooks_dir = repo_root.join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    install_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK)?;
    install_hook(&hooks_dir, "post-commit", POST_COMMIT_HOOK)?;
    install_hook(&hooks_dir, "post-merge", POST_MERGE_HOOK)?;
    install_hook(&hooks_dir, "pre-push", PRE_PUSH_HOOK)?;

    Ok(())
}

fn install_hook(hooks_dir: &Path, hook_name: &str, hook_content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(hook_name);

    if hook_path.exists() {
        // Check if it's already our hook
        let existing_content = fs::read_to_string(&hook_path)?;
        if existing_content.contains("Git Selective Ignore") {
            println!("ℹ️  {hook_name} hook already installed");
            return Ok(());
        }

        // Backup existing hook
        let backup_path = hooks_dir.join(format!("{hook_name}.backup"));
        fs::rename(&hook_path, backup_path)?;
        println!("ℹ️  Backed up existing {hook_name} hook");
    }

    fs::write(&hook_path, hook_content)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}