//! Node runtime downloading, verification, installation, and inventory registry.

pub mod release;
pub mod download;
pub mod installer;
pub mod registry;

pub use release::{fetch_releases, resolve_version_alias, get_archive_filename, NodeRelease};
pub use download::{download_and_verify_archive, DownloadedArchive};
pub use installer::{install_node_archive, uninstall_node_version};
pub use registry::{list_installed_versions, is_version_installed, InstalledVersion};
