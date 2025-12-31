use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// A constant string containing the content for the pre-commit hook script.
/// This script is executed before a commit is finalized. It runs the
/// `git-selective-ignore pre-commit` command, which cleans staged files.
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

/// A constant string containing the content for the pre-commit hook script.
/// This script is executed before a commit is finalized. It runs the
/// `git-selective-ignore pre-commit` command, which cleans staged files.
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

const STRICT_PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# Git Selective Ignore - Strict Pre-commit Hook

# Check if git-selective-ignore is available
if ! command -v git-selective-ignore > /dev/null 2>&1; then
    echo "Warning: git-selective-ignore not found in PATH"
    exit 0
fi

# Verify no ignored content is staged. Fail commit if found.
git-selective-ignore verify
if [ $? -ne 0 ]; then
    echo "Error: Ignored content detected in staging area. Commit aborted."
    exit 1
fi
"#;

/// A constant string containing the content for the post-merge hook script.
/// This is a placeholder for a future feature to handle merge conflicts
/// and pattern updates. It is currently not used.
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

/// `install_git_hooks` is the main public function for setting up the Git hooks.
///
/// It takes the repository root path and installs the `pre-commit` and `post-commit`
/// hooks in the `.git/hooks` directory. It also handles backing up any pre-existing
/// hooks to prevent data loss.
///
/// # Arguments
/// * `repo_root`: The `Path` to the root directory of the Git repository.
/// * `strict`: If true, installs the strict pre-commit hook that fails if ignored content is found.
pub fn install_git_hooks(repo_root: &Path, strict: bool) -> Result<()> {
    // Construct the path to the Git hooks directory.
    let hooks_dir = repo_root.join(".git").join("hooks");

    // Ensure the hooks directory exists before attempting to install hooks.
    // Being extra careful, hooks folder is part `git init`
    if !hooks_dir.exists() {
        fs::create_dir(&hooks_dir).context("Failed to create .git/hooks directory")?;
    }

    fs::create_dir_all(&hooks_dir)?;

    // Install the pre-commit, post-commit, post-merge and pre-push hooks.
    if strict {
        install_hook(&hooks_dir, "pre-commit", STRICT_PRE_COMMIT_HOOK)?;
    } else {
        install_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK)?;
    }
    install_hook(&hooks_dir, "post-commit", POST_COMMIT_HOOK)?;
    install_hook(&hooks_dir, "post-merge", POST_MERGE_HOOK)?;
    install_hook(&hooks_dir, "pre-push", PRE_PUSH_HOOK)?;

    Ok(())
}

/// `uninstall_git_hooks` is the main public function for removing the Git hooks.
///
/// It checks for the presence of our `pre-commit` and `post-commit` hooks and removes
/// them. If a backup of an original hook exists, it restores it.
///
/// # Arguments
/// * `repo_root`: The `Path` to the root directory of the Git repository.
pub fn uninstall_git_hooks(repo_root: &Path) -> Result<()> {
    // Construct the path to the Git hooks directory.
    let hooks_dir = repo_root.join(".git").join("hooks");

    // Ensure the hooks directory exists before attempting to install hooks.
    // Being extra careful, hooks folder is part `git init`
    if !hooks_dir.exists() {
        fs::create_dir(&hooks_dir).context("Failed to create .git/hooks directory")?;
    }

    // Install the pre-commit, post-commit, post-merge and pre-push hooks.
    uninstall_hook(&hooks_dir, "pre-commit")?;
    uninstall_hook(&hooks_dir, "post-commit")?;
    uninstall_hook(&hooks_dir, "post-merge")?;
    uninstall_hook(&hooks_dir, "pre-push")?;

    Ok(())
}

/// A private helper function to install a single hook file.
///
/// It first checks if a hook with the same name already exists. If it does
/// and it's not our hook, it renames the existing hook to a `.backup` file
/// before writing the new hook.
///
/// # Arguments
/// * `hooks_dir`: The `Path` to the `.git/hooks` directory.
/// * `hook_name`: The name of the hook file (e.g., "pre-commit").
/// * `hook_content`: The content of the hook script to be written.
fn install_hook(hooks_dir: &Path, hook_name: &str, hook_content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(hook_name);

    // Check if a hook with this name already exists.
    if hook_path.exists() {
        // Check if it's already our hook
        let existing_content = fs::read_to_string(&hook_path)?;
        if existing_content.contains("Git Selective Ignore") {
            println!("ℹ️  {hook_name} hook already installed");
            return Ok(());
        }

        // If an existing hook is not ours, back it up.
        let backup_path = hooks_dir.join(format!("{hook_name}.backup"));
        fs::rename(&hook_path, backup_path)?;
        println!("ℹ️  Backed up existing {hook_name} hook");
    }

    // Write the new hook script to the hooks directory.
    fs::write(&hook_path, hook_content)?;

    // Make the hook executable on Unix-like operating systems.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        // Set the executable bit for the file owner, group, and others.
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

/// A private helper function to uninstall a single hook file.
///
/// It checks for our hook content signature. If it finds it, it removes the file.
/// If a backup of an original hook exists, it is restored.
///
/// # Arguments
/// * `hooks_dir`: The `Path` to the `.git/hooks` directory.
/// * `hook_name`: The name of the hook file to uninstall.
fn uninstall_hook(hooks_dir: &Path, hook_name: &str) -> Result<()> {
    let hook_path = hooks_dir.join(hook_name);
    let backup_path = hooks_dir.join(format!("{hook_name}.backup"));

    // Check if the hook file exists.
    if hook_path.exists() {
        // Read the hook's content to verify it's one of ours before removing.
        let content = fs::read_to_string(&hook_path)?;
        if content.contains("Git Selective Ignore") {
            fs::remove_file(&hook_path)?;
            println!("✓ Removed {hook_name} hook");

            // If a backup of an original hook exists, restore it by renaming it.
            if backup_path.exists() {
                fs::rename(&backup_path, &hook_path)?;
                println!("✓ Restored original {hook_name} hook");
            }
        }
    }

    Ok(())
}
