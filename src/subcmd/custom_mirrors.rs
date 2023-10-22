use anyhow::Result;
use std::io::Write;
use std::process::Command;
use std::{env, fs, path::Path};

use crate::{utils::root, CUSTOM_MIRRORS};

pub fn execute() -> Result<()> {
    root()?;

    let p = Path::new(CUSTOM_MIRRORS);

    if let Some(parent) = p.parent() {
        if !p.is_dir() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(p)?;

    if f.metadata()?.len() == 0 {
        f.write_all(b"# AOSC OS mirrormgr custom mirror config file\n")?;
        f.write_all(b"# Usage: custom_mirror_name: URL\n")?;
        f.write_all(b"# Like: MY_NAS: https://localhost/aosc\n\n")?;
    }

    drop(f);

    let editor = env::var("EDITOR").unwrap_or("nano".to_string());
    Command::new(editor).arg(p).spawn()?.wait()?;

    Ok(())
}
