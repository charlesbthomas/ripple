use crate::cli::GraphFormat;
use crate::config::Config;
use crate::graph::{ModuleGraph, nearest_name};
use anstream::println;
use anyhow::{Result, bail};
use owo_colors::OwoColorize;
use std::collections::BTreeSet;

pub fn run(module: Option<&str>, dependents: bool, deps: bool, format: GraphFormat) -> Result<()> {
    let root = crate::config::find_root(&std::env::current_dir()?)?;
    let config = crate::config::load(&root)?;
    let graph = ModuleGraph::build(&config)?;

    match format {
        GraphFormat::Dot => print_dot(&config),
        GraphFormat::Mermaid => print_mermaid(&config),
        GraphFormat::Tree => match module {
            None => list_modules(&config),
            Some(name) => {
                require_module(&config, name)?;
                if deps {
                    print_tree(
                        name,
                        &|n| owned(graph.dependencies_of(&config, n)),
                        "depends on",
                    );
                } else if dependents {
                    print_tree(
                        name,
                        &|n| owned(graph.dependents_of(n)),
                        "is depended on by",
                    );
                } else {
                    summarize_module(&config, &graph, name);
                }
            }
        },
    }
    Ok(())
}

fn require_module(config: &Config, name: &str) -> Result<()> {
    if config.modules.contains_key(name) {
        return Ok(());
    }
    let mut message = format!("unknown module `{name}`");
    if let Some(suggestion) = nearest_name(name, config.modules.keys()) {
        message.push_str(&format!("\nhint: did you mean `{suggestion}`?"));
    }
    message.push_str("\nhint: run `ripple graph` to list all modules");
    bail!(message);
}

fn list_modules(config: &Config) {
    for module in config.modules.values() {
        if module.deps.is_empty() {
            println!("{}", module.name.bold());
        } else {
            println!(
                "{} {} {}",
                module.name.bold(),
                "->".dimmed(),
                module.deps.join(", ")
            );
        }
    }
}

fn summarize_module(config: &Config, graph: &ModuleGraph, name: &str) {
    let module = &config.modules[name];
    println!("{}", name.bold());
    println!("  {} {}", "paths:".dimmed(), module.paths.join(", "));
    println!("  {} {}", "declared in:".dimmed(), module.source.display());
    println!(
        "  {} {}",
        "depends on:".dimmed(),
        join_or_none(&module.deps)
    );
    let dependents: Vec<String> = graph
        .dependents_of(name)
        .iter()
        .map(|s| s.to_string())
        .collect();
    println!(
        "  {} {}",
        "depended on by:".dimmed(),
        join_or_none(&dependents)
    );
    println!(
        "{}",
        format!("\nhint: `ripple graph {name} --deps` or `--dependents` walks the full tree")
            .dimmed()
    );
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".to_string()
    } else {
        items.join(", ")
    }
}

fn owned(items: Vec<&str>) -> Vec<String> {
    items.into_iter().map(String::from).collect()
}

fn print_tree(name: &str, neighbors: &dyn Fn(&str) -> Vec<String>, relation: &str) {
    println!("{} {}", name.bold(), format!("({relation})").dimmed());
    let mut visited = BTreeSet::new();
    visited.insert(name.to_string());
    walk(name, neighbors, "", &mut visited);
}

fn walk(
    name: &str,
    neighbors: &dyn Fn(&str) -> Vec<String>,
    prefix: &str,
    visited: &mut BTreeSet<String>,
) {
    let children = neighbors(name);
    for (i, child) in children.iter().enumerate() {
        let last = i == children.len() - 1;
        let connector = if last { "└── " } else { "├── " };
        if visited.insert(child.clone()) {
            println!("{prefix}{connector}{child}");
            let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
            walk(child, neighbors, &child_prefix, visited);
        } else {
            println!("{prefix}{connector}{child} {}", "(already shown)".dimmed());
        }
    }
}

fn print_dot(config: &Config) {
    println!("digraph ripple {{");
    println!("  rankdir=LR;");
    for module in config.modules.values() {
        println!("  \"{}\";", module.name);
        for dep in &module.deps {
            println!("  \"{}\" -> \"{dep}\";", module.name);
        }
    }
    println!("}}");
}

fn print_mermaid(config: &Config) {
    println!("graph LR");
    for module in config.modules.values() {
        if module.deps.is_empty() {
            println!("  {}", module.name);
        }
        for dep in &module.deps {
            println!("  {} --> {dep}", module.name);
        }
    }
}
