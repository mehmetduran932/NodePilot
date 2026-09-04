use std::fs;
use tempfile::tempdir;
use nodepilot_core::{resolve_project_node, ConfigSourceType, LOCAL_METADATA_FILENAME};

#[test]
fn test_nodepilot_local_takes_highest_priority() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join(".nvmrc"), "18.20.8\n").unwrap();
    fs::write(root.join(LOCAL_METADATA_FILENAME), "22.18.0\n").unwrap();

    let resolved = resolve_project_node(root, None).expect("Should resolve Node version");
    assert_eq!(resolved.raw_version, "22.18.0");
    assert_eq!(resolved.source_type, ConfigSourceType::LocalOverride);
    assert!(!resolved.is_inherited);
}

#[test]
fn test_parent_directory_inheritance() {
    let dir = tempdir().unwrap();
    let company_root = dir.path().join("company");
    let app_dir = company_root.join("apps").join("admin");
    fs::create_dir_all(&app_dir).unwrap();

    fs::write(company_root.join(LOCAL_METADATA_FILENAME), "22.18.0\n").unwrap();

    let resolved = resolve_project_node(&app_dir, None).expect("Should inherit from parent");
    assert_eq!(resolved.raw_version, "22.18.0");
    assert!(resolved.is_inherited);
    assert_eq!(resolved.source_type, ConfigSourceType::InheritedLocal);

    fs::write(app_dir.join(LOCAL_METADATA_FILENAME), "16.20.2\n").unwrap();
    let resolved_child = resolve_project_node(&app_dir, None).expect("Should find local override");
    assert_eq!(resolved_child.raw_version, "16.20.2");
    assert!(!resolved_child.is_inherited);
    assert_eq!(resolved_child.source_type, ConfigSourceType::LocalOverride);
}
