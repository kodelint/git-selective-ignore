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
pub mod engine;
pub mod git;
pub mod version;
