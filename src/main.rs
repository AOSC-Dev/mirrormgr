mod i18n;
mod mgr;
mod subcmd;
mod utils;
use clap::{Parser, Subcommand};
use eyre::Result;
use i18n::I18N_LOADER;
use oma_console::OmaLayer;
use subcmd::{add, custom_mirrors, menu, remove, reset, set, speedtest};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

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
    #[arg(short, long)]
    debug: bool,
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

    init_logger(args.debug);

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

fn init_logger(is_debug: bool) {
    if !is_debug {
        let no_i18n_embd_info: EnvFilter = "i18n_embed=error,info".parse().unwrap();

        tracing_subscriber::registry()
            .with(
                OmaLayer
                    .with_filter(no_i18n_embd_info)
                    .and_then(LevelFilter::INFO),
            )
            .init();
    } else {
        let env_log = EnvFilter::try_from_default_env();

        if let Ok(filter) = env_log {
            tracing_subscriber::registry()
                .with(fmt::layer().with_filter(filter))
                .init();
        } else {
            tracing_subscriber::registry().with(fmt::layer()).init();
        }
    }
}
