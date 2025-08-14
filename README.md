# git-selective-ignore

#### A Git plugin to selectively ignore lines and code blocks during commits.

<p align="center">
  <img src="https://github.com/kodelint/blog-images/blob/main/common/01-git-selecting-ignore.png" alt="git-selective-ignore" width="500"/>
</p>

`git-selective-ignore` is a powerful Git extension that allows you to manage which parts of a file are committed without
modifying the file itself. Unlike `.gitignore`, which ignores entire files, this tool enables you to ignore specific lines,
regular expressions, or entire blocks of code. It's ideal for keeping local configuration changes, debug code,
or sensitive data out of your commits.

The tool works by temporarily stripping out the ignored content from staged files before a commit and then restoring
the files to their original state after the commit is complete. This process is managed automatically via Git hooks,
ensuring a seamless workflow.
---
## Features
- **Pattern-Based Ignoring:** Define ignore patterns using regular expressions, line numbers, or start/end markers for code blocks.
- **Automatic Processing:** Installs pre-commit and post-commit Git hooks to automatically process and restore files.
- **Flexible Configuration:** Configuration is stored in `.git/selective-ignore`.toml, allowing for granular control over patterns and global settings.
- **Import Existing Patterns:** Import patterns from existing `.gitignore` files to easily transition your setup.
- **Backup Strategies:** Choose between in-memory or temporary file backups to suit your needs.

---

## Installation
To install `git-selective-ignore`, you'll first need to compile the tool.

1. Clone the repository:
    ```bash
    git clone https://github.com/kodelint/git-selective-ignore.git
    cd git-selective-ignore
    ```
2. Build the project using Cargo:
    ```bash
    cargo install --path .
    ```

This command will build the project and install the executable `git-selective-ignore` in your Cargo binary directory,
making it available in your shell's path.

---

## Usages
Once installed, you can use `git-selective-ignore` within any Git repository.

#### 1. Initialize the Repository
Navigate to your Git repository and run the `init` command. This will create the configuration file `.git/selective-ignore.toml`
and prompt you to install the Git hooks.
```bash
git-selective-ignore init
```
#### 2. Install Git Hooks
If you didn't install the hooks during initialization, you can do so manually. This is a crucial step to enable the
automatic processing of your files.
```bash
git-selective-ignore install-hooks
```
To stop the automatic processing, you can uninstall the hooks.
```bash
git-selective-ignore uninstall-hooks
```
#### 3. Add an Ignore Pattern
You can add patterns using the `add` command. The tool supports multiple pattern types.
- **Using** `line-regex` **(default):** Ignore lines that match a specific regular expression.
    ```bash
    # Ignore all lines in `src/main.rs` containing the word `println`
    git-selective-ignore add src/main.rs ".*println.*" --pattern-type line-regex
    ```
- **Using** `line-number`: Ignore a specific line number.
    ```bash
    # Ignore line 15 in `src/config.rs`
    git-selective-ignore add src/config.rs 15 --pattern-type line-number
    ```
- **Using** `block-start-end`: Ignore a block of code defined by a start and end regex.
  The `|||` separator is used to delimit the start and end patterns.
    ```bash
    # Ignore a debug block in `src/lib.rs`
    git-selective-ignore add src/lib.rs "//# DEBUG START ||| //# DEBUG END" --pattern-type block-start-end
    ```
#### 4. List Patterns
To see all the patterns configured for the current repository, use the `list` command.
```bash
git-selective-ignore list
```

#### 5. Check Status
Use the `status` command to see which files have ignored content and how many lines would be removed in a commit.
```bash
git-selective-ignore status
```
This command provides a summary of the ignored lines in your project.

---
## Configuration
The tool's behavior is controlled by the `.git/selective-ignore.toml` file.

```toml
version = "1.0"

[global_settings]
backup_strategy = "TempFile"
auto_cleanup = true
verbose = false

[files."src/main.rs"]
[[files."src/main.rs".patterns]]
id = "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d"
pattern_type = "line-regex"
specification = "println!.*"
```
You can manually edit this file to configure your patterns and global settings.

---
## Contribution
Contributions! are welcome, feel free to open an issue or submit a pull request on GitHub.

- **Issues:** Report bugs or suggest new features.
- **Pull Requests:** Fork the repository and submit your changes.

