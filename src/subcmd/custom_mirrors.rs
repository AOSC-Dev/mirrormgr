use eyre::{eyre, Result};
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::{env, fs, path::Path};
use tracing::{info, error};

use crate::mgr::{Branches, CustomMirrors, DistroConfig, MirrorManager};
use crate::utils::{create_status, distro_and_custom_mirrors, refresh};
use crate::{fl, APT_CONFIG, BRANCHES_PATH, STATUS_FILE};
use crate::{utils::root, CUSTOM_MIRRORS};

pub fn execute() -> Result<()> {
    root()?;

    let p = Path::new(CUSTOM_MIRRORS);

    if let Some(parent) = p.parent() {
        if !p.is_dir() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(p)?;

    let len = f.metadata()?.len();

    let custom_map = if len == 0 {
        f.write_all(b"# AOSC OS mirrormgr custom mirror config file\n")?;
        f.write_all(b"# Usage: custom_mirror_name: URL\n")?;
        f.write_all(b"# Like: MY_NAS: https://localhost/aosc\n")?;
        f.write_all(b"# After, you can run `mirrormgr set --mirror MY_NAS' to use it.\n\n")?;

        CustomMirrors(HashMap::new())
    } else {
        match DistroConfig::from_file(&f) {
            Ok(config) => config,
            Err(err) => {
                error!("{}", fl!("custom-parse-failed", custom_path = CUSTOM_MIRRORS));
                return Err(eyre!(err));
            }
        }
    };

    drop(f);

    let editor = env::var("EDITOR").unwrap_or("nano".to_string());
    Command::new(editor).arg(p).spawn()?.wait()?;

    let custom_map2: CustomMirrors = match DistroConfig::from_path(p) {
        Ok(config) => config,
        Err(err) => {
            error!("{}", fl!("custom-parse-failed", custom_path = CUSTOM_MIRRORS));
            return Err(eyre!(err));
        }
    };

    let mut edited_map = HashMap::new();

    let mut is_edited = false;

    for (k, v) in custom_map.0 {
        if let Some(v2) = custom_map2.0.get(&k) {
            if &v != v2 {
                edited_map.insert(k, v);
                is_edited = true;
            }
        } else {
            edited_map.insert(k, "".to_string());
            is_edited = true;
        }
    }

    if is_edited {
        let status = create_status(STATUS_FILE)?;
        let mut mm = MirrorManager::new(status);
        let branches = Branches::from_path(BRANCHES_PATH)?;

        let mut removed_mirrors = vec![];
        let mut edited_mirrors = vec![];

        for (k, v) in edited_map {
            if v == "" {
                removed_mirrors.push(k);
            } else {
                edited_mirrors.push(k);
            }
        }

        let enabled_mirrors = mm.list_enabled_mirrors();

        let mut i = 0;
        while i < removed_mirrors.len() {
            if !enabled_mirrors.contains(&removed_mirrors[i].as_str()) {
                removed_mirrors.remove(i);
            } else {
                i += 1;
            }
        }

        let mut i = 0;
        while i < edited_mirrors.len() {
            if !enabled_mirrors.contains(&removed_mirrors[i].as_str()) {
                edited_mirrors.remove(i);
            } else {
                i += 1;
            }
        }

        mm.remove_mirrors(&removed_mirrors)?;
        mm.remove_mirrors(&edited_mirrors)?;
        let mm_info = distro_and_custom_mirrors()?;
        mm.add_mirrors(
            &mm_info,
            &edited_mirrors
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<_>>(),
        )?;

        info!("{}", fl!("write-sources"));
        mm.apply_config(&branches, APT_CONFIG)?;

        info!("{}", fl!("run-refresh"));
        refresh()?;
    }

    Ok(())
}
