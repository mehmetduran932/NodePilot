use std::fs;
use std::time::Instant;
use tempfile::tempdir;
use nodepilot_core::{resolve_project_node, LOCAL_METADATA_FILENAME};

#[test]
fn benchmark_shim_resolution_overhead() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a 6-level deep directory structure mimicking real-world workspaces:
    // root/org/team/monorepo/apps/my-service/src
    let deep_dir = root
        .join("org")
        .join("team")
        .join("monorepo")
        .join("apps")
        .join("my-service")
        .join("src");
    fs::create_dir_all(&deep_dir).unwrap();

    // Place .nodepilot.local at monorepo root (3 levels up)
    let monorepo_dir = root.join("org").join("team").join("monorepo");
    fs::write(monorepo_dir.join(LOCAL_METADATA_FILENAME), "22.18.0\n").unwrap();

    // Warm-up
    let _ = resolve_project_node(&deep_dir, None);

    // Benchmark 1,000 resolution passes
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let res = resolve_project_node(&deep_dir, None);
        assert!(res.is_some());
    }

    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() as f64 / iterations as f64;

    println!("\n=== NodePilot Shim Resolution Benchmark ===");
    println!("Total time for {} iterations: {:?}", iterations, elapsed);
    println!("Average lookup latency: {:.2} µs ({:.4} ms)", avg_micros, avg_micros / 1000.0);
    println!("Target ceiling: < 5.0 ms. Result: {:.4} ms\n", avg_micros / 1000.0);

    // Ensure latency is well below 1 millisecond
    assert!(avg_micros < 1000.0, "Resolution must take less than 1ms, took {:.2}µs", avg_micros);
}
