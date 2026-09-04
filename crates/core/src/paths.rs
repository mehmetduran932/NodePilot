use std::path::{Path, PathBuf};
#[cfg(not(target_os = "windows"))]
use directories::UserDirs;
use directories::BaseDirs;
use crate::constants::APP_NAME;

/// Normalizes a path, stripping Windows UNC verbatim `\\?\` prefixes that cause issues with cmd/explorer.
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(c) = path.canonicalize() {
        #[cfg(target_os = "windows")]
        {
            let s = c.to_string_lossy();
            if s.starts_with(r"\\?\UNC\") {
                PathBuf::from(format!(r"\\{}", &s[8..]))
            } else if s.starts_with(r"\\?\") {
                PathBuf::from(&s[4..])
            } else {
                c
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            c
        }
    } else {
        path.to_path_buf()
    }
}

#[derive(Debug, Clone)]
pub struct NodePilotPaths {
    pub root: PathBuf,
    pub versions_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl NodePilotPaths {
    /// Resolves standard NodePilot application directory paths.
    ///
    /// On Windows: `%LOCALAPPDATA%\NodePilot`
    /// On macOS/Linux: `~/.nodepilot`
    pub fn default() -> Result<Self, crate::error::NodePilotError> {
        let root = Self::resolve_root_dir()?;
        let paths = Self {
            versions_dir: root.join("versions"),
            cache_dir: root.join("cache"),
            bin_dir: root.join("bin"),
            config_dir: root.join("config"),
            state_dir: root.join("state"),
            logs_dir: root.join("logs"),
            root,
        };
        Ok(paths)
    }

    fn resolve_root_dir() -> Result<PathBuf, crate::error::NodePilotError> {
        #[cfg(target_os = "windows")]
        {
            if let Some(base) = BaseDirs::new() {
                // data_local_dir() resolves to %LOCALAPPDATA%
                Ok(base.data_local_dir().join(APP_NAME))
            } else {
                let home = std::env::var("USERPROFILE")
                    .map_err(|_| crate::error::NodePilotError::PathResolution("Cannot determine USERPROFILE".into()))?;
                Ok(PathBuf::from(home).join("AppData").join("Local").join(APP_NAME))
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(user_dirs) = UserDirs::new() {
                Ok(user_dirs.home_dir().join(".nodepilot"))
            } else {
                let home = std::env::var("HOME")
                    .map_err(|_| crate::error::NodePilotError::PathResolution("Cannot determine HOME directory".into()))?;
                Ok(PathBuf::from(home).join(".nodepilot"))
            }
        }
    }

    /// Ensures all system directories exist.
    pub fn ensure_dirs(&self) -> Result<(), crate::error::NodePilotError> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.versions_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.bin_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.state_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        Ok(())
    }

    /// Path to settings.json
    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// Path to state.json
    pub fn state_file(&self) -> PathBuf {
        self.state_dir.join("state.json")
    }

    /// Path to specific Node version directory
    pub fn version_dir(&self, version: &str) -> PathBuf {
        // Strip leading 'v' if present for clean directory names
        let clean_version = version.trim_start_matches('v');
        self.versions_dir.join(clean_version)
    }

    /// Path to Node binary for a specific version
    pub fn node_binary_path(&self, version: &str) -> PathBuf {
        let v_dir = self.version_dir(version);
        #[cfg(target_os = "windows")]
        {
            v_dir.join("node.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            v_dir.join("bin").join("node")
        }
    }

    /// Path to npm executable for a specific version
    pub fn npm_binary_path(&self, version: &str) -> PathBuf {
        let v_dir = self.version_dir(version);
        #[cfg(target_os = "windows")]
        {
            v_dir.join("npm.cmd")
        }
        #[cfg(not(target_os = "windows"))]
        {
            v_dir.join("bin").join("npm")
        }
    }

    /// Path to npx executable for a specific version
    pub fn npx_binary_path(&self, version: &str) -> PathBuf {
        let v_dir = self.version_dir(version);
        #[cfg(target_os = "windows")]
        {
            v_dir.join("npx.cmd")
        }
        #[cfg(not(target_os = "windows"))]
        {
            v_dir.join("bin").join("npx")
        }
    }

    /// Path to corepack executable for a specific version
    pub fn corepack_binary_path(&self, version: &str) -> PathBuf {
        let v_dir = self.version_dir(version);
        #[cfg(target_os = "windows")]
        {
            v_dir.join("corepack.cmd")
        }
        #[cfg(not(target_os = "windows"))]
        {
            v_dir.join("bin").join("corepack")
        }
    }

    /// Directory containing executables for a specific Node version
    pub fn version_bin_dir(&self, version: &str) -> PathBuf {
        let v_dir = self.version_dir(version);
        #[cfg(target_os = "windows")]
        {
            v_dir
        }
        #[cfg(not(target_os = "windows"))]
        {
            v_dir.join("bin")
        }
    }
}
