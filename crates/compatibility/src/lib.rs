//! Framework compatibility and recommendation matrix engine.

pub mod matrix;
pub mod evaluator;

pub use evaluator::{evaluate_compatibility, calculate_auto_assign_diff, CompatibilityReport, AutoAssignProposal};
pub use matrix::{get_angular_rules, get_nextjs_rules, FrameworkRule};
