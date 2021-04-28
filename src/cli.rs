use clap::{crate_version, App, AppSettings, Arg, SubCommand};

/// Build the CLI instance
pub fn build_cli() -> App<'static, 'static> {
    App::new("apt-gen-list-rs")
        .version(crate_version!())
        .author("AOSC-Dev")
        .about(
            "Utility for generating APT sources.list from available repository configurations."
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("set-branch")
                .about("Set APT repository branch (e.g., stable)")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Input branch name here")
                        .max_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-mirror")
                .about("Set APT repository mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .max_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("add-mirror")
                .about("Add additional APT repository mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove-mirror")
                .about("Remove APT repository mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("source.list mirror")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-mirror-to-default")
                .about("Set default APT repository mirror")
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Show apt-gen-list status")
        )
        .subcommand(
            SubCommand::with_name("add-component")
                .about("Set APT repository component")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Input component name")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove-component")
                .about("Remove APT repository component")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Input component name to be removed")
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("mirrors-speedtest")
                .about("Run speed-test on available mirrors")
        )
        .subcommand(
            SubCommand::with_name("set-fastest-mirror")
                .about("Set fastest mirror as default")
        )
}
