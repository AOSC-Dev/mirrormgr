use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml::Value;
use std::fs;
use url::Url;

mod cli;

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const REPO_MIRROR_FILE: &str = "/usr/share/distro-repository-data/mirrors.yml";
const APT_SOURCE_FILE: &str = "/etc/apt/sources.list";

#[derive(Deserialize, Serialize)]
struct Status {
    branch: String,
    component: Vec<String>,
    mirror: String,
}
fn main() -> Result<()> {
    let app = cli::build_cli().get_matches();
    let mut status = read_status()?;

    match app.subcommand() {
        ("status", _) => {
            println!("Branch: {}", status.branch);
            for i in status.component {
                println!("component: {}", i);
            }
            println!("mirror: {}", status.mirror);
        }
        ("set-mirror", Some(args)) => {
            let mirror_options = read_mirrors_option()?;
            let new_mirror = args.value_of("INPUT").unwrap();

            let mirror_url: &str;

            if let Some(v) = mirror_options.get(new_mirror) {
                status.mirror = new_mirror.to_string();
                mirror_url = v.get("url").unwrap().as_str().unwrap();
            } else if let Ok(_) = Url::parse(new_mirror) {
                status.mirror = new_mirror.to_string();
                mirror_url = new_mirror;
            } else {
                return Err(anyhow!("mirror or url isn't available"));
            }

            let result = to_config(mirror_url, &status)?;
            apply_config(&status, result)?;
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

fn read_mirrors_option() -> Result<Value> {
    let mirrors_data = fs::read(REPO_MIRROR_FILE.to_string())?;
    let mirrors_data = serde_yaml::from_slice(&mirrors_data)?;

    Ok(mirrors_data)
}

fn apply_config(status: &Status, source_list_str: String) -> Result<()> {
    fs::write(STATUS_FILE, serde_json::to_string(&status)?)?;
    fs::write(APT_SOURCE_FILE, format!("{} \n", source_list_str))?;
    Ok(())
}

fn to_config(mirror_url: &str, status: &Status) -> Result<String> {
    let mut result =
        format!("deb {}/debs {}", mirror_url, status.branch);
    result = format!("{} {}", result, status.component.join(" "));
    Ok(result)
}
