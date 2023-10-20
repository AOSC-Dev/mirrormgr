use anyhow::Result;
use oma_console::info;

use crate::{
    mgr::{Branches, DistroConfig, MirrorManager},
    utils::{create_status, refresh},
    APT_CONFIG, BRANCHES_PATH, STATUS_FILE, fl,
};

pub fn execute() -> Result<()> {
    let status = create_status(STATUS_FILE)?;
    let mm = MirrorManager::reset(status);
    let branches = Branches::from_path(BRANCHES_PATH)?;

    mm.apply_config(&branches, APT_CONFIG)?;

    info!("{}", fl!("write-sources"));
    mm.apply_config(&branches, APT_CONFIG)?;

    info!("{}", fl!("run-refresh"));
    refresh()?;

    Ok(())
}
