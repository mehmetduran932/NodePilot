use std::fs;
use tempfile::tempdir;
use nodepilot_scanner::scan_workspace;
use nodepilot_core::{PackageManager, CompatibilityStatus};

#[test]
fn test_scanner_detects_angular_and_lockfile() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let proj_dir = root.join("my-angular-app");
    fs::create_dir_all(&proj_dir).unwrap();

    let pkg_json = r#"{
        "name": "my-angular-app",
        "dependencies": {
            "@angular/core": "^13.3.0"
        }
    }"#;
    fs::write(proj_dir.join("package.json"), pkg_json).unwrap();
    fs::write(proj_dir.join("package-lock.json"), "{}").unwrap();

    let projects = scan_workspace(root, None, 5).unwrap();
    assert_eq!(projects.len(), 1);
    let p = &projects[0];
    assert_eq!(p.name, "my-angular-app");
    assert_eq!(p.framework, "Angular");
    assert_eq!(p.framework_version.as_deref(), Some("13.3.0"));
    assert_eq!(p.package_manager, PackageManager::Npm);
}

#[test]
fn test_scanner_detects_nextjs_and_pnpm() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let proj_dir = root.join("my-next-app");
    fs::create_dir_all(&proj_dir).unwrap();

    let pkg_json = r#"{
        "name": "my-next-app",
        "dependencies": {
            "next": "14.2.14",
            "react": "18.3.1"
        }
    }"#;
    fs::write(proj_dir.join("package.json"), pkg_json).unwrap();
    fs::write(proj_dir.join("pnpm-lock.yaml"), "lockfileVersion: '6.0'").unwrap();

    let projects = scan_workspace(root, None, 5).unwrap();
    assert_eq!(projects.len(), 1);
    let p = &projects[0];
    assert_eq!(p.framework, "Next.js");
    assert_eq!(p.framework_version.as_deref(), Some("14.2.14"));
    assert_eq!(p.package_manager, PackageManager::Pnpm);
}

#[test]
fn test_scanner_prunes_node_modules_and_dist() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let proj_dir = root.join("parent-app");
    let nm_dir = proj_dir.join("node_modules").join("nested-lib");
    let dist_dir = proj_dir.join("dist").join("build-lib");
    fs::create_dir_all(&nm_dir).unwrap();
    fs::create_dir_all(&dist_dir).unwrap();

    fs::write(proj_dir.join("package.json"), r#"{"name": "parent-app"}"#).unwrap();
    fs::write(nm_dir.join("package.json"), r#"{"name": "nested-lib"}"#).unwrap();
    fs::write(dist_dir.join("package.json"), r#"{"name": "build-lib"}"#).unwrap();

    let projects = scan_workspace(root, None, 5).unwrap();
    // Only parent-app should be discovered; node_modules and dist must be pruned!
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "parent-app");
}
