use crate::cli::{DiffArgs, Format};
use crate::config::Config;
use crate::engine::ChangedReport;
use crate::graph::ModuleGraph;
use crate::{config, engine, gitio, output};
use anstream::eprintln;
use anyhow::{Result, bail};
use owo_colors::OwoColorize;
use std::io::IsTerminal;

pub fn run(diff: &DiffArgs, direct_only: bool, format: Format, strict: bool) -> Result<()> {
    let report = compute_report(diff, direct_only)?;

    if !report.unowned_files.is_empty() {
        warn_unowned(&report.unowned_files);
        if strict {
            bail!(
                "{} not owned by any module (--strict)",
                output::plural(report.unowned_files.len(), "changed file")
            );
        }
    }

    match resolve_format(format) {
        Format::Table => output::table(&report),
        Format::Plain => output::plain(&report),
        Format::Json => output::json(&report)?,
        Format::Github => output::github(&report)?,
        Format::Auto => unreachable!(),
    }
    Ok(())
}

pub fn compute_report(diff: &DiffArgs, direct_only: bool) -> Result<ChangedReport> {
    let root = config::find_root(&std::env::current_dir()?)?;
    let config = config::load(&root)?;
    let graph = ModuleGraph::build(&config)?;
    let files = changed_files(&config, diff)?;
    let mut report = engine::compute(&config, &graph, &files)?;
    if direct_only {
        report
            .modules
            .retain(|m| m.status == crate::engine::Status::Direct);
    }
    Ok(report)
}

pub fn changed_files(config: &Config, diff: &DiffArgs) -> Result<Vec<String>> {
    let repo = &config.root;
    if diff.staged {
        return gitio::diff_staged(repo);
    }
    match &diff.range {
        Some(range) if range.contains("..") => gitio::diff_range(repo, range),
        Some(single_ref) => {
            let base = gitio::merge_base(repo, single_ref, "HEAD")?;
            gitio::diff_worktree(repo, &base)
        }
        None => {
            let base_ref = diff.base.as_deref().unwrap_or(&config.base);
            let base = gitio::merge_base(repo, base_ref, "HEAD")?;
            gitio::diff_worktree(repo, &base)
        }
    }
}

fn resolve_format(format: Format) -> Format {
    if format == Format::Auto {
        if std::io::stdout().is_terminal() {
            Format::Table
        } else {
            Format::Plain
        }
    } else {
        format
    }
}

fn warn_unowned(files: &[String]) {
    eprintln!(
        "{} {} not owned by any module:",
        "warning:".yellow().bold(),
        output::plural(files.len(), "changed file")
    );
    for file in files.iter().take(10) {
        eprintln!("  {file}");
    }
    if files.len() > 10 {
        eprintln!("  ... and {} more", files.len() - 10);
    }
    eprintln!(
        "{}",
        "hint: add them to a module in ripple.toml, or ignore this if intentional".dimmed()
    );
}
