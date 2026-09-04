//! Knowledge matrix for framework versions and Node.js compatibility.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRule {
    pub min_framework_major: u32,
    pub max_framework_major: Option<u32>,
    pub supported_node_majors: Vec<u32>,
    pub recommended_node: &'static str,
    pub notes: &'static str,
}

pub fn get_angular_rules() -> Vec<FrameworkRule> {
    vec![
        FrameworkRule {
            min_framework_major: 2,
            max_framework_major: Some(7),
            supported_node_majors: vec![8, 10],
            recommended_node: "10.24.1",
            notes: "Angular 2-7 requires Node 8.x or 10.x",
        },
        FrameworkRule {
            min_framework_major: 8,
            max_framework_major: Some(8),
            supported_node_majors: vec![10, 12],
            recommended_node: "12.22.12",
            notes: "Angular 8 requires Node 10.9+ or 12.x",
        },
        FrameworkRule {
            min_framework_major: 9,
            max_framework_major: Some(9),
            supported_node_majors: vec![10, 12],
            recommended_node: "12.22.12",
            notes: "Angular 9 requires Node 10.13+ or 12.x",
        },
        FrameworkRule {
            min_framework_major: 10,
            max_framework_major: Some(10),
            supported_node_majors: vec![12, 14],
            recommended_node: "14.21.3",
            notes: "Angular 10 requires Node 12.0+ or 14.x",
        },
        FrameworkRule {
            min_framework_major: 11,
            max_framework_major: Some(11),
            supported_node_majors: vec![10, 12, 14],
            recommended_node: "14.21.3",
            notes: "Angular 11 requires Node 10.13 - 12.x or 14.x",
        },
        FrameworkRule {
            min_framework_major: 12,
            max_framework_major: Some(12),
            supported_node_majors: vec![12, 14],
            recommended_node: "14.21.3",
            notes: "Angular 12 requires Node 12.14+ or 14.15+",
        },
        FrameworkRule {
            min_framework_major: 13,
            max_framework_major: Some(13),
            supported_node_majors: vec![12, 14, 16],
            recommended_node: "16.20.2",
            notes: "Angular 13 requires Node 12.20+, 14.15+, or 16.10+",
        },
        FrameworkRule {
            min_framework_major: 14,
            max_framework_major: Some(14),
            supported_node_majors: vec![14, 16],
            recommended_node: "16.20.2",
            notes: "Angular 14 requires Node 14.15+ or 16.10+",
        },
        FrameworkRule {
            min_framework_major: 15,
            max_framework_major: Some(15),
            supported_node_majors: vec![14, 16, 18],
            recommended_node: "18.20.8",
            notes: "Angular 15 requires Node 14.20+, 16.13+, or 18.10+",
        },
        FrameworkRule {
            min_framework_major: 16,
            max_framework_major: Some(16),
            supported_node_majors: vec![16, 18],
            recommended_node: "18.20.8",
            notes: "Angular 16 requires Node 16.14+ or 18.10+",
        },
        FrameworkRule {
            min_framework_major: 17,
            max_framework_major: Some(17),
            supported_node_majors: vec![18, 20],
            recommended_node: "20.19.5",
            notes: "Angular 17 requires Node 18.13+ or 20.9+",
        },
        FrameworkRule {
            min_framework_major: 18,
            max_framework_major: Some(18),
            supported_node_majors: vec![18, 20, 22],
            recommended_node: "22.18.0",
            notes: "Angular 18 requires Node 18.19+, 20.9+, or 22.0+",
        },
        FrameworkRule {
            min_framework_major: 19,
            max_framework_major: Some(19),
            supported_node_majors: vec![18, 20, 22],
            recommended_node: "22.18.0",
            notes: "Angular 19 requires Node 18.19+, 20.11+, or 22.0+",
        },
        FrameworkRule {
            min_framework_major: 20,
            max_framework_major: None,
            supported_node_majors: vec![20, 22, 24],
            recommended_node: "22.18.0",
            notes: "Angular 20+ requires Node 20.11+ or 22+",
        },
    ]
}

pub fn get_nextjs_rules() -> Vec<FrameworkRule> {
    vec![
        FrameworkRule {
            min_framework_major: 11,
            max_framework_major: Some(11),
            supported_node_majors: vec![12, 14, 16],
            recommended_node: "14.21.3",
            notes: "Next.js 11 requires Node 12.22+",
        },
        FrameworkRule {
            min_framework_major: 12,
            max_framework_major: Some(12),
            supported_node_majors: vec![12, 14, 16],
            recommended_node: "16.20.2",
            notes: "Next.js 12 requires Node 12.22+",
        },
        FrameworkRule {
            min_framework_major: 13,
            max_framework_major: Some(13),
            supported_node_majors: vec![14, 16, 18],
            recommended_node: "18.20.8",
            notes: "Next.js 13 requires Node 14.18.2+ or 16.8.0+",
        },
        FrameworkRule {
            min_framework_major: 14,
            max_framework_major: Some(14),
            supported_node_majors: vec![18, 20],
            recommended_node: "20.19.5",
            notes: "Next.js 14 requires Node 18.17+",
        },
        FrameworkRule {
            min_framework_major: 15,
            max_framework_major: None,
            supported_node_majors: vec![18, 20, 22],
            recommended_node: "22.18.0",
            notes: "Next.js 15 requires Node 18.18+",
        },
    ]
}
