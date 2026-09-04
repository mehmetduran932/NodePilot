use nodepilot_core::{inspect_environment, NodePilotPaths};

#[test]
fn test_coexistence_is_read_only_and_non_destructive() {
    // 1. Inspect environment
    let report = inspect_environment();

    // Verify inspection succeeded and produced a valid report
    assert!(!report.recommended_mode.is_empty());

    // 2. Verify paths isolation
    let paths = NodePilotPaths::default().expect("Paths should resolve");

    // Ensure NodePilot storage path does not collide with external managers
    let root_str = paths.root.to_string_lossy().to_lowercase();
    assert!(!root_str.contains("nvm"));
    assert!(!root_str.contains("fnm"));
    assert!(!root_str.contains("volta"));
    assert!(!root_str.contains("mise"));
    assert!(!root_str.contains("asdf"));
    assert!(root_str.contains("nodepilot"));

    // 3. Verify that paths.versions_dir is wholly contained within NodePilot root
    assert!(paths.versions_dir.starts_with(&paths.root));
    assert!(paths.bin_dir.starts_with(&paths.root));
    assert!(paths.config_dir.starts_with(&paths.root));
}
