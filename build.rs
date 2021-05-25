use clap::Shell;
use std::env;

include!("src/cli.rs");

const GENERATED_COMPLETIONS: &[Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

fn generate_completions() {
    let mut app = build_cli();
    for shell in GENERATED_COMPLETIONS {
        app.gen_completions("apt-gen-list", *shell, "completions");
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=AGL_GEN_COMPLETIONS");
    if env::var("AGL_GEN_COMPLETIONS").is_ok() {
        generate_completions();
    }
}
