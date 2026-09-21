//! Detects repository context used to associate todos with projects and paths.

pub mod git;

pub use git::detect_file_path;
pub use git::detect_project;
