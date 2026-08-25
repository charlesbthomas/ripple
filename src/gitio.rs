use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub fn merge_base(repo: &Path, a: &str, b: &str) -> Result<String> {
    let output = run(repo, &["merge-base", a, b]).with_context(|| {
        format!("failed to find merge-base of `{a}` and `{b}` (does `{a}` exist?)")
    })?;
    Ok(output.trim().to_string())
}

pub fn diff_worktree(repo: &Path, against: &str) -> Result<Vec<String>> {
    let mut files = parse_z(&run_raw(repo, &["diff", "--name-only", "-z", against])?);
    files.extend(parse_z(&run_raw(
        repo,
        &["ls-files", "--others", "--exclude-standard", "-z"],
    )?));
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn diff_staged(repo: &Path) -> Result<Vec<String>> {
    let mut files = parse_z(&run_raw(repo, &["diff", "--name-only", "-z", "--cached"])?);
    files.sort();
    Ok(files)
}

pub fn diff_range(repo: &Path, range: &str) -> Result<Vec<String>> {
    let mut files = parse_z(&run_raw(repo, &["diff", "--name-only", "-z", range])?);
    files.sort();
    Ok(files)
}

fn run(repo: &Path, args: &[&str]) -> Result<String> {
    let bytes = run_raw(repo, args)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn run_raw(repo: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-c")
        .arg("core.quotePath=false")
        .args(args)
        .current_dir(repo)
        .output()
        .context("failed to run git (is git installed and on PATH?)")?;
    if !output.status.success() {
        bail!(
            "git {} failed:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

fn parse_z(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|b| *b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect()
}
