use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::env;

include!("./src/args.rs");

const GENERATED_COMPLETIONS: &[Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

fn generate_completions() {
    let mut app = Args::command();
    for shell in GENERATED_COMPLETIONS {
        generate_to(*shell, &mut app, "mirrormgr", "completions")
            .expect("Failed to generate shell completions");
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=MIRRORMGR_GEN_COMPLETIONS");
    if env::var("MIRRORMGR_GEN_COMPLETIONS").is_ok() {
        generate_completions();
    }
}
