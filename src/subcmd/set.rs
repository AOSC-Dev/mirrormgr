use crate::{
    mgr::{Branches, DistroConfig, MirrorManager, Mirrors},
    utils::{create_status, refresh},
    Set, APT_CONFIG, BRANCHES_PATH, MIRRORS_PATH, STATUS_FILE, fl,
};
use anyhow::Result;
use oma_console::info;

pub fn execute(args: Set) -> Result<()> {
    let status_file = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status_file);
    let branches = Branches::from_path(BRANCHES_PATH)?;

    if let Some(mirror) = args.mirror {
        let mirrors = Mirrors::from_path(MIRRORS_PATH)?;
        mm.set_mirror(&mirror, &mirrors)?;
        info!("{}", fl!("set-mirror", mirror = mirror));
    }

    if let Some(branch) = args.branch {
        mm.set_branch(&branch, &branches)?;
        info!("Branch is set to {branch}");
    }

    info!("{}", fl!("write-sources"));
    mm.apply_config(&branches, APT_CONFIG)?;

    info!("{}", fl!("run-refresh"));
    refresh()?;

    Ok(())
}
