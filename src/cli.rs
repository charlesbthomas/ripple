use clap::builder::styling::{AnsiColor, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Yellow.on_default());

#[derive(Parser)]
#[command(
    name = "ripple",
    version,
    styles = STYLES,
    about = "Monorepo change detection: which modules changed for a given git diff",
    long_about = "\
Ripple answers one question: given a git diff, which modules of your monorepo changed?

Modules and their dependencies are declared in a `ripple.toml` at the repository
root. Ripple diffs with git, maps changed files onto modules, then follows the
dependency graph so anything that depends on a changed module is reported too --
the ripple effect.

Start with `ripple init` to scaffold a config, then `ripple changed` to see what
your current work affects. Every subcommand has worked examples in `--help`.",
    after_long_help = "\
Examples:
  ripple init                      Scaffold a starter ripple.toml
  ripple changed                   Modules affected by your working tree vs main
  ripple changed --format json     Machine-readable output for CI
  ripple validate                  Check the config and dependency graph
  ripple graph web --deps          What does `web` depend on?
  ripple explain api               Why is `api` marked as changed?"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(
        about = "List modules affected by a diff",
        long_about = "\
List modules affected by a diff: modules whose files changed (direct) plus every
module that transitively depends on one (indirect).

By default the diff is your working tree -- including uncommitted and untracked
files -- against the merge-base of HEAD and the base branch (`main`, or `base`
from ripple.toml). That matches what a pull request from your branch would
contain. Pass a RANGE for explicit control in CI.

Files that no module owns are reported on stderr; use --strict to make them an
error, which keeps the module map honest as the repository grows.",
        after_long_help = "\
Examples:
  ripple changed                        Working tree vs merge-base with main
  ripple changed --base develop         Use a different base branch
  ripple changed main...HEAD            Merge-base diff between two refs (CI)
  ripple changed HEAD~3..HEAD           Literal diff between two refs
  ripple changed --staged               Only staged changes (pre-commit)
  ripple changed --direct-only          Skip transitive dependents
  ripple changed --format json | jq     Full report as JSON
  ripple changed --format github        GitHub Actions matrix include list"
    )]
    Changed {
        #[command(flatten)]
        diff: DiffArgs,
        #[arg(long, help = "Only modules whose own files changed")]
        direct_only: bool,
        #[arg(
            long,
            value_enum,
            default_value_t = Format::Auto,
            help = "Output format (auto = table on a TTY, plain otherwise)"
        )]
        format: Format,
        #[arg(
            long,
            help = "Exit with an error if any changed file has no owning module"
        )]
        strict: bool,
    },

    #[command(
        about = "Check ripple.toml and the dependency graph for problems",
        long_about = "\
Check ripple.toml for problems: parse errors, duplicate module names, unknown
dependencies, dependency cycles, and module paths that do not exist on disk.

Exits non-zero on any error, so it slots directly into CI or a pre-commit hook.",
        after_long_help = "\
Examples:
  ripple validate          Validate the config found from the current directory"
    )]
    Validate,

    #[command(
        about = "Inspect the module dependency graph",
        long_about = "\
Inspect the module dependency graph.

With no arguments, lists every module with its paths and dependencies. With a
MODULE, walks the graph from it: --deps follows what it depends on, --dependents
follows what depends on it (the modules a change would ripple to).

The dot and mermaid formats render the whole graph for visualization tools.",
        after_long_help = "\
Examples:
  ripple graph                       List all modules
  ripple graph web --deps            Everything `web` depends on, as a tree
  ripple graph core --dependents     Everything a `core` change ripples to
  ripple graph --format mermaid      Mermaid graph for a markdown doc
  ripple graph --format dot | dot -Tsvg > graph.svg"
    )]
    Graph {
        #[arg(help = "Module to inspect (omit to list all modules)")]
        module: Option<String>,
        #[arg(
            long,
            conflicts_with = "deps",
            help = "Walk modules that depend on MODULE"
        )]
        dependents: bool,
        #[arg(long, help = "Walk modules that MODULE depends on")]
        deps: bool,
        #[arg(
            long,
            value_enum,
            default_value_t = GraphFormat::Tree,
            help = "Output format"
        )]
        format: GraphFormat,
    },

    #[command(
        about = "Show why a module is (or is not) affected by a diff",
        long_about = "\
Show why a module is, or is not, affected by a diff.

For a directly changed module, lists the exact files that matched its paths.
For an indirectly affected module, shows the dependency chain that connects it
to a changed module. Accepts the same diff selection as `ripple changed`.",
        after_long_help = "\
Examples:
  ripple explain api                     Why is `api` affected right now?
  ripple explain web main...HEAD         Same, for an explicit range"
    )]
    Explain {
        #[arg(help = "Module to explain")]
        module: String,
        #[command(flatten)]
        diff: DiffArgs,
    },

    #[command(
        about = "Create a starter ripple.toml in the current directory",
        long_about = "\
Create a starter ripple.toml in the current directory, with commented examples
covering modules, paths, globs, dependencies, and includes.

Refuses to overwrite an existing file.",
        after_long_help = "\
Examples:
  ripple init          Write ./ripple.toml"
    )]
    Init,

    #[command(
        about = "List all configured modules",
        long_about = "\
List every module declared in ripple.toml (including included fragments), one
per line, in sorted order.

Useful for scripting: it enumerates the full module set without computing a
diff, e.g. to treat every module as changed when CI cannot determine a range.",
        after_long_help = "\
Examples:
  ripple list                    One module name per line
  ripple list --format json      JSON array of module names"
    )]
    List {
        #[arg(
            long,
            value_enum,
            default_value_t = ListFormat::Plain,
            help = "Output format"
        )]
        format: ListFormat,
    },

    #[command(
        about = "Generate shell completions",
        long_about = "\
Generate a completion script for your shell on stdout.

Load it via your shell's usual mechanism, e.g.:
  ripple completions zsh > \"${fpath[1]}/_ripple\"
  ripple completions bash > /etc/bash_completion.d/ripple
  ripple completions fish > ~/.config/fish/completions/ripple.fish"
    )]
    Completions {
        #[arg(value_enum, help = "Shell to generate completions for")]
        shell: clap_complete::Shell,
    },
}

#[derive(Args)]
pub struct DiffArgs {
    #[arg(
        value_name = "RANGE",
        help = "Explicit diff: `A...B` (merge-base), `A..B` (literal), or a single ref"
    )]
    pub range: Option<String>,
    #[arg(
        long,
        value_name = "REF",
        conflicts_with_all = ["range", "staged"],
        help = "Base branch to diff against [default: `base` from ripple.toml, else main]"
    )]
    pub base: Option<String>,
    #[arg(long, conflicts_with = "range", help = "Diff staged changes only")]
    pub staged: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Auto,
    Table,
    Plain,
    Json,
    Github,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GraphFormat {
    Tree,
    Dot,
    Mermaid,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    Plain,
    Json,
}
