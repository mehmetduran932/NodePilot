//! Passive framework and framework version detector.

use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    pub framework: String,
    pub framework_version: Option<String>,
}

/// Detects framework and version purely passively from package.json and project files.
/// Never executes any code.
pub fn detect_framework(project_dir: &Path, pkg_json: Option<&Value>) -> FrameworkInfo {
    // 1. Angular
    let has_angular_json = project_dir.join("angular.json").is_file();
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);

        if let Some(ver) = deps.get("@angular/core").or_else(|| deps.get("@angular/cli")) {
            return FrameworkInfo {
                framework: "Angular".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }
    if has_angular_json {
        return FrameworkInfo {
            framework: "Angular".into(),
            framework_version: None,
        };
    }

    // 2. Next.js
    let has_next_config = project_dir.join("next.config.js").is_file()
        || project_dir.join("next.config.mjs").is_file()
        || project_dir.join("next.config.ts").is_file();
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("next") {
            return FrameworkInfo {
                framework: "Next.js".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }
    if has_next_config {
        return FrameworkInfo {
            framework: "Next.js".into(),
            framework_version: None,
        };
    }

    // 3. Nuxt
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("nuxt").or_else(|| deps.get("nuxt3")) {
            return FrameworkInfo {
                framework: "Nuxt".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 4. Vue
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("vue") {
            return FrameworkInfo {
                framework: "Vue".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 5. Svelte / SvelteKit
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("@sveltejs/kit") {
            return FrameworkInfo {
                framework: "SvelteKit".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
        if let Some(ver) = deps.get("svelte") {
            return FrameworkInfo {
                framework: "Svelte".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 6. NestJS
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("@nestjs/core") {
            return FrameworkInfo {
                framework: "NestJS".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 7. Express
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("express") {
            return FrameworkInfo {
                framework: "Express".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 8. React
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("react") {
            return FrameworkInfo {
                framework: "React".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }

    // 9. Vite
    let has_vite_config = project_dir.join("vite.config.js").is_file()
        || project_dir.join("vite.config.ts").is_file()
        || project_dir.join("vite.config.mjs").is_file();
    if let Some(pkg) = pkg_json {
        let deps = get_all_dependencies(pkg);
        if let Some(ver) = deps.get("vite") {
            return FrameworkInfo {
                framework: "Vite".into(),
                framework_version: Some(clean_semver(ver)),
            };
        }
    }
    if has_vite_config {
        return FrameworkInfo {
            framework: "Vite".into(),
            framework_version: None,
        };
    }

    // 10. Nx
    if project_dir.join("nx.json").is_file() {
        return FrameworkInfo {
            framework: "Nx".into(),
            framework_version: None,
        };
    }

    // Generic Node.js project
    FrameworkInfo {
        framework: "Node.js".into(),
        framework_version: None,
    }
}

fn get_all_dependencies(pkg: &Value) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    let sections = ["dependencies", "devDependencies", "peerDependencies"];
    for sec in sections {
        if let Some(obj) = pkg.get(sec).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(ver_str) = v.as_str() {
                    map.insert(k.clone(), ver_str.to_string());
                }
            }
        }
    }

    map
}

fn clean_semver(raw: &str) -> String {
    raw.trim_start_matches(['^', '~', '>', '=', 'v', '<'])
        .trim()
        .to_string()
}
