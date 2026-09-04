use nodepilot_compatibility::{evaluate_compatibility, calculate_auto_assign_diff};
use nodepilot_core::{CompatibilityStatus, ProjectRecord, PackageManager, ConfigSourceType};
use std::path::PathBuf;
use chrono::Utc;

#[test]
fn test_angular_compatibility_matrix() {
    // Angular 11 with Node 14 -> Compatible
    let r1 = evaluate_compatibility("Angular", Some("11.2.0"), Some("14.21.3"), None);
    assert_eq!(r1.status, CompatibilityStatus::Compatible);

    // Angular 11 with Node 22 -> Incompatible
    let r2 = evaluate_compatibility("Angular", Some("11.2.0"), Some("22.18.0"), None);
    assert_eq!(r2.status, CompatibilityStatus::Incompatible);
    assert_eq!(r2.recommended_node.as_deref(), Some("14.21.3"));

    // Angular 13 with Node 16 -> Compatible
    let r3 = evaluate_compatibility("Angular", Some("13.3.12"), Some("16.20.2"), None);
    assert_eq!(r3.status, CompatibilityStatus::Compatible);

    // Angular 13 with Node 22 -> Incompatible
    let r4 = evaluate_compatibility("Angular", Some("13.3.12"), Some("22.18.0"), None);
    assert_eq!(r4.status, CompatibilityStatus::Incompatible);
    assert_eq!(r4.recommended_node.as_deref(), Some("16.20.2"));
}

#[test]
fn test_nextjs_compatibility_matrix() {
    // Next.js 14 with Node 20 -> Compatible
    let r1 = evaluate_compatibility("Next.js", Some("14.2.14"), Some("20.19.5"), None);
    assert_eq!(r1.status, CompatibilityStatus::Compatible);

    // Next.js 14 with Node 14 -> Incompatible
    let r2 = evaluate_compatibility("Next.js", Some("14.2.14"), Some("14.21.3"), None);
    assert_eq!(r2.status, CompatibilityStatus::Incompatible);
}

#[test]
fn test_auto_assign_diff_calculation() {
    let p1 = ProjectRecord {
        id: "p1".into(),
        name: "legacy-admin".into(),
        path: PathBuf::from("/dev/legacy-admin"),
        workspace_root: PathBuf::from("/dev"),
        framework: "Angular".into(),
        framework_version: Some("13.3.12".into()),
        package_manager: PackageManager::Npm,
        node_requirements: None,
        assigned_node: Some("22.18.0".into()), // Incompatible!
        config_source: ConfigSourceType::LocalOverride,
        config_source_path: None,
        is_inherited: false,
        compatibility_status: CompatibilityStatus::Incompatible,
        compatibility_message: None,
        recommended_node: Some("16.20.2".into()),
        has_git: true,
        is_monorepo_root: false,
        monorepo_root_path: None,
        last_scanned: Utc::now(),
    };

    let diff = calculate_auto_assign_diff(&[p1]);
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].project_name, "legacy-admin");
    assert_eq!(diff[0].current_version.as_deref(), Some("22.18.0"));
    assert_eq!(diff[0].proposed_version, "16.20.2");
}
