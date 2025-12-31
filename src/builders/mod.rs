// This file is the module declaration file for the `builders` module.
// It declares and makes public all the sub-modules within the `src/builders`
// directory. These modules encapsulate various utility and construction logic.

// The `pub mod hooks;` declaration exposes the `hooks` module.
//
// `hooks` module:
// This module contains all the logic related to Git hooks. It is responsible
// for installing and uninstalling the `pre-commit` and `post-commit` hook
// scripts in the `.git/hooks` directory. The hooks are essential for automating
// the selective ignore process.
pub mod hooks;

// The `pub mod importer;` declaration exposes the `importer` module.
//
// `importer` module:
// This module provides functionality for importing ignore patterns from
// external sources, such as a custom file format or future integrations
// like `.gitignore`-style files. It handles the parsing and conversion
// of these external patterns into the internal `IgnorePattern` format.
pub mod importer;

// The `pub mod patterns;` declaration exposes the `patterns` module.
//
// `patterns` module:
// This is a fundamental module that defines the core data structures for
// ignore patterns (`IgnorePattern` and `PatternType`). It also provides
// the `PatternMatcher` trait and its implementation, which contain the
// logic for matching file content against various pattern types (e.g.,
// line regexes, line numbers, block start/end markers).
pub mod patterns;

// The `pub mod reporter;` declaration exposes the `reporter` module.
//
// `reporter` module:
// This module is responsible for generating human-readable reports and status
// updates. It defines a `StatusReporter` trait and its `ConsoleReporter`
// implementation, which displays a summary of the configured files and
// the ignored lines.
pub mod reporter;

// The `pub mod storage;` declaration exposes the `storage` module.
//
// `storage` module:
// This module provides an abstraction for handling temporary backup data.
// It defines the `StorageProvider` trait and concrete implementations
// like `TempFileStorage` and `MemoryStorage`, which are used to store
// and retrieve the original file content during the `pre-commit` and
// `post-commit` phases.
pub mod storage;

// The `pub mod validator;` declaration exposes the `validator` module.
//
// `validator` module:
// This module is dedicated to ensuring the integrity and correctness of
// the configuration. It defines the `ConfigValidator` trait and a
// `StandardValidator` implementation to check for common issues like
// invalid patterns, conflicting rules, and non-existent files.
pub mod validator;
