use crate::graph::ModuleGraph;
use crate::{config, output};
use anstream::{eprintln, println};
use anyhow::{Result, bail};
use owo_colors::OwoColorize;

pub fn run() -> Result<()> {
    let root = config::find_root(&std::env::current_dir()?)?;
    let config = config::load(&root)?;
    ModuleGraph::build(&config)?;

    let mut errors: Vec<String> = Vec::new();
    for module in config.modules.values() {
        for path in &module.paths {
            if path.is_empty() || path.contains(['*', '?', '[', '{']) {
                continue;
            }
            if !root.join(path).exists() {
                errors.push(format!(
                    "module `{}`: path `{path}` does not exist (declared in {})",
                    module.name,
                    module.source.display()
                ));
            }
        }
    }

    for warning in overlap_warnings(&config) {
        eprintln!("{} {warning}", "warning:".yellow().bold());
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{} {error}", "error:".red().bold());
        }
        bail!("{} found", output::plural(errors.len(), "problem"));
    }

    let edges: usize = config.modules.values().map(|m| m.deps.len()).sum();
    println!(
        "{} ripple.toml is valid: {}, {}",
        "ok:".green().bold(),
        output::plural(config.modules.len(), "module"),
        output::plural(edges, "dependency edge")
    );
    Ok(())
}

fn overlap_warnings(config: &config::Config) -> Vec<String> {
    let mut prefixes: Vec<(&str, &str)> = Vec::new();
    for module in config.modules.values() {
        for path in &module.paths {
            if !path.is_empty() && !path.contains(['*', '?', '[', '{']) {
                prefixes.push((module.name.as_str(), path.as_str()));
            }
        }
    }
    let mut warnings = Vec::new();
    for (i, (name_a, path_a)) in prefixes.iter().enumerate() {
        for (name_b, path_b) in &prefixes[i + 1..] {
            if name_a == name_b {
                continue;
            }
            let (outer, inner) = if is_path_prefix(path_a, path_b) {
                ((name_a, path_a), (name_b, path_b))
            } else if is_path_prefix(path_b, path_a) {
                ((name_b, path_b), (name_a, path_a))
            } else {
                continue;
            };
            warnings.push(format!(
                "path `{}` of module `{}` contains path `{}` of module `{}`; \
                 files under the inner path will belong to both modules",
                outer.1, outer.0, inner.1, inner.0
            ));
        }
    }
    warnings
}

fn is_path_prefix(outer: &str, inner: &str) -> bool {
    inner == outer || (inner.starts_with(outer) && inner.as_bytes().get(outer.len()) == Some(&b'/'))
}
