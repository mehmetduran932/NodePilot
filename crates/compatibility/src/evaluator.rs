//! Compatibility evaluator and auto-assignment recommendation engine.

use serde::{Deserialize, Serialize};
use nodepilot_core::{CompatibilityStatus, ProjectRecord};
use crate::matrix::{get_angular_rules, get_nextjs_rules, FrameworkRule};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityReport {
    pub status: CompatibilityStatus,
    pub message: String,
    pub recommended_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAssignProposal {
    pub project_id: String,
    pub project_name: String,
    pub current_version: Option<String>,
    pub proposed_version: String,
    pub reason: String,
}

/// Evaluates compatibility of a project framework & engines against assigned or potential Node versions.
pub fn evaluate_compatibility(
    framework: &str,
    framework_version: Option<&str>,
    assigned_node: Option<&str>,
    engines_node: Option<&str>,
) -> CompatibilityReport {
    let assigned_clean = assigned_node.map(|v| v.trim().trim_start_matches('v'));

    // Check engines.node requirement first if present
    if let (Some(engines_req), Some(node_ver)) = (engines_node, assigned_clean) {
        if let Ok(req) = semver::VersionReq::parse(engines_req) {
            if let Ok(ver) = semver::Version::parse(node_ver) {
                if !req.matches(&ver) {
                    return CompatibilityReport {
                        status: CompatibilityStatus::Incompatible,
                        message: format!("Assigned Node {} does not satisfy package engines: '{}'", node_ver, engines_req),
                        recommended_node: None,
                    };
                }
            }
        }
    }

    let framework_lower = framework.to_lowercase();
    let fw_major = framework_version.and_then(parse_major_version);

    if framework_lower.contains("angular") {
        if let Some(major) = fw_major {
            let rules = get_angular_rules();
            return match_rules(&rules, major, assigned_clean, "Angular");
        }
    } else if framework_lower.contains("next") {
        if let Some(major) = fw_major {
            let rules = get_nextjs_rules();
            return match_rules(&rules, major, assigned_clean, "Next.js");
        }
    }

    // Default / generic compatibility
    if let Some(node_ver) = assigned_clean {
        CompatibilityReport {
            status: CompatibilityStatus::Compatible,
            message: format!("Node {} configured", node_ver),
            recommended_node: None,
        }
    } else {
        CompatibilityReport {
            status: CompatibilityStatus::Unknown,
            message: "No Node version assigned yet".into(),
            recommended_node: Some("22.18.0".into()),
        }
    }
}

fn match_rules(
    rules: &[FrameworkRule],
    fw_major: u32,
    assigned_node: Option<&str>,
    framework_display: &str,
) -> CompatibilityReport {
    let rule = rules.iter().find(|r| {
        if fw_major < r.min_framework_major {
            return false;
        }
        if let Some(max) = r.max_framework_major {
            if fw_major > max {
                return false;
            }
        }
        true
    });

    let rule = match rule {
        Some(r) => r,
        None => {
            return CompatibilityReport {
                status: CompatibilityStatus::Compatible,
                message: format!("{} {} compatibility assumed", framework_display, fw_major),
                recommended_node: Some("22.18.0".into()),
            };
        }
    };

    let recommended = Some(rule.recommended_node.to_string());

    if let Some(node_ver) = assigned_node {
        if let Some(node_major) = parse_major_version(node_ver) {
            if rule.supported_node_majors.contains(&node_major) {
                CompatibilityReport {
                    status: CompatibilityStatus::Compatible,
                    message: format!("Node {} is compatible with {} {}", node_ver, framework_display, fw_major),
                    recommended_node: recommended,
                }
            } else {
                CompatibilityReport {
                    status: CompatibilityStatus::Incompatible,
                    message: format!(
                        "Node {} is incompatible with {} {}. {}",
                        node_ver, framework_display, fw_major, rule.notes
                    ),
                    recommended_node: recommended,
                }
            }
        } else {
            CompatibilityReport {
                status: CompatibilityStatus::Unknown,
                message: format!("Could not parse assigned Node version '{}'", node_ver),
                recommended_node: recommended,
            }
        }
    } else {
        CompatibilityReport {
            status: CompatibilityStatus::Incompatible,
            message: format!("Missing Node version for {} {}. {}", framework_display, fw_major, rule.notes),
            recommended_node: recommended,
        }
    }
}

fn parse_major_version(ver: &str) -> Option<u32> {
    let clean = ver.trim_start_matches(|c: char| !c.is_ascii_digit());
    clean.split('.').next()?.parse::<u32>().ok()
}

/// Computes the diff of recommended auto-assignments across a list of projects.
pub fn calculate_auto_assign_diff(projects: &[ProjectRecord]) -> Vec<AutoAssignProposal> {
    let mut diffs = Vec::new();

    for proj in projects {
        let report = evaluate_compatibility(
            &proj.framework,
            proj.framework_version.as_deref(),
            proj.assigned_node.as_deref(),
            proj.node_requirements.as_deref(),
        );

        if let Some(recommended) = report.recommended_node {
            let current = proj.assigned_node.clone();
            let is_different = match &current {
                Some(curr) => curr.trim_start_matches('v') != recommended.trim_start_matches('v'),
                None => true,
            };

            if is_different {
                diffs.push(AutoAssignProposal {
                    project_id: proj.id.clone(),
                    project_name: proj.name.clone(),
                    current_version: current,
                    proposed_version: recommended,
                    reason: report.message,
                });
            }
        }
    }

    diffs
}
