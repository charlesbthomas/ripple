use crate::config::Config;
use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};
use std::collections::BTreeMap;

enum Pattern {
    Root,
    Prefix(String),
    Glob(GlobMatcher),
}

pub struct Matcher {
    patterns: Vec<(String, Pattern)>,
}

impl Matcher {
    pub fn build(config: &Config) -> Result<Self> {
        let mut patterns = Vec::new();
        for module in config.modules.values() {
            for path in &module.paths {
                let pattern = if path.is_empty() {
                    Pattern::Root
                } else if path.contains(['*', '?', '[', '{']) {
                    Pattern::Glob(
                        Glob::new(path)
                            .with_context(|| {
                                format!("invalid glob `{path}` in module `{}`", module.name)
                            })?
                            .compile_matcher(),
                    )
                } else {
                    Pattern::Prefix(path.clone())
                };
                patterns.push((module.name.clone(), pattern));
            }
        }
        Ok(Self { patterns })
    }

    pub fn owners(&self, file: &str) -> Vec<&str> {
        let mut owners: Vec<&str> = self
            .patterns
            .iter()
            .filter(|(_, pattern)| match pattern {
                Pattern::Root => true,
                Pattern::Prefix(prefix) => {
                    file == prefix
                        || (file.len() > prefix.len()
                            && file.starts_with(prefix.as_str())
                            && file.as_bytes()[prefix.len()] == b'/')
                }
                Pattern::Glob(glob) => glob.is_match(file),
            })
            .map(|(name, _)| name.as_str())
            .collect();
        owners.sort();
        owners.dedup();
        owners
    }

    pub fn assign(&self, files: &[String]) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
        let mut owned: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut unowned: Vec<String> = Vec::new();
        for file in files {
            let owners = self.owners(file);
            if owners.is_empty() {
                unowned.push(file.clone());
            }
            for owner in owners {
                owned
                    .entry(owner.to_string())
                    .or_default()
                    .push(file.clone());
            }
        }
        (owned, unowned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Module};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn config(defs: &[(&str, &[&str])]) -> Config {
        let mut modules = BTreeMap::new();
        for (name, paths) in defs {
            modules.insert(
                name.to_string(),
                Module {
                    name: name.to_string(),
                    paths: paths.iter().map(|p| p.to_string()).collect(),
                    deps: Vec::new(),
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
    fn prefix_matches_directory_boundaries() {
        let matcher = Matcher::build(&config(&[("core", &["libs/core"])])).unwrap();
        assert_eq!(matcher.owners("libs/core/src/lib.rs"), vec!["core"]);
        assert_eq!(matcher.owners("libs/core"), vec!["core"]);
        assert!(matcher.owners("libs/core-extras/lib.rs").is_empty());
        assert!(matcher.owners("other/file.rs").is_empty());
    }

    #[test]
    fn glob_patterns_match() {
        let matcher = Matcher::build(&config(&[("proto", &["proto/**/*.proto"])])).unwrap();
        assert_eq!(matcher.owners("proto/api/v1/user.proto"), vec!["proto"]);
        assert!(matcher.owners("proto/README.md").is_empty());
    }

    #[test]
    fn root_path_matches_everything() {
        let matcher = Matcher::build(&config(&[("repo", &[""])])).unwrap();
        assert_eq!(matcher.owners("anything/at/all.txt"), vec!["repo"]);
    }

    #[test]
    fn assign_splits_owned_and_unowned() {
        let matcher = Matcher::build(&config(&[
            ("core", &["libs/core"]),
            ("api", &["services/api"]),
        ]))
        .unwrap();
        let files = vec![
            "libs/core/lib.rs".to_string(),
            "services/api/main.rs".to_string(),
            "README.md".to_string(),
        ];
        let (owned, unowned) = matcher.assign(&files);
        assert_eq!(owned["core"], vec!["libs/core/lib.rs"]);
        assert_eq!(owned["api"], vec!["services/api/main.rs"]);
        assert_eq!(unowned, vec!["README.md"]);
    }

    #[test]
    fn overlapping_paths_yield_multiple_owners() {
        let matcher = Matcher::build(&config(&[
            ("api", &["services/api"]),
            ("all-services", &["services"]),
        ]))
        .unwrap();
        assert_eq!(
            matcher.owners("services/api/main.rs"),
            vec!["all-services", "api"]
        );
    }
}
