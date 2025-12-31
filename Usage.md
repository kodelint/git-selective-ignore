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

Add patterns to tell the tool what to ignore.

**Ignore by Regex (Global):**
Ignore lines containing `API_KEY` in all files.
```bash
git-selective-ignore add all API_KEY --pattern-type line-regex
```

**Ignore by Block (Global):**
Ignore blocks starting with `// DEBUG BLOCK START` and ending with `// DEBUG BLOCK END`.
```bash
git-selective-ignore add all "// DEBUG BLOCK START ||| // DEBUG BLOCK END" --pattern-type block-start-end
```

**Ignore by Line Range (Specific File):**
Ignore lines 13-14 in `src/main.rs`.
```bash
git-selective-ignore add src/main.rs 13-14 --pattern-type line-range
```

### 5. Verify Configuration

List the installed patterns to verify they are correct.

```bash
git-selective-ignore list
```

#### 6. Check Status

Run `status` to see which lines will be ignored before you commit.

```bash
git-selective-ignore status
```

Output should indicate that your patterns are matching the lines in `src/main.rs`.

---

### 8. Advanced Usage

#### Dry Run Mode
You can simulate the pre-commit process without actually modifying any files in your working directory. This is useful for verifying exactly what *would* happen during a commit.

```bash
git-selective-ignore pre-commit --dry-run
```

#### Interactive Pattern Wizard
If you run the `add` command without arguments, the tool will guide you through the process of creating a new pattern.

```bash
git-selective-ignore add
```

#### Global Configuration
You can define patterns that apply to **all** your Git repositories. Create a file at `~/.git-selective-ignore.toml` (or `%USERPROFILE%\.git-selective-ignore.toml` on Windows).

Local repository configurations in `.git/selective-ignore.toml` will be merged with this global configuration, with local settings taking precedence.

#### Strict Verification Hook
When you run `install-hooks`, the tool also installs a `pre-push` hook. This hook runs `git-selective-ignore verify`, which strictly prevents you from pushing if any ignored patterns are detected in your staged files. This acts as a final safety net.

#### Funny Mode
Enable "Funny Mode" for entertaining commit messages!
Edit `.git/selective-ignore.toml`:
```toml
[global_settings]
funny_mode = true
```