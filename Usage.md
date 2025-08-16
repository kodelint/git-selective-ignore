## Example

## Table of Contents

1. Create a repository
2. Create a sample code file with content-not-needed-in-git-history
3. Check created files
4. Add **Ignore Patterns** for the repo
5. Check `list` of Ignore Pattern installed
6. Check `status` for violations
7. Stage files
8. Commit files
9. Verify Git history
10. Workspace after commit
11. Add more code to `src/lib.rs`
12. Run `status` again
13. Commit changes
14. Check committed changes to `git`
15. Workspace after commit

---

1.  Create a repository:

    ```bash
    cd testing-git-selective-ignore
    git init
    ```

2.  Create a sample code file with **content-not-needed-in-git-history**:

    ```rust
    use std::env;

    fn main() {
        println!("Starting application...");

        // DEBUG BLOCK START
        println!("Debug: Application started in debug mode");
        // DEBUG BLOCK END

        let API_KEY = "sk_live_1234567890abcdef";
        println!("Using API key: {}", API_KEY);

        /* Below lines are temporary and line numbers are 13-16 */
        let temp_feature = "experimental_feature_xyz";
        println!("Testing temporary feature: {}", temp_feature);
        /* Remember to remove lines from 13-16 */

        let SECRET = "Some Dumb key";
        println!("SECRET Removed");
        let GITHUB_TOKEN = "Another Dumb Key";

        println!("Application completed successfully");
    }

    fn process_data() -> i32 {
        42
    }
    ```

3.  Check created files:
    ```bash
    tree
    .
    └── src
        ├── lib.rs
        └── main.rs
    ```
4.  Add **Ignore Patterns** for the repo:

    ```bash
    >> git-selective-ignore init
    ✓ Initialized selective ignore for this repository
    Run 'git-selective-ignore install-hooks' to enable automatic processing

    >> git-selective-ignore install-hooks
    ✓ Installed Git hooks for automatic processing

    >> git-selective-ignore add all --pattern-type line-regex API_KEY
    ✓ Configuration is valid.
    ✓ Added ignore pattern

    >> git-selective-ignore add all --pattern-type line-regex APP_KEY
    ✓ Configuration is valid.
    ✓ Added ignore pattern

    >> git-selective-ignore add all --pattern-type line-regex GITHUB_TOKEN
    ✓ Configuration is valid.
    ✓ Added ignore pattern
    >> git-selective-ignore add all --pattern-type block-start-end "// DEBUG BLOCK START ||| // DEBUG BLOCK END"
    ✓ Configuration is valid.
    ✓ Added ignore pattern

    >> git-selective-ignore add all --pattern-type line-regex SECRET
    ✓ Configuration is valid.
    ✓ Added ignore pattern
    >> git-selective-ignore add all --pattern-type line-regex password
    ✓ Configuration is valid.
    ✓ Added ignore pattern

    >> git-selective-ignore add src/main.rs --pattern-type line-range 13-16
    ✓ Configuration is valid.
    ✓ Added ignore pattern

    >> git-selective-ignore add src/main.rs --pattern-type block-start-end "/* TEMP_CODE_START */ ||| /* TEMP_CODE_END */"
    ✓ Configuration is valid.
    ✓ Added ignore pattern
    ```

5.  Let's check `list` of Ignore Pattern installed:

    ```bash
    >> git-selective-ignore list
    ✓ Configuration is valid.

    📁 File: src/main.rs
      🔍 ID: 31ca2ff0-90d8-47ea-90db-413cedf09bcf | Type: LineRange | Pattern: 13-16
      🔍 ID: a941d428-87ed-4378-898d-d5156723dfd0 | Type: BlockStartEnd | Pattern: /* TEMP_CODE_START */ ||| /* TEMP_CODE_END */

    📁 File: all
      🔍 ID: 78ed02f4-db7c-4921-b565-5e8986f19705 | Type: LineRegex | Pattern: API_KEY
      🔍 ID: 7fb165d1-bab6-4c79-a13b-51f2f29a88e9 | Type: LineRegex | Pattern: APP_KEY
      🔍 ID: 02b17597-bb85-428c-be56-3d0cd4a3c44b | Type: LineRegex | Pattern: GITHUB_TOKEN
      🔍 ID: 76447f06-dd03-4c3b-b27a-b611579e9cb8 | Type: BlockStartEnd | Pattern: // DEBUG BLOCK START ||| // DEBUG BLOCK END
      🔍 ID: 48f984d1-dd90-4984-99d6-ae6c63c591d6 | Type: LineRegex | Pattern: SECRET
      🔍 ID: b9a54bc2-048d-4fa0-b6ff-dc66aff6e706 | Type: LineRegex | Pattern: password
    ```

6.  Check `status` for violations:

    ```bash
    >> git-selective-ignore status
    ✓ Configuration is valid.
       ├─ Line Range Pattern '13-16': 4 line(s) matched
       │  └─ Lines 13-16
       ├─ Regex Pattern 'API_KEY': 1 line(s) matched
       │  └─ Line 10
       ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
       │  └─ Line 20
       ├─ Block Pattern '// DEBUG BLOCK START ||| // DEBUG BLOCK END': 3 line(s) matched
       │  └─ Lines 6-8
       ├─ Regex Pattern 'SECRET': 1 line(s) matched
       │  └─ Line 18
       └─ Summary: 10 line(s) ignored, 17 line(s) remaining (of 27 total)
       ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
       │  └─ Line 7
       └─ Summary: 1 line(s) ignored, 18 line(s) remaining (of 19 total)
    📊 Git Selective Ignore Status Report
    =====================================
    🎯 Specifically Configured Files:
    🟡 src/main.rs (8 patterns, 10/27 lines ignored, 37.0%)

    🌐 Files Affected by Global 'ALL' Patterns:
    🟡 src/lib.rs (6 patterns, 1/19 lines ignored, 5.3%)

    📈 Summary:
      Total files: 2
      Total patterns: 8
      Total ignored lines: 11
      Files with issues: 2

    📋 Breakdown:
      Specifically configured files: 1
      Files affected by 'ALL' patterns only: 1
    ```

    It looked for violations in `src/main.rs`

    ```bash
    🎯 Specifically Configured Files:
    🟡 src/main.rs (8 patterns, 10/27 lines ignored, 37.0%)
    ```

    And also checked global patterns:

    ```bash
    🌐 Files Affected by Global 'ALL' Patterns:
    🟡 src/lib.rs (6 patterns, 1/19 lines ignored, 5.3%)
    ```

7.  Stage files

    ```bash
    >> git add -A
    >> git status
    On branch main

    No commits yet

    Changes to be committed:
      (use "git rm --cached <file>..." to unstage)
            new file:   src/lib.rs
            new file:   src/main.rs
    ```

8.  Commit files:

    ```bash
    >> git commit -m "Committing files with content which are not supposed to be in (GIT HISTORY)"
    ✓ Configuration is valid.
    📝 Processing files with selective ignore patterns...

    📄 Processing file: src/lib.rs
      └─ Found 6 ignore pattern(s) installed
      ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
      │  └─ Line 4
      └─ Summary: 1 line(s) ignored, 6 line(s) remaining (of 7 total)

    📄 Processing file: src/main.rs
      └─ Found 8 ignore pattern(s) installed
      ├─ Regex Pattern 'API_KEY': 1 line(s) matched
      │  └─ Line 10
      ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
      │  └─ Line 20
      ├─ Block Pattern '// DEBUG BLOCK START ||| // DEBUG BLOCK END': 3 line(s) matched
      │  └─ Lines 6-8
      ├─ Regex Pattern 'SECRET': 1 line(s) matched
      │  └─ Line 18
      ├─ Line Range Pattern '13-16': 4 line(s) matched
      │  └─ Lines 13-16
      └─ Summary: 10 line(s) ignored, 17 line(s) remaining (of 27 total)

    🔄 Re-staging modified files...
    ✅ Pre-commit processing complete.
    🔄 Restoring files after commit...
    ✓ Restored src/main.rs
    ✓ Restored src/lib.rs
    ✅ Post-commit processing complete.

    [main (root-commit) 8192612] Committing files with content which are not supposed to be in (GIT HISTORY)

    2 files changed, 20 insertions(+)
    create mode 100644 src/lib.rs
    create mode 100644 src/main.rs
    ```

9.  Verify Git history:

    ```bash
    git show
    commit 8192612da61bf7bfc38012cd67de999d1a06457c (HEAD -> main)
    Author: kodelint <kodelint@gmail.com>
    Date:   Sat Aug 16 10:09:05 2025 -0700

        Committing files with content which are not supposed to be in (GIT HISTORY)

    diff --git a/src/lib.rs b/src/lib.rs
    new file mode 100644
    index 0000000..b392a32
    --- /dev/null
    +++ b/src/lib.rs
    @@ -0,0 +1,5 @@
    +fn main() {
    +    println!("Another Test");
    +
    +    println!("{} <- My GitHub Token", GITHUB_TOKEN);
    +}
    diff --git a/src/main.rs b/src/main.rs
    new file mode 100644
    index 0000000..b8b545e
    --- /dev/null
    +++ b/src/main.rs
    @@ -0,0 +1,15 @@
    +use std::env;
    +
    +fn main() {
    +    println!("Starting application...");
    +
    +    println!("Using API key: {}", API_KEY);
    +
    +    println!("SECRET Removed");
    +
    +    println!("Application completed successfully");
    +}
    +
    +fn process_data() -> i32 {
    +    42
    +}
    ```

10. Workspace after commit:
    Notice `+` lines are not in Git history

    ```bash
    >> cat src/*
          │ File: src/lib.rs
      1   │ fn main() {
      2   │     println!("Another Test");
      3   │
      4 + │     let GITHUB_TOKEN = "github_fake_token_093790841-831-8lncdlwnelkqix12=-1x;xm;m"
      5 + │
      6   │     println!("{} <- My GitHub Token", GITHUB_TOKEN);
      7   │ }

          │ File: src/main.rs
      1   │ use std::env;
      2   │
      3   │ fn main() {
      4   │     println!("Starting application...");
      5   │
      6 + │     // DEBUG BLOCK START
      7 + │     println!("Debug: Application started in debug mode");
      8 + │     // DEBUG BLOCK END
      9 + │
      10 +│     let API_KEY = "sk_live_1234567890abcdef";
      11  │     println!("Using API key: {}", API_KEY);
      12  │
      13 +│     /* Imagine the below lines are temporary and line numbers are 13-16 */
      14 +│     let temp_feature = "experimental_feature_xyz";
      15 +│     println!("Testing temporary feature: {}", temp_feature);
      16 +│     /* Remember to remove lines from 10-14 */
      17 +│
      18 +│     let SECRET = "Some Dumb key";
      19  │     println!("SECRET Removed");
      20 +│     let GITHUB_TOKEN = "Another Dumb Key";
      21  │
      22  │     println!("Application completed successfully");
      23  │ }
      24  │
      25  │ fn process_data() -> i32 {
      26  │     42
      27  │ }
    ```

11. Add more code to `src/lib.rs`:

    ```rust
    let API_KEY = env::var('API_KEY');

    match env::var('API_KEY') {
        Ok(value) => {
            println!("The value of APP_KEY is: {}", API_KEY);
        }
        Err(e) => {
            eprintln!("Error getting environment variable {}: {}", 'API_KEY', e);
        }
    }
    ```

    So, now `API_KEY` which is an `IgnorePattern` under `all` section has **2** values, clear text key in `main.rs` and code to get the key from environment variable `API_KEY`

12. Run `status` again

    ```bash
    >> git-selective-ignore status
    ✓ Configuration is valid.
      ├─ Line Range Pattern '13-16': 4 line(s) matched
      │  └─ Lines 13-16
      ├─ Regex Pattern 'API_KEY': 1 line(s) matched
      │  └─ Line 10
      ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
      │  └─ Line 20
      ├─ Block Pattern '// DEBUG BLOCK START ||| // DEBUG BLOCK END': 3 line(s) matched
      │  └─ Lines 6-8
      ├─ Regex Pattern 'SECRET': 1 line(s) matched
      │  └─ Line 18
      └─ Summary: 10 line(s) ignored, 17 line(s) remaining (of 27 total)
      ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
      │  └─ Line 7
      └─ Summary: 1 line(s) ignored, 20 line(s) remaining (of 21 total)
    📊 Git Selective Ignore Status Report
    =====================================
    🎯 Specifically Configured Files:
    🟡 src/main.rs (8 patterns, 10/27 lines ignored, 37.0%)

    🌐 Files Affected by Global 'ALL' Patterns:
    🟡 src/lib.rs (6 patterns, 1/21 lines ignored, 4.8%)

    📈 Summary:
      Total files: 2
      Total patterns: 8
      Total ignored lines: 11
      Files with issues: 2

    📋 Breakdown:
      Specifically configured files: 1
      Files affected by 'ALL' patterns only: 1
    ```

    `src/lib.rs` only violates 1 `IgnorePatter`

    ```bash
    🌐 Files Affected by Global 'ALL' Patterns:
    🟡 src/lib.rs (6 patterns, 1/21 lines ignored, 4.8%)
    ```

    `git-selective-ignore` recoginzed the in `src/lib.rs` has not clear text key
    infact it is pulling it from an environment variable.

13. Commit changes:

    ```bash
    git commit -m "more violations"
    ✓ Configuration is valid.
    📝 Processing files with selective ignore patterns...

    📄 Processing file: src/lib.rs
       └─ Found 6 ignore pattern(s) installed
       ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
       │  └─ Line 7
       └─ Summary: 1 line(s) ignored, 20 line(s) remaining (of 21 total)

    📄 Processing file: src/main.rs
       └─ Found 8 ignore pattern(s) installed
       ├─ Regex Pattern 'API_KEY': 1 line(s) matched
       │  └─ Line 10
       ├─ Regex Pattern 'GITHUB_TOKEN': 1 line(s) matched
       │  └─ Line 20
       ├─ Block Pattern '// DEBUG BLOCK START ||| // DEBUG BLOCK END': 3 line(s) matched
       │  └─ Lines 6-8
       ├─ Regex Pattern 'SECRET': 1 line(s) matched
       │  └─ Line 18
       ├─ Line Range Pattern '13-16': 4 line(s) matched
       │  └─ Lines 13-16
       └─ Summary: 10 line(s) ignored, 17 line(s) remaining (of 27 total)

    🔄 Re-staging modified files...
    ✅ Pre-commit processing complete.
    ✓ Configuration is valid.
    🔄 Restoring files after commit...
    ✓ Restored src/main.rs
    ✓ Restored src/lib.rs
    ✅ Post-commit processing complete.
    [main 1a05526] more violations

     1 file changed, 13 insertions(+)
    ```

    Only, 1 line was ignored.

14. Check committed changes to `git`:

    ```bash
    >> git show :src/lib.rs
    use std::env;

    fn main() {
        println!("Another Test");

        println!("{} <- My GitHub Token", GITHUB_TOKEN);

        let API_KEY = env::var('API_KEY');

        match env::var('API_KEY') {
            Ok(value) => {
                println!("The value of APP_KEY is: {}", API_KEY);
            }
            Err(e) => {
                eprintln!("Error getting environment variable {}: {}", 'API_KEY', e);
            }
        }
    }
    ```

15. Workspace after commit:

    ```bash
           │ File: src/lib.rs
       1   │ use std::env;
       2   │
       3 + │
       4   │ fn main() {
       5   │     println!("Another Test");
       6   │
       7 + │     let GITHUB_TOKEN = "github_fake_token_093790841-831-8lncdlwnelkqix12=-1x;xm;m"
       8 + │
       9   │     println!("{} <- My GitHub Token", GITHUB_TOKEN);
      10   │
      11   │     let API_KEY = env::var('API_KEY');
      12   │
      13   │     match env::var('API_KEY') {
      14   │         Ok(value) => {
      15   │             println!("The value of APP_KEY is: {}", API_KEY);
      16   │         }
      17   │         Err(e) => {
      18   │             eprintln!("Error getting environment variable {}: {}", 'API_KEY', e);
      19   │         }
      20   │     }
      21   │ }
    ```

    Only `GITHUB_API` was removed because it had clear text API Key. Notice the `+` indication.
