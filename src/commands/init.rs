use crate::config::CONFIG_FILE;
use anstream::println;
use anyhow::{Result, bail};
use owo_colors::OwoColorize;

const TEMPLATE: &str = r#"# ripple.toml -- monorepo module map for `ripple`
#
# Each [modules.<name>] declares a module: which paths belong to it and which
# other modules it depends on. `ripple changed` maps a git diff onto these
# modules and follows `deps` to report everything a change ripples to.
#
# Docs for every command: `ripple --help`, `ripple changed --help`, ...

# Base branch used by `ripple changed` when no range is given.
# base = "main"

# Pull in additional [modules.*] tables from other ripple.toml files.
# Paths inside a fragment are relative to that fragment's directory.
# include = ["services/*/ripple.toml"]

# A module owning a single directory:
# [modules.core]
# path = "libs/core"

# A module owning several paths; entries with glob characters are matched as
# globs, everything else is a directory prefix:
# [modules.api]
# path = ["services/api", "proto/api/**"]
# deps = ["core"]

# A module that changes when anything it depends on changes:
# [modules.web]
# path = "apps/web"
# deps = ["api"]
"#;

pub fn run() -> Result<()> {
    let path = std::env::current_dir()?.join(CONFIG_FILE);
    if path.exists() {
        bail!("{} already exists, refusing to overwrite", path.display());
    }
    std::fs::write(&path, TEMPLATE)?;
    println!("{} wrote {}", "ok:".green().bold(), path.display());
    println!(
        "{}",
        "next: declare your modules, then check them with `ripple validate`".dimmed()
    );
    Ok(())
}
