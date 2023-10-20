use anyhow::Result;
use oma_console::info;

use crate::{
    fl,
    mgr::{Branches, Comps, DistroConfig, MirrorManager, Mirrors},
    utils::{create_status, refresh},
    NormalArgs, APT_CONFIG, BRANCHES_PATH, COMPONENTS_PATH, MIRRORS_PATH, STATUS_FILE,
};

pub fn execute(args: NormalArgs) -> Result<()> {
    let status = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status);

    if let Some(mirrors) = args.mirrors {
        let mm_info = Mirrors::from_path(MIRRORS_PATH)?;
        mm.add_mirrors(&mm_info, mirrors)?;
    }

    if let Some(comps) = args.components {
        let comps_info = Comps::from_path(COMPONENTS_PATH)?;
        mm.add_components(&comps_info, comps)?;
    }

    let branches = Branches::from_path(BRANCHES_PATH)?;

    info!("{}", fl!("write-sources"));
    mm.apply_config(&branches, APT_CONFIG)?;

    info!("{}", fl!("run-refresh"));
    refresh()?;

    Ok(())
}
