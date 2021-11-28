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
    if cfg!(feature = "aosc") && !OsRelease::new().unwrap().name.contains("AOSC OS") {
        panic!("If you detect that you are using aosc feature but your system is not AOSC OS, please use --no-default-features to compile");
    } else if !cfg!(feature = "aosc") && OsRelease::new().unwrap().name.contains("AOSC OS") {
        println!("cargo:warning=Detected that you are using AOSC OS but did not turn on feature aosc, if you don't know, use --features aosc to turn on this fearture")
    }
    println!("cargo:rerun-if-env-changed=AGL_GEN_COMPLETIONS");
    if env::var("AGL_GEN_COMPLETIONS").is_ok() {
        generate_completions();
    }
}
