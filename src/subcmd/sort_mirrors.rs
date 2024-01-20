use dialoguer::{theme::ColorfulTheme, Sort};
use eyre::{eyre, Result};
use tracing::info;

use crate::{
    fl,
    mgr::{Branches, DistroConfig, MirrorManager},
    utils::{create_status, root},
    APT_CONFIG, BRANCHES_PATH, STATUS_FILE,
};

pub fn execute() -> Result<()> {
    root()?;
    let status_file = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status_file);
    let branches = Branches::from_path(BRANCHES_PATH)?;

    let enabled_mirrors = mm
        .list_enabled_mirrors()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let sorted = Sort::with_theme(&ColorfulTheme::default())
        .with_prompt("Order enabled mirrors")
        .items(&enabled_mirrors)
        .interact()
        .map_err(|_| eyre!(""))?;

    let mut res = vec![];

    for i in sorted {
        res.push(enabled_mirrors[i].clone());
    }

    mm.reorder_mirrors(res);

    info!("{}", fl!("write-sources"));
    mm.apply_config(&branches, APT_CONFIG)?;

    Ok(())
}
