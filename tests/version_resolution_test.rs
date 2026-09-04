use std::fs;
use tempfile::tempdir;
use nodepilot_core::{resolve_project_node, ConfigSourceType, LOCAL_METADATA_FILENAME};

#[test]
fn test_nodepilot_local_takes_highest_priority() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create .nvmrc and .nodepilot.local in same directory
    fs::write(root.join(".nvmrc"), "18.20.8\n").unwrap();
    fs::write(root.join(LOCAL_METADATA_FILENAME), "22.18.0\n").unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "22.18.0");
    assert_eq!(resolved.source_type, ConfigSourceType::LocalOverride);
    assert!(!resolved.is_inherited);
}

#[test]
fn test_nvmrc_priority_over_engines() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join(".nvmrc"), "20.19.5\n").unwrap();
    fs::write(root.join("package.json"), r#"{"engines": {"node": ">=16.0.0"}}"#).unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "20.19.5");
    assert_eq!(resolved.source_type, ConfigSourceType::Nvmrc);
}

#[test]
fn test_node_version_file_priority() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join(".node-version"), "18.20.8\n").unwrap();
    fs::write(root.join("package.json"), r#"{"engines": {"node": ">=14.0.0"}}"#).unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "18.20.8");
    assert_eq!(resolved.source_type, ConfigSourceType::NodeVersion);
}

#[test]
fn test_volta_node_in_package_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("package.json"), r#"{"volta": {"node": "20.19.5"}}"#).unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "20.19.5");
    assert_eq!(resolved.source_type, ConfigSourceType::Volta);
}

#[test]
fn test_tool_versions_file() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join(".tool-versions"), "ruby 3.2.0\nnodejs 22.18.0\npython 3.11.0\n").unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "22.18.0");
    assert_eq!(resolved.source_type, ConfigSourceType::ToolVersions);
}

#[test]
fn test_mise_toml_file() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("mise.toml"), "[tools]\nnode = \"20.19.5\"\n").unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "20.19.5");
    assert_eq!(resolved.source_type, ConfigSourceType::Mise);
}

#[test]
fn test_parent_directory_inheritance() {
    let dir = tempdir().unwrap();
    let company_root = dir.path().join("company");
    let app_dir = company_root.join("apps").join("admin");
    fs::create_dir_all(&app_dir).unwrap();

    // Put .nodepilot.local at company_root
    fs::write(company_root.join(LOCAL_METADATA_FILENAME), "22.18.0\n").unwrap();

    // From inside app_dir, resolution should ascend and find company_root's version
    let resolved = resolve_project_node(&app_dir, None).expect("Should inherit from parent");
    assert_eq!(resolved.raw_version, "22.18.0");
    assert!(resolved.is_inherited);
    assert_eq!(resolved.source_type, ConfigSourceType::InheritedLocal);

    // Now define a specific override inside app_dir
    fs::write(app_dir.join(LOCAL_METADATA_FILENAME), "16.20.2\n").unwrap();
    let resolved_child = resolve_project_node(&app_dir, None).expect("Should find local override");
    assert_eq!(resolved_child.raw_version, "16.20.2");
    assert!(!resolved_child.is_inherited);
    assert_eq!(resolved_child.source_type, ConfigSourceType::LocalOverride);
}
