use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Option<MirrorMgrCommand>,
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum MirrorMgrCommand {
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
    pub mirror: Option<String>,
    #[clap(short, long)]
    pub branch: Option<String>,
}

#[derive(Parser, Debug)]
#[group(required = true)]
pub struct NormalArgs {
    #[clap(short, long)]
    pub mirrors: Option<Vec<String>>,
    #[clap(short, long)]
    pub components: Option<Vec<String>>,
}
