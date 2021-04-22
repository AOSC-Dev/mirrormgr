use anyhow::{anyhow, Result};
use serde_json;
use serde::{Deserialize, Serialize};
use std::fs;

mod cli;

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const REPO_DATA_DIR: &str = "/usr/share/distro-repository-data";

#[derive(Deserialize, Serialize)]
struct Status {
    branch: String,
    component: Vec<String>,
    mirror: Vec<String>
}
fn main() -> Result<()> {
    let app = cli::build_cli().get_matches();
    let status = read_status()?;

    match app.subcommand() {
        ("status", _) => {
            println!("Branch: {}", status.branch);
            for i in status.component {
                println!("component: {}", i);
            }
            for i in status.mirror {
                println!("mirror: {}", i);
            }
        }
        _ => {
            unreachable!()
        }
    }
    Ok(())
}

fn read_status() -> Result<Status> {
    let status = fs::read(STATUS_FILE)?;
    let status: Status = serde_json::from_slice(&status)?;

    Ok(status)
}
