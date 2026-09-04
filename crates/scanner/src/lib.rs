//! NodePilot passive workspace scanner and framework inspector.

pub mod framework;
pub mod package_manager;
pub mod monorepo;
pub mod walker;

pub use framework::{detect_framework, FrameworkInfo};
pub use package_manager::detect_package_manager;
pub use monorepo::{detect_monorepo_root, MonorepoInfo};
pub use walker::scan_workspace;
