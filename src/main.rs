use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_zoxide_scores() -> HashMap<String, f64> {
    let output = Command::new("zoxide")
        .args(["query", "--list", "--score"])
        .output()
        .expect("failed to execute zoxide");

    if !output.status.success() {
        panic!("zoxide returned non-zero exit code");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut scores = HashMap::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: "<score> <path>"
        // Path may contain spaces -> split once
        if let Some((score_str, path)) = line.split_once(' ') {
            if let Ok(score) = score_str.trim().parse::<f64>() {
                scores.insert(path.trim().to_string(), score);
            }
        }
    }

    scores
}

fn list_dirs_std(base: &Path) -> Vec<PathBuf> {
    fs::read_dir(base)
        .expect("failed to read sandbox directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    let home = env::var("HOME").expect("HOME not set");
    // Both paths are hardcoded. Don't really care about extending it.
    let sandbox = Path::new(&home).join("sandbox");
    let home_path = Path::new(&home);

    let scores = get_zoxide_scores();
    let mut dirs = Vec::new();

    dirs.extend(list_dirs_std(&sandbox));
    dirs.extend(list_dirs_std(home_path));

    let mut results: Vec<(f64, String)> = dirs
        .into_iter()
        .map(|d| {
            let path_str = d.to_string_lossy().to_string();
            let score = scores.get(&path_str).copied().unwrap_or(0.0);
            (score, path_str)
        })
        .collect();

    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    for (score, path) in results {
        println!("{:6.1} {}", score, path);
    }
}
