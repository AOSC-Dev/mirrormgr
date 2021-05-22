use anyhow::{anyhow, Result};
use attohttpc;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};
use url::Url;

mod cli;

lazy_static! {
    static ref REPO_DATA_DIRECTORY: String = get_repo_data_path();
    static ref REPO_MIRROR_FILE: String = REPO_DATA_DIRECTORY.to_string() + &"mirrors.yml";
    static ref REPO_COMPONENT_FILE: String = REPO_DATA_DIRECTORY.to_string() + &"comps.yml";
    static ref REPO_BRANCH_FILE: String = REPO_DATA_DIRECTORY.to_string() + &"branches.yml";
}

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const APT_SOURCE_FILE: &str = "/etc/apt/sources.list";
const CUSTOM_MIRROR_FILE: &str = "/etc/apt-gen-list/custom_mirror.yml";

#[derive(Deserialize, Serialize)]
struct Status {
    branch: String,
    component: Vec<String>,
    mirror: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize)]
struct BranchItem {
    desc: String,
    suites: Vec<String>,
}

impl Default for Status {
    fn default() -> Self {
        Status {
            branch: "stable".to_string(),
            component: vec!["main".to_string()],
            mirror: vec![("origin".to_string(), "https://repo.aosc.io".to_string())],
        }
    }
}

fn main() -> Result<()> {
    let app = cli::build_cli().get_matches();
    let mut status = read_status()?;

    match app.subcommand() {
        ("status", _) => {
            let mirror_name_list: Vec<String> =
                status.mirror.into_iter().map(|x| x.0).rev().collect();
            println!("Branch: {}", status.branch);
            println!("Component: {}", status.component.join(", "));
            println!("Mirror: {}", mirror_name_list.join(", "));
        }
        ("set-mirror", Some(args)) => {
            set_mirror(args.value_of("INPUT").unwrap(), &mut status)?;
        }
        ("add-mirror", Some(args)) => {
            add_mirror(args, &mut status)?;
        }
        ("remove-mirror", Some(args)) => {
            remove_mirror(args, &mut status)?;
        }
        ("add-component", Some(args)) => {
            add_component(args, &mut status)?;
        }
        ("remove-component", Some(args)) => {
            remove_component(args, status)?;
        }
        ("set-branch", Some(args)) => {
            let new_branch = args.value_of("INPUT").unwrap();
            if read_distro_branches_file()?.get(new_branch).is_some() {
                status.branch = new_branch.to_string();
            } else {
                return Err(anyhow!("Branch undefined or does not exist!"));
            }
            println!("Setting {} as branch", new_branch);
            apply_status(&status, gen_sources_list_string(&status)?)?;
        }
        ("mirrors-speedtest", _) => {
            let mirrors_score_table = get_mirror_score_table()?;
            println!("Mirror    Speed");
            for (mirror_name, score) in mirrors_score_table {
                println!("{:<10}{}s", mirror_name, score);
            }
        }
        ("set-fastest-mirror-to-default", _) => {
            set_fastest_mirror_to_default(status)?;
        }
        ("add-custom-mirror", Some(args)) => {
            let custom_mirror_args: Vec<&str> = args.values_of("INPUT").unwrap().collect();
            add_custom_mirror(custom_mirror_args[0], custom_mirror_args[1])?;
        }
        ("remove-custom-mirror", Some(args)) => {
            let custom_mirror_args = args.values_of("INPUT").unwrap();
            for entry in custom_mirror_args {
                remove_custom_mirror(entry)?;
            }
        }
        _ => {
            unreachable!()
        }
    }

    Ok(())
}

fn get_repo_data_path() -> String {
    let not_local_directory_path = String::from("/usr/share/distro-repository-data/");
    if let Ok(v) = fs::metadata(&not_local_directory_path) {
        if v.is_dir() {
            return not_local_directory_path;
        }
    }

    return String::from("/usr/local/share/distro-repository-data/");
}

fn set_fastest_mirror_to_default(mut status: Status) -> Result<(), anyhow::Error> {
    let mirrors_score_table = get_mirror_score_table()?;
    println!(
        "Fastest mirror: {}, speed: {}s, Setting {} as default mirror ...",
        mirrors_score_table[0].0, mirrors_score_table[0].1, mirrors_score_table[0].0
    );
    set_mirror(mirrors_score_table[0].0.as_str(), &mut status)?;

    Ok(())
}

fn get_mirror_score_table() -> Result<Vec<(String, f32)>, anyhow::Error> {
    let mut mirrors_score_table = Vec::new();
    let mirrors_hashmap = read_distro_mirrors_file()?;
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(50);
    for (index, mirror_name) in mirrors_hashmap.keys().enumerate() {
        bar.set_message(format!(
            "Benchmarking {} ({}/{}) ...",
            mirror_name,
            index,
            mirrors_hashmap.len()
        ));
        if let Ok(score) = get_mirror_speed_score(mirror_name.as_str()) {
            mirrors_score_table.push((mirror_name.to_owned(), score));
        } else {
            warn!("Failed to test mirror: {}!", mirror_name);
        }
    }
    bar.finish_and_clear();
    mirrors_score_table.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    if mirrors_score_table.len() == 0 {
        return Err(anyhow!(
            "Get All mirror failed! Please check your network connection!"
        ));
    }
    Ok(mirrors_score_table)
}

fn set_mirror(new_mirror: &str, status: &mut Status) -> Result<(), anyhow::Error> {
    status.mirror = vec![(new_mirror.to_string(), get_mirror_url(new_mirror)?)];
    println!("Setting {} as mirror!", new_mirror);
    apply_status(&*status, gen_sources_list_string(&*status)?)?;

    Ok(())
}

fn remove_mirror(args: &clap::ArgMatches, status: &mut Status) -> Result<(), anyhow::Error> {
    if status.mirror.len() == 1 {
        return Err(anyhow!(
            "You only have one mirror left, refusing to remove!"
        ));
    }
    let entry: Vec<&str> = args.values_of("INPUT").unwrap().collect();
    for i in &entry {
        if let Some(index) = status.mirror.iter().position(|v| &v.0 == i) {
            status.mirror.remove(index);
        } else {
            warn!("Cannot find mirror: {}.", i);
        }
    }
    println!("Removing {} from sources.list ...", entry.join(", "));
    apply_status(&*status, gen_sources_list_string(&status)?)?;

    Ok(())
}

fn add_mirror(args: &clap::ArgMatches, status: &mut Status) -> Result<(), anyhow::Error> {
    let entry: Vec<&str> = args.values_of("INPUT").unwrap().collect();
    println!("Adding mirror {} to sources.list ...", entry.join(", "));
    for i in &entry {
        let mirror_url = get_mirror_url(i)?;
        if status
            .mirror
            .contains(&(i.to_string(), mirror_url.to_owned()))
        {
            warn!("Mirror {} already enabled!", i);
        } else {
            status.mirror.push((i.to_string(), mirror_url));
        }
    }
    apply_status(&*status, gen_sources_list_string(&status)?)?;

    Ok(())
}

fn add_custom_mirror(mirror_name: &str, mirror_url: &str) -> Result<()> {
    let mut custom_mirror_data = read_custom_mirror()?;
    if Url::parse(mirror_url).is_err() {
        return Err(anyhow!("mirror_url is not a URL!"));
    }
    if custom_mirror_data.get(mirror_name).is_none() {
        custom_mirror_data.insert(mirror_name.to_string(), mirror_url.to_string());
    } else {
        return Err(anyhow!("Custom mirror {} already exists!", mirror_name));
    }
    println!(
        "Adding custom mirror {} to {}",
        mirror_name, CUSTOM_MIRROR_FILE
    );
    fs::write(
        CUSTOM_MIRROR_FILE,
        serde_yaml::to_string(&custom_mirror_data)?,
    )?;

    Ok(())
}

fn remove_custom_mirror(mirror_name: &str) -> Result<()> {
    let mut custom_mirror = read_custom_mirror()?;
    if custom_mirror.get(mirror_name).is_none() {
        return Err(anyhow!("Custom mirror {} does not exist!", mirror_name));
    } else {
        custom_mirror.remove(mirror_name);
    }
    println!(
        "Removing custom mirror {} from {}",
        mirror_name, CUSTOM_MIRROR_FILE
    );
    fs::write(CUSTOM_MIRROR_FILE, serde_yaml::to_string(&custom_mirror)?)?;

    Ok(())
}

fn read_custom_mirror() -> Result<HashMap<String, String>> {
    if let Ok(file_data) = fs::read(CUSTOM_MIRROR_FILE) {
        return Ok(serde_yaml::from_slice(&file_data)?);
    }
    fs::create_dir_all("/etc/apt-gen-list")?;
    fs::File::create(CUSTOM_MIRROR_FILE)?;

    Ok(read_custom_mirror()?)
}

fn remove_component(args: &clap::ArgMatches, mut status: Status) -> Result<(), anyhow::Error> {
    let entry: Vec<&str> = args.values_of("INPUT").unwrap().collect();
    if !entry.contains(&"main") {
        for i in &entry {
            if let Some(index) = status.component.iter().position(|v| v == i) {
                status.component.remove(index);
            } else {
                warn!("Component {} is not enabled or does not exist.", &i);
            }
        }
    } else {
        return Err(anyhow!("Refusing to remove essential component \"main\"."));
    }
    println!("Disabling component {} ...", entry.join(", "));
    apply_status(&status, gen_sources_list_string(&status)?)?;

    Ok(())
}

fn add_component(args: &clap::ArgMatches, status: &mut Status) -> Result<(), anyhow::Error> {
    let entry: Vec<&str> = args.values_of("INPUT").unwrap().collect();
    for i in &entry {
        if status.component.contains(&i.to_string()) {
            warn!("Component {} is already enabled.", &i);
        } else if read_distro_components_file()?.get(&i.to_string()).is_some() {
            status.component.push(i.to_string());
        } else {
            return Err(anyhow!("Component {} does not exist.", &i));
        }
    }
    println!("Enabling component {} ...", entry.join(", "));
    apply_status(&status, gen_sources_list_string(&status)?)?;

    Ok(())
}

fn read_status() -> Result<Status> {
    let status = match fs::read(STATUS_FILE) {
        Ok(v) => v,
        Err(_) => {
            fs::create_dir_all("/var/lib/apt/gen")?;
            fs::File::create(STATUS_FILE)?;
            fs::read(STATUS_FILE)?
        }
    };
    let status: Status = serde_json::from_slice(&status).unwrap_or_default();

    Ok(status)
}

fn read_distro_components_file() -> Result<HashMap<String, String>> {
    if let Ok(file_data) = fs::read(REPO_COMPONENT_FILE.to_string()) {
        let file_data: HashMap<String, String> = serde_yaml::from_slice(&file_data)?;
        return Ok(file_data);
    }

    Err(anyhow!(
        "Could not find repository data, please check your aosc-os-repository-data installation."
    ))
}

fn read_distro_branches_file() -> Result<HashMap<String, BranchItem>> {
    return Ok(serde_yaml::from_slice(&fs::read(
        REPO_BRANCH_FILE.to_string(),
    )?)?);
}

fn read_distro_mirrors_file() -> Result<HashMap<String, HashMap<String, String>>> {
    return Ok(serde_yaml::from_slice(&fs::read(
        REPO_MIRROR_FILE.to_string(),
    )?)?);
}

fn apply_status(status: &Status, source_list_str: String) -> Result<()> {
    println!("Writing to apt-gen-list status file ...");
    fs::write(
        STATUS_FILE,
        format!("{} \n", serde_json::to_string(&status)?),
    )?;
    println!("Writing /etc/apt/sources.list ...");
    fs::write(APT_SOURCE_FILE, source_list_str)?;

    Ok(())
}

fn gen_sources_list_string(status: &Status) -> Result<String> {
    let mut result = String::from("# Generated by apt-gen-list. DO NOT EDIT THIS FILE! \n");
    let directory_name = get_directory_name()?;
    for (_, mirror_url) in &status.mirror {
        let debs_url = Url::parse(&mirror_url)?.join(directory_name)?;
        for branch in get_branch_suites(&status.branch)? {
            result += &format!(
                "deb {} {} {} \n",
                debs_url.as_str(),
                branch,
                status.component.join(" ")
            );
        }
    }

    Ok(result)
}

fn get_mirror_speed_score(mirror_name: &str) -> Result<f32> {
    let timer = Instant::now();
    let download_url = Url::parse(get_mirror_url(mirror_name)?.as_str())?
        .join("misc/u-boot-sunxi-with-spl.bin")?;
    if attohttpc::get(download_url)
        .timeout(Duration::from_secs(10))
        .send()?
        .is_success()
    {
        return Ok(timer.elapsed().as_secs_f32());
    }

    Err(anyhow!(
        "Failed to download from {}, please check your network connection!",
        mirror_name
    ))
}

fn get_mirror_url(mirror_name: &str) -> Result<String> {
    let mirror_url = read_distro_mirrors_file()?
        .get(mirror_name)
        .ok_or_else(|| anyhow!("Cannot find mirror!"))?
        .get("url")
        .ok_or_else(|| anyhow!("Cannot get URL!"))?
        .to_owned();

    Ok(mirror_url)
}

fn get_branch_suites(branch_name: &str) -> Result<Vec<String>> {
    let branch_suites = read_distro_branches_file()?
        .get(branch_name)
        .ok_or_else(|| anyhow!("Cannot find branch!"))?
        .suites
        .to_owned();

    Ok(branch_suites)
}

fn get_directory_name() -> Result<&'static str> {
    if let Ok(file_data) = fs::read_to_string("/etc/os-release") {
        let os_release: Vec<&str> = file_data.split("\n").into_iter().collect();
        for i in os_release {
            if i.starts_with("NAME=") {
                return match &i[5..] {
                    "\"AOSC OS\"" => Ok("debs"),
                    "\"AOSC OS/Retro\"" => Ok("debs-retro"),
                    _ => Ok(""),
                };
            }
        }
    }

    Err(anyhow!("Could not find /etc/os-release! or brokend!"))
}
