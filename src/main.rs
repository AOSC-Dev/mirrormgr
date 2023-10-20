mod mgr;
mod i18n;
mod subcmd;
mod utils;
use clap::{Parser, Subcommand};
use i18n::I18N_LOADER;
use subcmd::set;
use anyhow::Result;

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const MIRRORS_PATH: &str = "/usr/share/distro-repository-data/mirrors.yml";
const BRANCHES_PATH: &str = "/usr/share/distro-repository-data/branches.yml";
const APT_CONFIG: &str = "/etc/apt/sources.list";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    subcommand: MirrorMgrCommand,
}

#[derive(Subcommand, Debug)]
enum MirrorMgrCommand {
    /// Set APT repository mirror, branch and components
    Set(Set),
    /// Add APT repository mirror, branch and components
    Add(NormalArgs),
    /// Remove APT repository mirror, branch and components
    Remove(NormalArgs),
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
    mirror: Option<Vec<String>>,
    #[clap(short, long)]
    branch: Option<String>,
    #[clap(short, long)]
    components: Option<Vec<String>>,
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.subcommand {
        MirrorMgrCommand::Set(s) => set::execute(s),
        MirrorMgrCommand::Add(_) => todo!(),
        MirrorMgrCommand::Remove(_) => todo!(),
    }
}
