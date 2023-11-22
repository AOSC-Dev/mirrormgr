use crate::{
    fl,
    mgr::{Branches, DistroConfig, MirrorManager},
    utils::{create_status, distro_and_custom_mirrors, refresh, root},
    Set, APT_CONFIG, BRANCHES_PATH, STATUS_FILE,
};
use eyre::Result;
use tracing::info;

pub fn execute(args: Set) -> Result<()> {
    root()?;
    let status_file = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status_file);
    let branches = Branches::from_path(BRANCHES_PATH)?;

    if let Some(mirror) = args.mirror {
        let mirrors = distro_and_custom_mirrors()?;
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
