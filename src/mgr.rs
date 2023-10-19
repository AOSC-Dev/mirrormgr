use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::Path,
};

use anyhow::{bail, Context, Result};

use indexmap::{indexmap, IndexMap};
use oma_console::warn;
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

    pub fn set_mirror(&mut self, set_mirror: String, mirrors: &Mirrors) -> Result<()> {
        let entry = mirrors.0.get(&set_mirror);

        if entry.is_none() {
            bail!(fl!("mirror-not-found", mirror = set_mirror));
        }

        self.status
            .set_mirror(set_mirror, entry.unwrap().url.clone());

        Ok(())
    }

    pub fn add_mirrors(&mut self, mirrors: &Mirrors, add_mirrors: Vec<String>) -> Result<()> {
        for m in add_mirrors {
            let entry = mirrors.0.get(&m);

            if entry.is_none() {
                bail!(fl!("mirror-not-found", mirror = m));
            }

            let res = self
                .status
                .add_mirror(m.clone(), entry.unwrap().url.clone());

            if !res {
                warn!("{}", fl!("mirror-already-enabled", mirror = m));
            }
        }

        Ok(())
    }

    pub fn remove_mirrors(&mut self, mirrors: &Mirrors, remove_mirrors: Vec<String>) -> Result<()> {
        if self.status.mirror.len() <= 1 {
            bail!(fl!("no-delete-only-mirror"));
        }

        for m in remove_mirrors {
            let entry = mirrors.0.get(&m);

            if entry.is_none() {
                bail!(fl!("mirror-not-found", mirror = m));
            }

            let res = self.status.remove_mirror(&m);

            if !res {
                warn!("{}", fl!("mirror-already-disabled", mirror = m));
            }
        }

        Ok(())
    }

    pub fn add_component(&mut self, comps: &Comps, add_comps: Vec<String>) -> Result<()> {
        for c in add_comps {
            let has = comps.has(&c);
            if !has {
                bail!(fl!("comp-not-found"))
            }

            let res = self.status.add_component(c);

            if !res {
                warn!("{}", fl!("comp-already-enabled"));
            }
        }

        Ok(())
    }
}
