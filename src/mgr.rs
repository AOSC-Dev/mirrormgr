use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::Path,
};

use anyhow::{bail, Context, Result};

use indexmap::{indexmap, IndexMap};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::fl;

pub struct MirrorManager {
    status: MirrorStatus,
}

#[derive(Serialize, Deserialize)]
struct MirrorStatus {
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

#[derive(Serialize, Deserialize)]
pub struct Branches(HashMap<String, BranchInfo>);

#[derive(Serialize, Deserialize)]
pub struct Mirrors(HashMap<String, MirrorInfo>);

#[derive(Serialize, Deserialize)]
pub struct Comps(HashMap<String, String>);

trait ReadConfig: DeserializeOwned {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = fs::read(path)?;
        let s = serde_json::from_slice(&f)?;

        Ok(s)
    }

    fn has(&self, s: &str) -> bool;
}

impl ReadConfig for MirrorStatus {
    fn has(&self, s: &str) -> bool {
        self.mirror.contains_key(s)
    }
}
impl ReadConfig for Branches {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}
impl ReadConfig for Comps {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}
impl ReadConfig for Mirrors {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}

impl Default for MirrorStatus {
    fn default() -> Self {
        Self {
            branch: "stable".to_string(),
            component: vec!["main".to_string()],
            mirror: indexmap! { "origin".to_string() => "https://repo.aosc.io".to_string() },
        }
    }
}

impl MirrorStatus {
    pub fn set_mirror(&mut self, mirror: String, url: String) {
        self.mirror.clear();
        self.add_mirror(mirror, url);
    }

    pub fn add_mirror(&mut self, mirror: String, url: String) -> bool {
        if !self.has(&mirror) {
            self.mirror.insert(mirror, url);
            return true;
        }

        false
    }

    pub fn remove_mirror(&mut self, mirror: &str) -> bool {
        if self.has(mirror) {
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

impl MirrorManager {
    pub fn new<P: AsRef<Path>>(status_file: P) -> Self {
        let status = MirrorStatus::from_file(status_file).unwrap_or_default();

        Self { status }
    }

    pub fn set_mirror<P: AsRef<Path>>(&mut self, mirror: String, mirrors_file: P) -> Result<()> {
        let mirrors = Mirrors::from_file(mirrors_file)?;
        let entry = mirrors.0.get(&mirror);

        if entry.is_none() {
            bail!(fl!("mirror-not-found"));
        }

        self.status.set_mirror(mirror, entry.unwrap().url.clone());

        Ok(())
    }
}
