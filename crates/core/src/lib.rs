//! NodePilot Core library.

pub mod constants;
pub mod paths;
pub mod error;
pub mod config;
pub mod state;
pub mod resolution;
pub mod gitignore;
pub mod managers;
pub mod integration;

pub use constants::*;
pub use paths::NodePilotPaths;
pub use error::NodePilotError;
pub use config::Settings;
pub use state::{AppState, ProjectRecord, PackageManager, CompatibilityStatus, ConfigSourceType};
pub use resolution::{resolve_project_node, resolve_directory_node, ResolutionResult};
pub use gitignore::ensure_gitignore_entry;
pub use managers::{inspect_environment, EnvironmentReport, DetectedManager};
pub use integration::{get_integration_status, enable_integration, disable_integration, IntegrationStatus};
