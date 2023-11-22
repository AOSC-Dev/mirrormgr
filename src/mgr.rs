use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{Read, Seek, Write},
    path::Path,
};

use eyre::{bail, eyre, Context, Result};

use indexmap::{indexmap, IndexMap};
use os_release::OsRelease;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{info, warn};

use crate::{fl, utils::url_strip};

pub struct MirrorManager {
    status: MirrorStatus,
    status_file: File,
}

#[derive(Serialize, Deserialize, Debug)]
struct MirrorStatus {
    branch: String,
    component: Vec<String>,
    mirror: IndexMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MirrorInfo {
    url: String,
    desc: String,
}

impl MirrorInfo {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }
}

pub struct Mirror<'a>(&'a str, &'a MirrorInfo);

impl Mirror<'_> {
    pub fn inner(&self) -> (&str, &MirrorInfo) {
        (self.0, self.1)
    }
}

impl Display for Mirror<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.1.desc())
    }
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

#[derive(Serialize, Deserialize)]
pub struct CustomMirrors(HashMap<String, String>);

pub trait DistroConfig: DeserializeOwned {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = fs::read(path)?;
        let s = serde_yaml::from_slice(&f)?;

        Ok(s)
    }

    fn from_str(s: &str) -> Result<Self> {
        let s = serde_yaml::from_str(s)?;

        Ok(s)
    }

    fn from_file(f: &File) -> Result<Self> {
        let mut f = f;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        let s = serde_yaml::from_slice(&buf)?;

        Ok(s)
    }

    fn has(&self, s: &str) -> bool;
}

impl DistroConfig for MirrorStatus {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = fs::read(path)?;
        let s = serde_json::from_slice(&f)?;

        Ok(s)
    }

    fn from_file(f: &File) -> Result<Self> {
        let mut f = f;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        let s = serde_json::from_slice(&buf)?;

        Ok(s)
    }

    fn from_str(s: &str) -> Result<Self> {
        let s = serde_json::from_str(s)?;

        Ok(s)
    }

    fn has(&self, s: &str) -> bool {
        self.mirror.contains_key(s)
    }
}

impl DistroConfig for Branches {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}

impl DistroConfig for Comps {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}
impl DistroConfig for Mirrors {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}

impl DistroConfig for CustomMirrors {
    fn has(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
}

impl Mirrors {
    pub fn list_mirrors(&self) -> Vec<Mirror> {
        let mut res = vec![];
        for (k, v) in &self.0 {
            res.push(Mirror(k.as_str(), v));
        }

        res
    }

    pub fn init_custom_mirrors(&mut self, c: CustomMirrors) -> Result<()> {
        for (k, v) in c.0 {
            if self.0.contains_key(&k) {
                bail!("Distro mirror file contains {k}.");
            }

            self.0.insert(
                k,
                MirrorInfo {
                    url: v.clone(),
                    desc: format!("[Custom mirror] {v}"),
                },
            );
        }

        Ok(())
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
    pub fn set_mirror(&mut self, mirror: &str, url: String) {
        self.mirror.clear();
        self.add_mirror(mirror, url);
    }

    pub fn add_mirror(&mut self, mirror: &str, url: String) -> bool {
        if !self.has(mirror) {
            self.mirror.insert(mirror.to_owned(), url);
            return true;
        }

        false
    }

    pub fn remove_mirror(&mut self, mirror: &str) -> Result<bool> {
        if self.has(mirror) {
            if self.mirror.len() == 1 {
                bail!(fl!("no-delete-only-mirror"));
            }
            self.mirror.remove(mirror);
            return Ok(true);
        }

        Ok(false)
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

    pub fn set_branch(&mut self, branch: &str) -> bool {
        if self.branch == branch {
            return false;
        }

        self.branch = branch.to_string();

        true
    }

    pub fn write_config(&self, status_file: &File) -> Result<()> {
        let mut status_file = status_file;
        // 1. 先把文件大小设置为 0
        status_file.set_len(0)?;
        // 2. set_len 并不会调整指针的位置，因此光标的位置可能在意外的位置
        // 所以需要 reset 到开始的地方
        status_file.rewind()?;
        let s = serde_json::to_vec(self)?;
        status_file.write_all(&s)?;

        Ok(())
    }

    pub fn list_enabled_mirrors(&self) -> Vec<&str> {
        self.mirror.keys().map(|x| x.as_str()).collect()
    }
}

impl MirrorManager {
    pub fn new(status_file: File) -> Self {
        let status = MirrorStatus::from_file(&status_file).unwrap_or_default();

        Self {
            status,
            status_file,
        }
    }

    pub fn reset(status_file: File) -> Self {
        let status = MirrorStatus::default();

        Self {
            status,
            status_file,
        }
    }

    pub fn set_mirror(&mut self, set_mirror: &str, mirrors: &Mirrors) -> Result<()> {
        let entry = mirrors.0.get(set_mirror);

        if entry.is_none() {
            bail!(fl!("mirror-not-found", mirror = set_mirror));
        }

        self.status
            .set_mirror(set_mirror, entry.unwrap().url.clone());

        Ok(())
    }

    pub fn add_mirrors(&mut self, mirrors: &Mirrors, add_mirrors: &[&str]) -> Result<()> {
        for m in add_mirrors {
            let entry = mirrors.0.get(m.to_owned());

            if entry.is_none() {
                bail!(fl!("mirror-not-found", mirror = m.to_string()));
            }

            let res = self.status.add_mirror(m, entry.unwrap().url.clone());

            info!("{}", fl!("set-mirror", mirror = m.to_string()));

            if !res {
                warn!("{}", fl!("mirror-already-enabled", mirror = m.to_string()));
            }
        }

        Ok(())
    }

    pub fn remove_mirrors(&mut self, remove_mirrors: &[String]) -> Result<()> {
        for m in remove_mirrors {
            info!("{}", fl!("remove-mirror", mirror = m.clone()));
            let res = self.status.remove_mirror(m)?;

            if !res {
                warn!("{}", fl!("mirror-already-disabled", mirror = m.clone()));
            }
        }

        Ok(())
    }

    pub fn add_components(&mut self, comps: &Comps, add_comps: Vec<String>) -> Result<()> {
        for c in add_comps {
            let has = comps.has(&c);
            if !has {
                bail!(fl!("comp-not-found", comp = c))
            }

            let res = self.status.add_component(c);

            if !res {
                warn!("{}", fl!("comp-already-enabled"));
            }
        }

        Ok(())
    }

    pub fn remove_components(&mut self, remove_comps: Vec<String>) -> Result<()> {
        for c in remove_comps {
            if c == "main" {
                bail!(fl!("no-delete-only-comp"))
            }

            info!("{}", fl!("disable-comp", comp = c.clone()));

            let res = self.status.remove_component(&c);

            if !res {
                warn!("{}", fl!("comp-already-enabled"));
            }
        }

        Ok(())
    }

    pub fn set_branch(&mut self, branch: &str, branches: &Branches) -> Result<()> {
        if !branches.has(branch) {
            bail!(fl!("branch-not-found"))
        }

        let res = self.status.set_branch(branch);

        if !res {
            warn!("{}", fl!("branch-already-enabled", branch = branch));
        }

        Ok(())
    }

    pub fn try_to_string(&self, branches: &Branches) -> Result<String> {
        let mut s = String::new();
        let branches = &branches
            .0
            .get(&self.status.branch)
            .ok_or_else(|| eyre!(fl!("branch-not-found")))?
            .suites;

        let components = self.status.component.join(" ");

        for (_, url) in &self.status.mirror {
            for branch in branches {
                let url = url_strip(url);
                let entry = format!(
                    "deb {url}{} {branch} {components}\n",
                    match OsRelease::new()?.name.as_str() {
                        "AOSC OS" => "debs",
                        "AOSC OS/Retro" | "Afterglow" => "debs-retro",
                        _ => "",
                    }
                );
                s.push_str(&entry);
            }
        }

        Ok(s)
    }

    pub fn apply_config<P: AsRef<Path>>(&self, branches: &Branches, apt_path: P) -> Result<()> {
        self.status.write_config(&self.status_file)?;
        let res = self.try_to_string(branches)?;
        fs::write(apt_path, res).context("Can not write apt config")?;

        Ok(())
    }

    pub fn list_enabled_mirrors(&self) -> Vec<&str> {
        self.status.list_enabled_mirrors()
    }
}
