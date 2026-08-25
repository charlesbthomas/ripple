use crate::config::Config;
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug)]
pub struct ModuleGraph {
    pub dependents: BTreeMap<String, Vec<String>>,
}

impl ModuleGraph {
    pub fn build(config: &Config) -> Result<Self> {
        let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in config.modules.keys() {
            dependents.insert(name.clone(), Vec::new());
        }
        for module in config.modules.values() {
            for dep in &module.deps {
                if !config.modules.contains_key(dep) {
                    let mut message = format!(
                        "module `{}` depends on unknown module `{dep}` (declared in {})",
                        module.name,
                        module.source.display()
                    );
                    if let Some(suggestion) = nearest_name(dep, config.modules.keys()) {
                        message.push_str(&format!("\nhint: did you mean `{suggestion}`?"));
                    }
                    bail!(message);
                }
                dependents.get_mut(dep).unwrap().push(module.name.clone());
            }
        }
        let graph = Self { dependents };
        graph.check_cycles(config)?;
        Ok(graph)
    }

    fn check_cycles(&self, config: &Config) -> Result<()> {
        let mut state: BTreeMap<&str, u8> = BTreeMap::new();
        for start in config.modules.keys() {
            if state.get(start.as_str()).copied().unwrap_or(0) != 0 {
                continue;
            }
            let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
            let mut path: Vec<&str> = Vec::new();
            while let Some((node, next_dep)) = stack.pop() {
                if next_dep == 0 {
                    state.insert(node, 1);
                    path.push(node);
                }
                let deps = &config.modules[node].deps;
                if next_dep < deps.len() {
                    stack.push((node, next_dep + 1));
                    let dep = deps[next_dep].as_str();
                    match state.get(dep).copied().unwrap_or(0) {
                        0 => stack.push((dep, 0)),
                        1 => {
                            let cycle_start = path.iter().position(|n| *n == dep).unwrap();
                            let mut cycle: Vec<&str> = path[cycle_start..].to_vec();
                            cycle.push(dep);
                            bail!("dependency cycle detected: {}", cycle.join(" -> "));
                        }
                        _ => {}
                    }
                } else {
                    state.insert(node, 2);
                    path.pop();
                }
            }
        }
        Ok(())
    }

    pub fn dependents_closure(&self, direct: &BTreeSet<String>) -> BTreeMap<String, Vec<String>> {
        let mut via: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut queue: VecDeque<String> = direct.iter().cloned().collect();
        while let Some(node) = queue.pop_front() {
            let Some(dependents) = self.dependents.get(&node) else {
                continue;
            };
            for dependent in dependents {
                if direct.contains(dependent) || via.contains_key(dependent) {
                    continue;
                }
                let mut chain = vec![node.clone()];
                chain.extend(via.get(&node).cloned().unwrap_or_default());
                via.insert(dependent.clone(), chain);
                queue.push_back(dependent.clone());
            }
        }
        via
    }

    pub fn dependencies_of<'a>(&self, config: &'a Config, name: &str) -> Vec<&'a str> {
        config
            .modules
            .get(name)
            .map(|m| m.deps.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    pub fn dependents_of(&self, name: &str) -> Vec<&str> {
        self.dependents
            .get(name)
            .map(|d| d.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }
}

pub fn nearest_name<'a, I>(target: &str, candidates: I) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a String>,
{
    candidates
        .into_iter()
        .map(|c| (edit_distance(target, c), c.as_str()))
        .filter(|(d, c)| *d <= 2.min(c.len().saturating_sub(1)))
        .min()
        .map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            current.push((prev[j] + cost).min(prev[j + 1] + 1).min(current[j] + 1));
        }
        prev = current;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Module};
    use std::path::PathBuf;

    fn config(defs: &[(&str, &[&str])]) -> Config {
        let modules = defs
            .iter()
            .map(|(name, deps)| {
                (
                    name.to_string(),
                    Module {
                        name: name.to_string(),
                        paths: vec![name.to_string()],
                        deps: deps.iter().map(|d| d.to_string()).collect(),
                        source: PathBuf::from("ripple.toml"),
                    },
                )
            })
            .collect();
        Config {
            root: PathBuf::from("."),
            base: "main".to_string(),
            modules,
        }
    }

    #[test]
    fn rejects_unknown_dep_with_suggestion() {
        let config = config(&[("proto", &[]), ("web", &["protos"])]);
        let err = ModuleGraph::build(&config).unwrap_err().to_string();
        assert!(err.contains("unknown module `protos`"), "{err}");
        assert!(err.contains("did you mean `proto`?"), "{err}");
    }

    #[test]
    fn rejects_cycles_with_path() {
        let config = config(&[("a", &["b"]), ("b", &["c"]), ("c", &["a"])]);
        let err = ModuleGraph::build(&config).unwrap_err().to_string();
        assert!(err.contains("cycle"), "{err}");
        assert!(err.contains("a -> b -> c -> a"), "{err}");
    }

    #[test]
    fn closure_reports_via_chains() {
        let config = config(&[
            ("core", &[]),
            ("api", &["core"]),
            ("web", &["api"]),
            ("cli", &["core"]),
            ("docs", &[]),
        ]);
        let graph = ModuleGraph::build(&config).unwrap();
        let direct: BTreeSet<String> = ["core".to_string()].into();
        let via = graph.dependents_closure(&direct);
        assert_eq!(via["api"], vec!["core"]);
        assert_eq!(via["web"], vec!["api", "core"]);
        assert_eq!(via["cli"], vec!["core"]);
        assert!(!via.contains_key("docs"));
    }

    #[test]
    fn closure_excludes_direct_modules() {
        let config = config(&[("core", &[]), ("api", &["core"])]);
        let graph = ModuleGraph::build(&config).unwrap();
        let direct: BTreeSet<String> = ["core".to_string(), "api".to_string()].into();
        assert!(graph.dependents_closure(&direct).is_empty());
    }
}
