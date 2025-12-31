# git-selective-ignore

![Rust Logo](https://img.shields.io/badge/Rust-red?style=for-the-badge&logo=rust)
![Platform](https://img.shields.io/badge/Platform-macOS-blue?style=for-the-badge&logo=apple)
![Platform](https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge&logo=linux)
![Platform](https://img.shields.io/badge/Platform-Windows-blue?style=for-the-badge&logo=windows)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/kodelint/git-selective-ignore/actions/workflows/workflow.yml/badge.svg)](https://github.com/kodelint/git-selective-ignore/actions/workflows/workflow.yml)
[![GitHub release](https://img.shields.io/github/release/kodelint/git-selective-ignore.svg)](https://github.com/kodelint/git-selective-ignore/releases)
[![GitHub stars](https://img.shields.io/github/stars/kodelint/git-selective-ignore.svg)](https://github.com/kodelint/git-selective-ignore/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/kodelint/git-selective-ignore.svg)](https://github.com/kodelint/git-selective-ignore/commits/main)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/kodelint/git-selective-ignore/pulls)
[![git-cliff](https://img.shields.io/badge/%F0%9F%9A%80-git--cliff-ff69b4.svg)](https://github.com/orhun/git-cliff)

<p align="center">
  <img src="https://github.com/kodelint/blog-images/blob/main/common/01-git-selecting-ignore.png" alt="git-selective-ignore" width="500"/>
</p>

#### A Git plugin to selectively ignore lines and code blocks during commits.

`git-selective-ignore` is a Git extension that lets you control which parts of a file get committed without modifying the
file itself. Unlike `.gitignore`, which excludes whole files, this tool allows you to ignore specific lines, regex patterns,
or code blocks. It’s especially useful for local configs, debug statements, or sensitive values that you don’t want in your Git history.

It works by stripping ignored content from staged files before commit, then restoring files afterward. Git hooks handle
this automatically, so the workflow stays seamless.

---

## Features

- **Pattern-Based Ignoring:** Ignore by regex, line numbers, or block start/end markers.
- **Automatic Processing:** Installs `pre-commit` and `post-commit` hooks to strip and restore files.
- **Flexible Configuration:** Stores settings in `.git/selective-ignore.toml` with per-file and global rules.
- **Global Configuration:** Support for `~/.git-selective-ignore.toml` to apply patterns across all your projects.
- **Interactive Wizard:** Easily add patterns via a guided CLI wizard.
- **Dry Run Mode:** Simulate the cleaning process before committing.
- **Import Existing Patterns:** Import patterns from existing `.gitignore` files to easily transition your setup.
- **Backup Strategies:** Choose between in-memory or temporary file backups to suit your needs.
- **Funny Mode:** Add a touch of humor to your development workflow with entertaining messages.

---

## Installation

You can install [git-selective-ignore](https://github.com/kodelint/git-selective-ignore) in several ways:

1. **Using [setup-devbox](https://github.com/kodelint/setup-devbox)**: Add below tool details to `tools.yaml`:
   ```yaml
   - name: git-selective-ignore
     version: 0.1.0
     source: github
     repo: kodelint/git-selective-ignore
     tag: v0.1.0
   ```
2. **From [Release Page](https://github.com/kodelint/git-selective-ignore/releases)**, download the latest binary

3. **Build from source:**
   ```bash
   git clone https://github.com/kodelint/git-selective-ignore.git
   cd git-selective-ignore
   cargo install --path .
   ```
   The binary is installed into Cargo’s bin directory, so make sure it’s in your shell `PATH`.

---

## Usages

Once installed, you can use `git-selective-ignore` inside any repository.

#### 1. Initialize the Repository

Navigate to your Git repository and run the `init` command. This will create the configuration file `.git/selective-ignore.toml`.

```bash
git-selective-ignore init
```

#### 2. Install or uninstall hooks

If you didn't install the hooks during initialization, you can do so manually. This is a crucial step to enable the
automatic processing of your files.

```bash
git-selective-ignore install-hooks
git-selective-ignore uninstall-hooks
```

#### 3. Add an Ignore Pattern

You can add patterns using the `add` command. The tool supports multiple pattern types. `add` command works for single specified file or `all` files. default is `all`

- **Using** `line-regex` **(default):**
  - Ignore lines that match a specific regular expression.
    ```bash
    # Ignore all lines in `src/main.rs` containing the word `println`
    git-selective-ignore add src/main.rs ".*println.*" --pattern-type line-regex
    ```
  - Another example: Ignore line with `API_KEY` hardcoded from all staged files
    ```bash
    # Ignore all lines in all files which containing the word `API_KEYS`
    git-selective-ignore add all API_KEYS --pattern-type line-regex
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
  - If you or your team follow a specific pattern
    ```bash
    # Ignore a debug block in `src/lib.rs`
    git-selective-ignore add all "//# DEBUG START ||| //# DEBUG END" --pattern-type block-start-end
    ```

#### 4. List Patterns

To see all the patterns configured for the current repository, use the `list` command.

```bash
git-selective-ignore list
```

#### 5. Check Status

Use the `status` command to see which files have ignored content and how many lines would be removed in a commit. However, keep in mind that `status` can be
an expensive command depending on the size of the repository.
Presently, it looks at all files for the ignore pattern under `all` section.

```bash
git-selective-ignore status
```

This command provides a summary of the ignored lines in your project.

---

#### Documented [Example](./Usage.md)

---

## Configuration

All settings live in `.git/selective-ignore.toml` file.

```toml
version = "1.0"

[global_settings]
backup_strategy = "TempFile"
auto_cleanup = true
verbose = false
funny_mode = false # Enable for humorous output messages

[[files.all]]
id = "78ed02f4-db7c-4921-b565-5e8986f19705"
pattern_type = "LineRegex"
specification = "API_KEY"
compiled_regex = "API_KEY"

[[files."src/main.rs"]]
id = "31ca2ff0-90d8-47ea-90db-413cedf09bcf"
pattern_type = "LineRange"
specification = "13-16"
```

You can manually edit this file to configure your patterns and global settings.

---

## Contribution

Contributions! are welcome, feel free to open an issue or submit a pull request on GitHub.

- **Issues:** Report bugs or suggest new features.

- **Pull Requests:** Fork the repository and submit your changes.

- **Changelog:** Use `make changelog` to generate the `CHANGELOG.md` file using `git-cliff`.
