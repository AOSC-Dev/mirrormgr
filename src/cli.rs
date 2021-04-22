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
                        .min_values(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-branch-to-default")
                .about("Set source.list branch to default")
        )
        .subcommand(
            SubCommand::with_name("add-mirror")
                .about("Set the APT source mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Add source.list mirror")
                ),
        )
        .subcommand(
            SubCommand::with_name("remove-mirror")
                .about("Remove the APT source mirror")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Remove source.list mirror")
                ),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Get apt-gen-list status")
        )
}
