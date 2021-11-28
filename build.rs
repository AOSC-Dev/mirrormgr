use clap::Shell;
use os_release::OsRelease;
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
    let is_aosc = OsRelease::new().unwrap().name.contains("AOSC OS");
    if cfg!(feature = "aosc") && !is_aosc {
        panic!("It appears that you are not using AOSC OS, please re-compile apt-gen-list with the --no-default-features option");
    } else if !cfg!(feature = "aosc") && is_aosc {
        println!("cargo:warning=It appears that you are running apt-gen-list on AOSC OS without distro-specific features enabled. Please re-compile apt-gen-list with the --features aosc option.")
    }
    println!("cargo:rerun-if-env-changed=AGL_GEN_COMPLETIONS");
    if env::var("AGL_GEN_COMPLETIONS").is_ok() {
        generate_completions();
    }
}
