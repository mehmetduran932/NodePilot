//! Idempotent .gitignore maintenance for NodePilot.
//!
//! Preserves existing line endings (\r\n vs \n), prevents duplicate entries,
//! and creates .gitignore if absent.

use std::path::Path;
use crate::constants::LOCAL_METADATA_FILENAME;
use crate::error::NodePilotError;

/// Ensures `.nodepilot.local` is listed in the project's `.gitignore`.
///
/// Returns `Ok(true)` if modified/created, or `Ok(false)` if already present.
pub fn ensure_gitignore_entry(project_dir: &Path) -> Result<bool, NodePilotError> {
    let gitignore_path = project_dir.join(".gitignore");

    if !gitignore_path.exists() {
        // Create new .gitignore with native or standard newline
        #[cfg(target_os = "windows")]
        let content = format!("{}\r\n", LOCAL_METADATA_FILENAME);
        #[cfg(not(target_os = "windows"))]
        let content = format!("{}\n", LOCAL_METADATA_FILENAME);

        std::fs::write(&gitignore_path, content)?;
        return Ok(true);
    }

    let raw_bytes = std::fs::read(&gitignore_path)?;
    let content = String::from_utf8_lossy(&raw_bytes);

    // Detect if already present as an entry (ignoring trailing whitespace)
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == LOCAL_METADATA_FILENAME || trimmed == format!("/{}", LOCAL_METADATA_FILENAME) {
            return Ok(false); // Already ignored!
        }
    }

    // Detect existing line ending style
    let line_ending = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let mut new_content = content.to_string();
    // Ensure trailing newline before adding our entry
    if !new_content.is_empty() && !new_content.ends_with('\n') && !new_content.ends_with('\r') {
        new_content.push_str(line_ending);
    }

    new_content.push_str(LOCAL_METADATA_FILENAME);
    new_content.push_str(line_ending);

    std::fs::write(&gitignore_path, new_content.as_bytes())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_creates_gitignore_when_missing() {
        let dir = tempdir().unwrap();
        let modified = ensure_gitignore_entry(dir.path()).unwrap();
        assert!(modified);

        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains(".nodepilot.local"));
    }

    #[test]
    fn test_idempotent_no_duplicate() {
        let dir = tempdir().unwrap();
        let p = dir.path().join(".gitignore");
        std::fs::write(&p, "node_modules/\n.nodepilot.local\n").unwrap();

        let modified = ensure_gitignore_entry(dir.path()).unwrap();
        assert!(!modified);

        let content = std::fs::read_to_string(&p).unwrap();
        let occurrences = content.matches(".nodepilot.local").count();
        assert_eq!(occurrences, 1);
    }

    #[test]
    fn test_preserves_crlf_line_endings() {
        let dir = tempdir().unwrap();
        let p = dir.path().join(".gitignore");
        std::fs::write(&p, "node_modules/\r\ndist/\r\n").unwrap();

        let modified = ensure_gitignore_entry(dir.path()).unwrap();
        assert!(modified);

        let bytes = std::fs::read(&p).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("dist/\r\n.nodepilot.local\r\n"));
    }
}
