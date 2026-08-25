use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootFile {
    pub base: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub modules: BTreeMap<String, ModuleDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentFile {
    #[serde(default)]
    pub modules: BTreeMap<String, ModuleDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDef {
    pub path: PathSpec,
    #[serde(default)]
    pub deps: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PathSpec {
    One(String),
    Many(Vec<String>),
}

impl PathSpec {
    pub fn entries(&self) -> Vec<String> {
        match self {
            PathSpec::One(p) => vec![p.clone()],
            PathSpec::Many(ps) => ps.clone(),
        }
    }
}
