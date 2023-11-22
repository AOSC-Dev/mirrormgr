use eyre::Result;
use tracing::info;

use crate::{
    fl,
    mgr::{Branches, Comps, DistroConfig, MirrorManager},
    utils::{create_status, distro_and_custom_mirrors, refresh, root},
    NormalArgs, APT_CONFIG, BRANCHES_PATH, COMPONENTS_PATH, STATUS_FILE,
};

pub fn execute(args: NormalArgs) -> Result<()> {
    root()?;
    let status = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status);

    if let Some(mirrors) = args.mirrors {
        let mm_info = distro_and_custom_mirrors()?;
        mm.add_mirrors(
            &mm_info,
            &mirrors.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
        )?;
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
