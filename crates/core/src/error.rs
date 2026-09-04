//! Comprehensive typed errors for NodePilot.

use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum NodePilotError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Path resolution error: {0}")]
    PathResolution(String),

    #[error("Node version '{version}' is not installed at {expected_path}")]
    VersionNotInstalled {
        version: String,
        expected_path: PathBuf,
    },

    #[error("Version parse error for '{0}'")]
    VersionParse(String),

    #[error("Node release metadata fetch error: {0}")]
    ReleaseFetch(String),

    #[error("Download error for {url}: {reason}")]
    Download {
        url: String,
        reason: String,
    },

    #[error("Checksum mismatch for {file}. Expected {expected}, got {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("Archive extraction failed: {0}")]
    Extraction(String),

    #[error("Path traversal detected in archive entry: {0}")]
    SecurityTraversal(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Recursive shim invocation detected. Aborting execution.")]
    ShimRecursionDetected,

    #[error("Project not found at path: {0}")]
    ProjectNotFound(PathBuf),

    #[error("Shell integration error: {0}")]
    Integration(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
