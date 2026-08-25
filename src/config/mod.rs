pub mod schema;

use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

pub const CONFIG_FILE: &str = "ripple.toml";

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub paths: Vec<String>,
    pub deps: Vec<String>,
    pub source: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    pub root: PathBuf,
    pub base: String,
    pub modules: BTreeMap<String, Module>,
}

pub fn find_root(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(CONFIG_FILE).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!(
                "no {CONFIG_FILE} found in {} or any parent directory\n\
                 hint: run `ripple init` at your repository root to create one",
                start.display()
            );
        }
    }
}

pub fn load(root: &Path) -> Result<Config> {
    let root_file = root.join(CONFIG_FILE);
    let text = std::fs::read_to_string(&root_file)
        .with_context(|| format!("failed to read {}", root_file.display()))?;
    let parsed: schema::RootFile = toml::from_str(&text)
        .with_context(|| format!("failed to parse {}", root_file.display()))?;

    let mut modules: BTreeMap<String, Module> = BTreeMap::new();
    insert_modules(&mut modules, parsed.modules, &root_file, root, root)?;

    for pattern in &parsed.include {
        let fragments = expand_include(root, pattern)
            .with_context(|| format!("failed to expand include pattern `{pattern}`"))?;
        for fragment_path in fragments {
            if fragment_path == root_file {
                continue;
            }
            let text = std::fs::read_to_string(&fragment_path)
                .with_context(|| format!("failed to read {}", fragment_path.display()))?;
            let fragment: schema::FragmentFile = toml::from_str(&text)
                .with_context(|| format!("failed to parse {}", fragment_path.display()))?;
            let fragment_dir = fragment_path.parent().unwrap_or(root).to_path_buf();
            insert_modules(
                &mut modules,
                fragment.modules,
                &fragment_path,
                &fragment_dir,
                root,
            )?;
        }
    }

    Ok(Config {
        root: root.to_path_buf(),
        base: parsed.base.unwrap_or_else(|| "main".to_string()),
        modules,
    })
}

fn insert_modules(
    modules: &mut BTreeMap<String, Module>,
    defs: BTreeMap<String, schema::ModuleDef>,
    source: &Path,
    base_dir: &Path,
    root: &Path,
) -> Result<()> {
    for (name, def) in defs {
        if let Some(existing) = modules.get(&name) {
            bail!(
                "module `{name}` is defined twice:\n  - {}\n  - {}",
                existing.source.display(),
                source.display()
            );
        }
        let mut paths = Vec::new();
        for entry in def.path.entries() {
            paths.push(resolve_path_entry(&entry, base_dir, root).with_context(|| {
                format!(
                    "invalid path `{entry}` for module `{name}` in {}",
                    source.display()
                )
            })?);
        }
        modules.insert(
            name.clone(),
            Module {
                name,
                paths,
                deps: def.deps,
                source: source.to_path_buf(),
            },
        );
    }
    Ok(())
}

fn resolve_path_entry(entry: &str, base_dir: &Path, root: &Path) -> Result<String> {
    let joined = base_dir.join(entry);
    let mut parts: Vec<String> = Vec::new();
    for component in joined.strip_prefix(root).unwrap_or(&joined).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::ParentDir => {
                if parts.pop().is_none() {
                    bail!("path escapes the repository root");
                }
            }
            other => bail!("unsupported path component `{other:?}`"),
        }
    }
    Ok(parts.join("/"))
}

fn expand_include(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let full = root.join(pattern);
    let full = full.to_string_lossy();
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in glob::glob(&full)? {
        let path = entry?;
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn loads_root_modules_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "ripple.toml",
            r#"
[modules.core]
path = "libs/core"

[modules.api]
path = ["services/api", "proto/api/**"]
deps = ["core"]
"#,
        );
        let config = load(dir.path()).unwrap();
        assert_eq!(config.base, "main");
        assert_eq!(config.modules.len(), 2);
        let api = &config.modules["api"];
        assert_eq!(api.paths, vec!["services/api", "proto/api/**"]);
        assert_eq!(api.deps, vec!["core"]);
    }

    #[test]
    fn respects_configured_base() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "ripple.toml", "base = \"develop\"\n");
        let config = load(dir.path()).unwrap();
        assert_eq!(config.base, "develop");
    }

    #[test]
    fn merges_included_fragments_with_relative_paths() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "ripple.toml",
            r#"
include = ["services/*/ripple.toml"]

[modules.core]
path = "libs/core"
"#,
        );
        write(
            dir.path(),
            "services/api/ripple.toml",
            r#"
[modules.api]
path = "."
deps = ["core"]
"#,
        );
        write(
            dir.path(),
            "services/web/ripple.toml",
            r#"
[modules.web]
path = ["ui", "shared/assets"]
"#,
        );
        let config = load(dir.path()).unwrap();
        assert_eq!(config.modules.len(), 3);
        assert_eq!(config.modules["api"].paths, vec!["services/api"]);
        assert_eq!(
            config.modules["web"].paths,
            vec!["services/web/ui", "services/web/shared/assets"]
        );
    }

    #[test]
    fn rejects_duplicate_module_names() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "ripple.toml",
            r#"
include = ["libs/*/ripple.toml"]

[modules.core]
path = "libs/core"
"#,
        );
        write(
            dir.path(),
            "libs/core/ripple.toml",
            r#"
[modules.core]
path = "."
"#,
        );
        let err = load(dir.path()).unwrap_err().to_string();
        assert!(err.contains("defined twice"), "{err}");
    }

    #[test]
    fn rejects_paths_escaping_root() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "ripple.toml",
            r#"
[modules.bad]
path = "../outside"
"#,
        );
        let err = load(dir.path()).unwrap_err();
        assert!(format!("{err:#}").contains("escapes"), "{err:#}");
    }

    #[test]
    fn finds_root_from_nested_directory() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "ripple.toml", "");
        write(dir.path(), "a/b/file.txt", "");
        let found = find_root(&dir.path().join("a/b")).unwrap();
        assert_eq!(
            found.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }
}
