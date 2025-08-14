// This file is the module declaration file for the `core` module.
// In Rust, a `mod.rs` file within a directory (e.g., `src/core/`)
// serves two main purposes:
//
// 1. It declares the submodules contained within that directory.
// 2. It exposes these submodules to the parent module (`src/` in this case),
//    making them accessible to the entire crate.

// The `pub mod config;` declaration tells the Rust compiler to look for
// a file named `config.rs` (or `config/mod.rs`) within the same directory.
// The `pub` keyword makes the `config` module and all its public items
// (structs, functions, traits) available to the parent crate.
//
// `config` module:
// This module is responsible for managing the application's configuration.
// It defines the data structures for the configuration file (e.g., `SelectiveIgnoreConfig`),
// provides a `ConfigProvider` trait for abstracting configuration access, and
// includes a `ConfigManager` to handle file I/O operations like loading,
// saving, and validating the configuration.
pub mod config;

// The `pub mod engine;` declaration makes the `engine` module available.
//
// `engine` module:
// This module contains the main business logic of the selective ignore tool.
// It defines the `IgnoreEngine` struct, which orchestrates the entire workflow.
// This includes:
// - Interacting with the Git repository (`git2` crate).
// - Processing files during `pre-commit` and `post-commit` stages.
// - Applying to ignore patterns to file content.
// - Handling backups via a `StorageProvider`.
// The engine is the central component that ties together the configuration,
// patterns, and file system interactions to achieve the tool's primary goal.
pub mod engine;
