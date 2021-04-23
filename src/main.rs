use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml::Value;
use std::fs;
use url::Url;

mod cli;

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const REPO_MIRROR_FILE: &str = "/usr/share/distro-repository-data/mirrors.yml";
const REPO_COMPONENT_FILE: &str = "/usr/share/distro-repository-data/comps.yml";
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
            let new_mirror = args.value_of("INPUT").unwrap();
            let mirror_url: String;

            status.mirror = new_mirror.to_string();
            mirror_url = get_mirror_url(new_mirror)?;

            let result = to_config(&mirror_url, &status)?;
            apply_config(&status, result)?;
        }
        ("add-component", Some(args)) => {
            let new_components = args.values_of("INPUT").unwrap();
            let component_options = read_components_option()?;

            for i in new_components {
                if status.component.contains(&i.to_string()) {
                    return Err(anyhow!(format!("{} already exist in component.", &i)));
                } else if component_options.get(i).is_none() {
                    return Err(anyhow!(format!("{} is not option.", &i)));
                } else {
                    status.component.push(i.to_string());
                }
            }

            let mirror_url = get_mirror_url(&status.mirror)?;
            let result = to_config(&mirror_url, &status)?;

            apply_config(&status, result)?;
        }
        ("remove-component", Some(args)) => {
            let remove_components = args.values_of("INPUT").unwrap();

            for i in remove_components {
                if let Some(index) = status.component.iter().position(|v| v == i) {
                    status.component.remove(index);
                } else {
                    return Err(anyhow!(format!(
                        "Component: {} doesn't exist in component.",
                        &i
                    )));
                }
            }

            let mirror_url = get_mirror_url(&status.mirror)?;
            let result = to_config(&mirror_url, &status)?;

            apply_config(&status, result)?;
        }
        _ => {
            unreachable!()
        }
    }
    Ok(())
}

fn new_default_status() -> Status {
    let default_status = Status {
        branch: "stable".to_string(),
        component: vec!["main".to_string()],
        mirror: "origin".to_string(),
    };

    default_status
}

fn read_status() -> Result<Status> {
    let status = fs::read(STATUS_FILE);
    let status = match status {
        Ok(v) => v,
        Err(_) => {
            fs::create_dir_all("/var/lib/apt/gen")?;
            fs::File::create(STATUS_FILE)?;
            fs::read(STATUS_FILE)?
        }
    };
    let status: Result<Status, _> = serde_json::from_slice(&status);
    let status = match status {
        Ok(v) => v,
        Err(_) => new_default_status(),
    };

    Ok(status)
}

fn read_mirrors_option() -> Result<Value> {
    let mirrors_data = fs::read(REPO_MIRROR_FILE);
    if mirrors_data.is_err() {
        return Err(anyhow!(
            "mirrors data not found! Pleease check your aosc-os-repository-data package."
        ));
    }
    let mirrors_data = mirrors_data.unwrap();
    let mirrors_data = serde_yaml::from_slice(&mirrors_data)?;

    Ok(mirrors_data)
}

fn read_components_option() -> Result<Value> {
    let components_data = fs::read(REPO_COMPONENT_FILE);
    if components_data.is_err() {
        return Err(anyhow!(
            "component data not found! Pleease check your aosc-os-repository-data package."
        ));
    }
    let components_data = components_data.unwrap();
    let components_data = serde_yaml::from_slice(&components_data)?;

    Ok(components_data)
}

fn apply_config(status: &Status, source_list_str: String) -> Result<()> {
    fs::write(
        STATUS_FILE,
        format!("{} \n", serde_json::to_string(&status)?),
    )?;
    fs::write(APT_SOURCE_FILE, source_list_str)?;
    Ok(())
}

fn to_config(mirror_url: &str, status: &Status) -> Result<String> {
    let mirror_url = Url::parse(mirror_url)?;
    let debs_url = mirror_url.join("./debs")?;
    let result = format!(
        "deb {} {} {} \n",
        debs_url.as_str(),
        status.branch,
        status.component.join(" ")
    );
    Ok(result)
}

fn get_mirror_url(mirror_name: &str) -> Result<String> {
    if Url::parse(mirror_name).is_ok() {
        return Ok(mirror_name.to_string());
    }
    let mirror_options = read_mirrors_option()?;
    let mirror_url = mirror_options
        .get(mirror_name)
        .ok_or_else(|| anyhow!("mirror doesn't exist!"))?
        .get("url")
        .ok_or_else(|| anyhow!("No url found on value!"))?
        .as_str()
        .ok_or_else(|| anyhow!("Url isn't String!"))?
        .to_owned();

    Ok(mirror_url)
}
