//! Matching of project version specs (`20`, `v20.19`, `lts/*`, `>=18 <23`, `^20 || ^22`) against concrete versions.

use crate::paths::NodePilotPaths;

/// LTS codenames by major version.
const LTS_CODENAMES: &[(&str, u64)] = &[
    ("argon", 4),
    ("boron", 6),
    ("carbon", 8),
    ("dubnium", 10),
    ("erbium", 12),
    ("fermium", 14),
    ("gallium", 16),
    ("hydrogen", 18),
    ("iron", 20),
    ("jod", 22),
    ("krypton", 24),
];

/// True when `spec` pins one version line (`20`, `20.19`, `20.19.5`) rather than a range or alias.
pub fn is_partial_version(spec: &str) -> bool {
    let s = spec.trim().trim_start_matches(['v', 'V']);
    !s.is_empty() && s.split('.').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// True when `spec` is a semver range (e.g. from `engines.node`) rather than a pinned version or alias.
pub fn is_range_spec(spec: &str) -> bool {
    let s = spec.trim().to_lowercase();
    !is_partial_version(&s) && !is_alias(&s)
}

fn is_alias(s: &str) -> bool {
    matches!(s, "node" | "latest" | "stable" | "current" | "lts" | "*") || s.starts_with("lts/")
}

/// Checks whether a concrete `version` (e.g. `22.18.0`) satisfies `spec`.
pub fn version_satisfies(spec: &str, version: &str) -> bool {
    let spec = spec.trim().to_lowercase();
    let version = version.trim().trim_start_matches('v');
    let Ok(parsed) = semver::Version::parse(version) else {
        return spec.trim_start_matches('v') == version;
    };

    match spec.as_str() {
        "node" | "latest" | "stable" | "current" | "*" | "" => return true,
        "lts" | "lts/*" => return parsed.major >= 4 && parsed.major % 2 == 0,
        _ => {}
    }
    if let Some(codename) = spec.strip_prefix("lts/") {
        return LTS_CODENAMES
            .iter()
            .any(|(name, major)| *name == codename && *major == parsed.major);
    }

    if is_partial_version(&spec) {
        let wanted: Vec<u64> = spec
            .trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        let actual = [parsed.major, parsed.minor, parsed.patch];
        return wanted.iter().zip(actual.iter()).all(|(w, a)| w == a);
    }

    // npm-style range: alternatives separated by `||`, comparators separated by spaces
    spec.split("||").any(|alt| {
        let comparators: Vec<String> = alt
            .split_whitespace()
            .map(|c| c.trim_end_matches(',').to_string())
            .collect();
        if comparators.is_empty() {
            return true;
        }
        semver::VersionReq::parse(&comparators.join(", "))
            .map(|req| req.matches(&parsed))
            .unwrap_or(false)
    })
}

/// Returns the highest installed NodePilot version satisfying `spec`, if any.
pub fn find_installed_match(paths: &NodePilotPaths, spec: &str) -> Option<String> {
    // Exact directory match first (cheap and handles non-semver folder names)
    let exact = spec.trim().trim_start_matches('v');
    if !exact.is_empty() && paths.node_binary_path(exact).is_file() {
        return Some(exact.to_string());
    }

    let entries = std::fs::read_dir(&paths.versions_dir).ok()?;
    entries
        .flatten()
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| !name.starts_with('.') && paths.node_binary_path(name).is_file())
        .filter_map(|name| semver::Version::parse(&name).ok().map(|v| (v, name)))
        .filter(|(_, name)| version_satisfies(spec, name))
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_versions_match_by_prefix() {
        assert!(version_satisfies("20", "20.19.5"));
        assert!(version_satisfies("v20.19", "20.19.5"));
        assert!(!version_satisfies("20.1", "20.19.5"));
        assert!(!version_satisfies("2", "20.19.5"));
    }

    #[test]
    fn aliases_and_ranges() {
        assert!(version_satisfies("lts/*", "22.18.0"));
        assert!(!version_satisfies("lts/*", "23.1.0"));
        assert!(version_satisfies("lts/iron", "20.19.5"));
        assert!(!version_satisfies("lts/iron", "22.18.0"));
        assert!(version_satisfies(">=18.19.1", "22.18.0"));
        assert!(version_satisfies(">=18 <21", "20.19.5"));
        assert!(!version_satisfies(">=18 <21", "22.18.0"));
        assert!(version_satisfies("^18.19.1 || ^20.11.1 || >=22.0.0", "20.19.5"));
        assert!(version_satisfies("18.x", "18.20.8"));
        assert!(is_range_spec(">=18"));
        assert!(!is_range_spec("20"));
        assert!(!is_range_spec("lts/*"));
    }
}
