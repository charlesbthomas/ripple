use crate::engine::{ChangedReport, Status};
use anstream::println;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn table(report: &ChangedReport) {
    if report.modules.is_empty() {
        println!("{}", "no modules affected".dimmed());
        return;
    }
    let name_width = report
        .modules
        .iter()
        .map(|m| m.name.len())
        .chain(["MODULE".len()])
        .max()
        .unwrap();
    println!(
        "{}  {}  {}",
        format!("{:<name_width$}", "MODULE").bold(),
        format!("{:<8}", "STATUS").bold(),
        "VIA".bold()
    );
    for module in &report.modules {
        match module.status {
            Status::Direct => {
                println!("{:<name_width$}  {}", module.name, "direct".green());
            }
            Status::Indirect => {
                println!(
                    "{:<name_width$}  {}  {}",
                    module.name,
                    "indirect".yellow(),
                    module.via.join(" -> ").dimmed()
                );
            }
        }
    }
    let direct = report
        .modules
        .iter()
        .filter(|m| m.status == Status::Direct)
        .count();
    println!(
        "{}",
        format!(
            "\n{} affected ({direct} direct, {} indirect)",
            plural(report.modules.len(), "module"),
            report.modules.len() - direct
        )
        .dimmed()
    );
}

pub fn plain(report: &ChangedReport) {
    for module in &report.modules {
        println!("{}", module.name);
    }
}

pub fn json(report: &ChangedReport) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

pub fn github(report: &ChangedReport) -> Result<()> {
    let include: Vec<serde_json::Value> = report
        .modules
        .iter()
        .map(|m| serde_json::json!({ "module": m.name }))
        .collect();
    println!("{}", serde_json::json!({ "include": include }));
    Ok(())
}

pub fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}
