use crate::config::Config;
use crate::graph::ModuleGraph;
use crate::matcher::Matcher;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Direct,
    Indirect,
}

#[derive(Debug, Serialize)]
pub struct ChangedModule {
    pub name: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangedReport {
    pub modules: Vec<ChangedModule>,
    pub unowned_files: Vec<String>,
}

pub fn compute(config: &Config, graph: &ModuleGraph, files: &[String]) -> Result<ChangedReport> {
    let matcher = Matcher::build(config)?;
    let (owned, unowned_files) = matcher.assign(files);

    let direct: BTreeSet<String> = owned.keys().cloned().collect();
    let indirect = graph.dependents_closure(&direct);

    let mut modules: Vec<ChangedModule> = Vec::new();
    for (name, files) in owned {
        modules.push(ChangedModule {
            name,
            status: Status::Direct,
            via: Vec::new(),
            files,
        });
    }
    for (name, via) in indirect {
        modules.push(ChangedModule {
            name,
            status: Status::Indirect,
            via,
            files: Vec::new(),
        });
    }
    modules.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ChangedReport {
        modules,
        unowned_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Module};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn config() -> Config {
        let defs: &[(&str, &str, &[&str])] = &[
            ("core", "libs/core", &[]),
            ("api", "services/api", &["core"]),
            ("web", "apps/web", &["api"]),
            ("docs", "docs", &[]),
        ];
        let mut modules = BTreeMap::new();
        for (name, path, deps) in defs {
            modules.insert(
                name.to_string(),
                Module {
                    name: name.to_string(),
                    paths: vec![path.to_string()],
                    deps: deps.iter().map(|d| d.to_string()).collect(),
                    source: PathBuf::from("ripple.toml"),
                },
            );
        }
        Config {
            root: PathBuf::from("."),
            base: "main".to_string(),
            modules,
        }
    }

    #[test]
    fn reports_direct_and_transitive_changes() {
        let config = config();
        let graph = ModuleGraph::build(&config).unwrap();
        let files = vec![
            "libs/core/src/lib.rs".to_string(),
            "unowned.txt".to_string(),
        ];
        let report = compute(&config, &graph, &files).unwrap();

        let names: Vec<(&str, Status)> = report
            .modules
            .iter()
            .map(|m| (m.name.as_str(), m.status))
            .collect();
        assert_eq!(
            names,
            vec![
                ("api", Status::Indirect),
                ("core", Status::Direct),
                ("web", Status::Indirect),
            ]
        );
        let web = report.modules.iter().find(|m| m.name == "web").unwrap();
        assert_eq!(web.via, vec!["api", "core"]);
        assert_eq!(report.unowned_files, vec!["unowned.txt"]);
    }

    #[test]
    fn empty_diff_yields_empty_report() {
        let config = config();
        let graph = ModuleGraph::build(&config).unwrap();
        let report = compute(&config, &graph, &[]).unwrap();
        assert!(report.modules.is_empty());
        assert!(report.unowned_files.is_empty());
    }
}
