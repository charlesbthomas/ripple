mod cli;
mod commands;
mod config;
mod engine;
mod gitio;
mod graph;
mod matcher;
mod output;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command};
use owo_colors::OwoColorize;

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Changed {
            diff,
            direct_only,
            format,
            strict,
        } => commands::changed::run(diff, *direct_only, *format, *strict),
        Command::Validate => commands::validate::run(),
        Command::Graph {
            module,
            dependents,
            deps,
            format,
        } => commands::graph::run(module.as_deref(), *dependents, *deps, *format),
        Command::Explain { module, diff } => commands::explain::run(module, diff),
        Command::Init => commands::init::run(),
        Command::List { format } => commands::list::run(*format),
        Command::Completions { shell } => {
            let mut command = Cli::command();
            clap_complete::generate(*shell, &mut command, "ripple", &mut std::io::stdout());
            Ok(())
        }
    };
    if let Err(error) = result {
        anstream::eprintln!("{} {error:#}", "error:".red().bold());
        std::process::exit(1);
    }
}
