use crate::cli::DiffArgs;
use crate::engine::Status;
use crate::graph::nearest_name;
use anstream::println;
use anyhow::{Result, bail};
use owo_colors::OwoColorize;

pub fn run(module: &str, diff: &DiffArgs) -> Result<()> {
    let root = crate::config::find_root(&std::env::current_dir()?)?;
    let config = crate::config::load(&root)?;
    if !config.modules.contains_key(module) {
        let mut message = format!("unknown module `{module}`");
        if let Some(suggestion) = nearest_name(module, config.modules.keys()) {
            message.push_str(&format!("\nhint: did you mean `{suggestion}`?"));
        }
        bail!(message);
    }

    let report = super::changed::compute_report(diff, false)?;
    let Some(entry) = report.modules.iter().find(|m| m.name == module) else {
        println!(
            "{} is {} by this diff",
            module.bold(),
            "not affected".green()
        );
        return Ok(());
    };

    match entry.status {
        Status::Direct => {
            println!(
                "{} is {}: {} of its files changed",
                module.bold(),
                "directly changed".green().bold(),
                entry.files.len()
            );
            for file in &entry.files {
                println!("  {file}");
            }
        }
        Status::Indirect => {
            println!(
                "{} is {}: it depends on a changed module",
                module.bold(),
                "indirectly affected".yellow().bold()
            );
            let mut chain = vec![module.to_string()];
            chain.extend(entry.via.clone());
            println!("  {}", chain.join(" -> "));
            let changed = entry.via.last().map(String::as_str).unwrap_or_default();
            println!(
                "{}",
                format!("\nhint: `ripple explain {changed}` shows what changed at the source")
                    .dimmed()
            );
        }
    }
    Ok(())
}
