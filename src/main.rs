mod i18n;
mod mgr;
mod subcmd;
mod utils;
use clap::{Parser, Subcommand};
use eyre::Result;
use i18n::I18N_LOADER;
use subcmd::{add, custom_mirrors, menu, remove, reset, set, speedtest};

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const MIRRORS_PATH: &str = "/usr/share/distro-repository-data/mirrors.yml";
const BRANCHES_PATH: &str = "/usr/share/distro-repository-data/branches.yml";
const COMPONENTS_PATH: &str = "/usr/share/distro-repository-data/comps.yml";
const APT_CONFIG: &str = "/etc/apt/sources.list";
const CUSTOM_MIRRORS: &str = "/etc/apt-gen-list/custom_mirror.yml";
const SPEEDTEST_FILE_CHECKSUM: &str =
    "30e14955ebf1352266dc2ff8067e68104607e750abb9d3b36582b8af909fcb58";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    subcommand: Option<MirrorMgrCommand>,
}

#[derive(Subcommand, Debug)]
enum MirrorMgrCommand {
    /// Set APT repository mirror, branch and components
    Set(Set),
    /// Add APT repository mirror, branch and components
    Add(NormalArgs),
    /// Remove APT repository mirror, branch and components
    Remove(NormalArgs),
    /// Reset all APT repositories mirror settings
    Reset,
    /// Mirrormgr menu
    Menu,
    /// Speedtest mirrors
    Speedtest,
    /// Edit custom mirror settings
    CustomMirrors,
}

#[derive(Parser, Debug)]
#[group(required = true)]
pub struct Set {
    #[clap(short, long)]
    mirror: Option<String>,
    #[clap(short, long)]
    branch: Option<String>,
}

#[derive(Parser, Debug)]
#[group(required = true)]
pub struct NormalArgs {
    #[clap(short, long)]
    mirrors: Option<Vec<String>>,
    #[clap(short, long)]
    components: Option<Vec<String>>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(subcmd) = args.subcommand {
        match subcmd {
            MirrorMgrCommand::Set(s) => set::execute(s),
            MirrorMgrCommand::Add(a) => add::execute(a),
            MirrorMgrCommand::Remove(a) => remove::execute(a),
            MirrorMgrCommand::Reset => reset::execute(),
            MirrorMgrCommand::Menu => menu::execute(),
            MirrorMgrCommand::Speedtest => speedtest::execute(),
            MirrorMgrCommand::CustomMirrors => custom_mirrors::execute(),
        }?;
    } else {
        menu::execute()?
    }

    Ok(())
}
