use clap::{crate_version, App, AppSettings, Arg, SubCommand};

/// Build the CLI instance
pub fn build_cli() -> App<'static, 'static> {
    App::new("apt-gen-list-rs")
        .version(crate_version!())
        .author("AOSC-Dev")
        .about(
            "Utility for generating sources.list for APT according to available repository configurations."
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("set-branch")
                .about("Add the APT source.list branch eg: stable")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Add the input branch name to use")
                        .max_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-mirror")
                .about("Set the APT source mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .max_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("add-mirror")
                .about("Set the multi APT source mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove-mirror")
                .about("Remove the APT source mirror (only use multi mirror)")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-mirror-to-default")
                .about("Set the APT source mirror to default")
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Get apt-gen-list status")
        )
        .subcommand(
            SubCommand::with_name("add-component")
                .about("Set the APT source mirror component")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Add source.list component")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove-component")
                .about("Remove the APT source mirror component")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Remove source.list component")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("mirrors-speedtest")
                .about("Get mirrors speedtest")
        )
}
