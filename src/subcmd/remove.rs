use eyre::Result;
use tracing::info;

use crate::{
    fl,
    mgr::{Branches, DistroConfig, MirrorManager},
    utils::{create_status, refresh, root},
    APT_CONFIG, BRANCHES_PATH, STATUS_FILE, args::NormalArgs,
};

pub fn execute(args: NormalArgs) -> Result<()> {
    root()?;
    let status = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status);

    if let Some(mirrors) = args.mirrors {
        mm.remove_mirrors(&mirrors)?;
    }

    if let Some(comps) = args.components {
        mm.remove_components(comps)?;
    }

    let branches = Branches::from_path(BRANCHES_PATH)?;

    info!("{}", fl!("write-sources"));
    mm.apply_config(&branches, APT_CONFIG)?;

    info!("{}", fl!("run-refresh"));
    refresh()?;

    Ok(())
}
