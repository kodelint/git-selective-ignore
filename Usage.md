## Usage Guide

This document provides a detailed walkthrough of how to use `git-selective-ignore`.

## Table of Contents

1.  [Create a repository](#1-create-a-repository)
2.  [Create a sample code file](#2-create-a-sample-code-file)
3.  [Initialize and Configure](#3-initialize-and-configure)
4.  [Add Ignore Patterns](#4-add-ignore-patterns)
5.  [Verify Configuration](#5-verify-configuration)
6.  [Check Status](#6-check-status)
7.  [Commit and Verify](#7-commit-and-verify)
8.  [Advanced Usage](#8-advanced-usage)

---

### 1. Create a repository

```bash
mkdir testing-git-selective-ignore
cd testing-git-selective-ignore
git init
```

### 2. Create a sample code file

Create a file with content that should not be committed to Git history (e.g., secrets, debug code).

```rust
// src/main.rs
fn main() {
    println!("Starting application...");

    // DEBUG BLOCK START
    println!("Debug: Application started in debug mode");
    // DEBUG BLOCK END

    let API_KEY = "sk_live_1234567890abcdef";
    println!("Using API key: {}", API_KEY);

    /* Imagine the below lines are temporary */
    let temp_feature = "experimental_feature_xyz";
    println!("Testing temporary feature: {}", temp_feature);

    let SECRET = "Some Dumb key";
    println!("SECRET Removed");
}
```

### 3. Initialize and Configure

Initialize the tool in your repository. This creates the `.git/selective-ignore.toml` config file.

```bash
git-selective-ignore init
```

Install the Git hooks to enable automatic processing.

```bash
git-selective-ignore install-hooks
```

### 4. Add Ignore Patterns

Add patterns to tell the tool what to ignore. You can use the `add` command directly or use the interactive wizard.

**Using the Wizard:**
```bash
git-selective-ignore add-wizard
```

**Manual Addition:**



- **Using** `line-regex` **(default):**

  - Ignore lines that match a specific regular expression.

    ```bash

    # Ignore all lines in `src/main.rs` containing the word `println`

    git-selective-ignore add src/main.rs ".*println.*" --pattern-type line-regex

    ```

- **Using** `line-number`:

  - Ignore a specific line number.

    ```bash

    # Ignore line 15 in `src/config.rs`

    git-selective-ignore add src/config.rs 15 --pattern-type line-number

    ```

- **Using** `block-start-end`:

  - Ignore a block of code defined by a start and end regex.

    The `|||` separator is used to delimit the start and end patterns.

    ```bash

    # Ignore a debug block in `src/lib.rs`

    git-selective-ignore add src/lib.rs "//# DEBUG START ||| //# DEBUG END" --pattern-type block-start-end

    ```



### 5. Dry Run Mode



If you want to see what would happen during a commit without actually modifying any files, use the `--dry-run` flag with `pre-commit` or `post-commit`.

```bash
git-selective-ignore pre-commit --dry-run
```

### 6. Strict Verification

For extra safety, you can install the "Strict" version of the pre-commit hook. This hook will fail the commit if it detects any ignored content in the staging area, instead of automatically cleaning it.

```bash
git-selective-ignore install-hooks --strict
```

### 7. Global Configuration

Patterns that you want to apply to *all* your repositories can be added to the global configuration file at `~/.config/git-selective-ignore/config.toml`. The tool will automatically merge these patterns with your local repository settings.

### 8. Verify Configuration

List the installed patterns to verify they are correct.

```bash
git-selective-ignore list
```

### 6. Check Status

Run `status` to see which lines will be ignored before you commit.

```bash
git-selective-ignore status
```

Output should indicate that your patterns are matching the lines in `src/main.rs`.

### 7. Commit and Verify

Stage your files:
```bash
git add .
```

Commit your changes. The `pre-commit` hook will automatically remove the ignored lines, and the `post-commit` hook will restore them to your working directory.

```bash
git commit -m "Initial commit with selective ignore"
```

**Verify Git History:**
Check the commit content. The ignored lines (API key, debug block, etc.) should be missing.
```bash
git show HEAD
```

**Verify Working Directory:**
Check your local file. The ignored lines should still be there!
```bash
cat src/main.rs
```

### 8. Advanced Usage

#### Funny Mode
Enable "Funny Mode" for entertaining commit messages!
Edit `.git/selective-ignore.toml`:
```toml
[global_settings]
funny_mode = true
```

#### Importing from .gitignore
You can import patterns from an existing `.gitignore` file.
```bash
git-selective-ignore import .gitignore --import-type gitignore
```

#### Exporting Configuration
Share your patterns with your team by exporting them.
```bash
git-selective-ignore export my-config.toml --format toml
```