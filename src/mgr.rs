use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
};

use anyhow::{Context, Result};

use indexmap::{indexmap, IndexMap};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::fl;

pub struct MirrorManager {
    config: MirrorConfig,
    mirrors_map: OnceCell<HashMap<String, Vec<MirrorInfo>>>,
    branches_map: OnceCell<HashMap<String, Vec<BranchInfo>>>,
    comps: OnceCell<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct MirrorConfig {
    branch: String,
    component: Vec<String>,
    mirror: IndexMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct MirrorInfo {
    url: String,
}

#[derive(Serialize, Deserialize)]
struct BranchInfo {
    suites: Vec<String>,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            branch: "stable".to_string(),
            component: vec!["main".to_string()],
            mirror: indexmap! { "origin".to_string() => "https://repo.aosc.io".to_string() },
        }
    }
}

impl MirrorConfig {
    pub fn from_file<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let f = fs::read(config_path)?;
        let s = serde_json::from_slice(&f)?;

        Ok(s)
    }

    pub fn set_mirror(&mut self, mirror: String, url: String) {
        self.mirror.clear();
        self.add_mirror(mirror, url);
    }

    pub fn add_mirror(&mut self, mirror: String, url: String) -> bool {
        if self.mirror.get(&mirror).is_none() {
            self.mirror.insert(mirror, url);
            return true;
        }

        false
    }

    pub fn remove_mirror(&mut self, mirror: &str) -> bool {
        if self.mirror.get(mirror).is_some() {
            self.mirror.remove(mirror);
            return true;
        }

        false
    }

    pub fn add_component(&mut self, comp: String) -> bool {
        let pos = self.component.iter().position(|x| x == &comp);
        if pos.is_none() {
            self.component.push(comp);
            return true;
        }

        false
    }

    pub fn remove_component(&mut self, comp: &str) -> bool {
        let pos = self.component.iter().position(|x| x == comp);

        if let Some(pos) = pos {
            self.component.remove(pos);
            return true;
        }

        false
    }

    pub fn set_branch(&mut self, branch: String) -> bool {
        if self.branch == branch {
            return false;
        }

        self.branch = branch;

        true
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn write_config<P: AsRef<Path>>(&mut self, config_path: P) -> Result<()> {
        let s = serde_json::to_vec(self)?;
        fs::write(config_path, s)?;

        Ok(())
    }
}
