use anyhow::{anyhow, Result};
use futures::future;
use indexmap::{indexmap, IndexMap};
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use log::warn;
use os_release::OsRelease;
use owo_colors::OwoColorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tokio::runtime::Builder;
use url::Url;

mod cli;
mod i18n;

use i18n::I18N_LOADER;

lazy_static! {
    static ref REPO_DATA_DIRECTORY: PathBuf = get_repo_data_path();
    static ref REPO_MIRROR_FILE: PathBuf = REPO_DATA_DIRECTORY.join("mirrors.yml");
    static ref REPO_COMPONENT_FILE: PathBuf = REPO_DATA_DIRECTORY.join("comps.yml");
    static ref REPO_BRANCH_FILE: PathBuf = REPO_DATA_DIRECTORY.join("branches.yml");
}

const STATUS_FILE: &str = "/var/lib/apt/gen/status.json";
const APT_SOURCE_FILE: &str = "/etc/apt/sources.list";
const CUSTOM_MIRROR_FILE: &str = "/etc/apt-gen-list/custom_mirror.yml";
const OMAKASE_CONFIG_FILE: &str = "/etc/omakase/config.toml";
const SPEEDTEST_FILE_CHECKSUM: &str = "399c1475c74b6534fe1c272035fce276bf587989";
const DOWNLOAD_PATH: &str = "misc/u-boot-sunxi-with-spl.bin";
const SPEEDTEST_FILE_SIZE_KIB: f32 = 389.106_45;

#[derive(Deserialize, Serialize)]
struct Status {
    branch: String,
    component: Vec<String>,
    mirror: IndexMap<String, String>,
}

#[cfg(feature = "aosc")]
#[derive(Deserialize)]
struct OldStatus {
    branch: String,
    component: Vec<String>,
    mirror: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct BranchInfo {
    desc: String,
    suites: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct MirrorInfo {
    desc: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OmakaseConfig {
    #[serde(flatten)]
    other: toml::Value,
    repo: HashMap<String, OmakaseMirror>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OmakaseMirror {
    url: String,
    distribution: String,
    components: Vec<String>,
    keys: Vec<String>,
}

struct FrontendStatus {
    has_apt: bool,
    has_oma: bool,
}

type BranchesData = HashMap<String, BranchInfo>;
type MirrorsData = IndexMap<String, MirrorInfo>;
type ComponentData = HashMap<String, String>;
type CustomMirrorData = HashMap<String, String>;

#[cfg(feature = "aosc")]
impl Default for Status {
    fn default() -> Self {
        Status {
            branch: "stable".to_string(),
            component: vec!["main".to_string()],
            mirror: indexmap! {"origin".to_string() => "https://repo.aosc.io".to_string()},
        }
    }
}

fn main() -> Result<()> {
    let app = cli::build_cli().get_matches();
    let mut status = read_status()?;

    match app.subcommand() {
        ("status", _) => {
            let mirror_list = status
                .mirror
                .into_iter()
                .map(|(mirror_name, mirror_url)| format!("{} ({})", mirror_name, mirror_url))
                .collect::<Vec<String>>();
            println!("{}", fl!("branch", branch = status.branch));
            println!("{}", fl!("component", comp = status.component.join(", ")));
            println!("{}", fl!("mirror", mirror = mirror_list.join(", ")));
        }
        ("set-mirror", Some(args)) => {
            set_mirror(args.value_of("MIRROR").unwrap(), &mut status)?;
        }
        ("add-mirror", Some(args)) => {
            add_mirror(args.values_of("MIRROR").unwrap().collect(), &mut status)?;
        }
        ("remove-mirror", Some(args)) => {
            remove_mirror(args, &mut status)?;
        }
        ("add-component", Some(args)) => {
            add_component(args, &mut status)?;
        }
        ("remove-component", Some(args)) => {
            remove_component(args.values_of("COMPONENT").unwrap().collect(), status)?;
        }
        ("set-branch", Some(args)) => {
            let new_branch = args.value_of("BRANCH").unwrap();
            if read_distro_file::<BranchesData, _>(&*REPO_BRANCH_FILE)?
                .get(new_branch)
                .is_some()
            {
                status.branch = new_branch.to_string();
            } else {
                return Err(anyhow!(fl!("branch-not-found")));
            }
            println!("{}", fl!("set-branch", branch = new_branch));
            apply_status(&status)?;
        }
        ("speedtest", Some(args)) => {
            let mirrors_score_table = get_mirror_score_table(args.is_present("parallel"))?;
            println!(" {:<20}Speed", "Mirror");
            println!(" {:<20}---", "---");
            for (mirror_name, score) in mirrors_score_table {
                println!(" {:<20}{}", mirror_name, score);
            }
        }
        ("set-fastest-mirror-as-default", _) => {
            set_fastest_mirror_as_default(status)?;
        }
        ("add-custom-mirror", Some(args)) => {
            let custom_mirror_name = args.value_of("MIRROR_NAME").unwrap();
            let custom_mirror_url = args.value_of("MIRROR_URL").unwrap();
            add_custom_mirror(custom_mirror_name, custom_mirror_url)?;
            if args.is_present("also-set-mirror") {
                set_mirror(custom_mirror_name, &mut status)?;
            } else if args.is_present("also-add-mirror") {
                add_mirror(vec![custom_mirror_name], &mut status)?;
            }
        }
        ("remove-custom-mirror", Some(args)) => {
            let custom_mirror_args = args.values_of("MIRROR").unwrap();
            for entry in custom_mirror_args {
                remove_custom_mirror(entry)?;
            }
        }
        ("reset-mirror", _) => {
            #[cfg(feature = "aosc")]
            {
                status = Status::default();
                apply_status(&status)?;
            }
            #[cfg(not(feature = "aosc"))]
            {
                unreachable!();
            }
        }
        ("list-mirrors", _) => {
            get_available_mirror(&status)?;
        }
        _ => {
            unreachable!()
        }
    }

    Ok(())
}

fn get_repo_data_path() -> PathBuf {
    let not_local_directory_path = PathBuf::from("/usr/share/distro-repository-data/");
    if not_local_directory_path.is_dir() {
        not_local_directory_path
    } else {
        PathBuf::from("/usr/local/share/distro-repository-data/")
    }
}

fn set_fastest_mirror_as_default(mut status: Status) -> Result<()> {
    let mirrors_score_table = get_mirror_score_table(false)?;
    println!(
        "{}",
        fl!(
            "set-fastest-mirror",
            mirror = mirrors_score_table[0].0.clone(),
            speed = mirrors_score_table[0].1.clone()
        )
    );
    set_mirror(mirrors_score_table[0].0.as_str(), &mut status)?;

    Ok(())
}

fn get_mirror_score_table(is_parallel: bool) -> Result<Vec<(String, String)>> {
    let mirrors_indexmap = read_distro_file::<MirrorsData, _>(&*REPO_MIRROR_FILE)?;
    let bar = ProgressBar::new_spinner();
    let mut mirrors_score_table = if is_parallel {
        bar.set_message(fl!("test-mirrors"));
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .unwrap();
        let client = reqwest::Client::new();
        runtime.block_on(async move {
            let task = mirrors_indexmap
                .keys()
                .into_iter()
                .map(|x| get_mirror_speed_score_parallel(x, &client))
                .collect::<Vec<_>>();
            bar.enable_steady_tick(50);
            let results = future::join_all(task).await;
            let mut result = Vec::new();
            for (index, mirror_name) in mirrors_indexmap.keys().enumerate() {
                if let Ok(time) = results[index] {
                    result.push((mirror_name.to_owned(), SPEEDTEST_FILE_SIZE_KIB / time));
                }
            }

            result
        })
    } else {
        let mut result = Vec::new();
        for (index, mirror_name) in mirrors_indexmap.keys().enumerate() {
            bar.set_message(fl!(
                "test-mirrors-sync",
                count = index,
                all = mirrors_indexmap.len()
            ));
            bar.enable_steady_tick(50);
            if let Ok(time) = get_mirror_speed_score(mirror_name) {
                result.push((mirror_name.to_owned(), SPEEDTEST_FILE_SIZE_KIB / time));
            }
        }

        result
    };
    mirrors_score_table.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    if mirrors_score_table.is_empty() {
        return Err(anyhow!(fl!("mirror-test-failed")));
    }
    let mut result = Vec::new();
    for (mirror_name, mut score) in mirrors_score_table {
        let mut unit = "KiB/s";
        if score > 1000.0 {
            score /= 1024.0;
            unit = "MiB/s";
        }
        result.push((mirror_name.to_owned(), format!("{:.2}{}", score, unit)));
    }

    Ok(result)
}

fn get_available_mirror(status: &Status) -> Result<()> {
    let mut result_table = IndexMap::new();
    let distro_mirror = read_distro_file::<MirrorsData, _>(&*REPO_MIRROR_FILE)?;
    for (mirror_name, mirror_info) in distro_mirror {
        result_table.insert(mirror_name, mirror_info.desc);
    }
    if let Ok(custom_mirror) = read_distro_file::<CustomMirrorData, _>(CUSTOM_MIRROR_FILE) {
        for (mirror_name, mirror_url) in custom_mirror {
            result_table.insert(mirror_name, format!("{} {}", fl!("custom"), mirror_url));
        }
    }
    result_table.sort_keys();
    println!("  {}\n", fl!("mirror-list-explain"));
    for (mirror_name, mirror_info) in &result_table {
        let s = format!("{:<10}{}", mirror_name, mirror_info);
        if status.mirror.get(mirror_name).is_some() {
            println!("* {}", s.cyan().bold());
            continue;
        }
        println!("  {}", s);
    }

    Ok(())
}

fn set_mirror(new_mirror: &str, status: &mut Status) -> Result<()> {
    status.mirror = indexmap! {new_mirror.to_string() => get_mirror_url(new_mirror)?};
    println!("{}", fl!("set-mirror", mirror = new_mirror));
    apply_status(&*status)?;

    Ok(())
}

fn remove_mirror(args: &clap::ArgMatches, status: &mut Status) -> Result<()> {
    if status.mirror.len() == 1 {
        return Err(anyhow!(fl!("no-delete-only-mirror")));
    }
    let entry: Vec<&str> = args.values_of("MIRROR").unwrap().collect();
    for i in &entry {
        if status.mirror.get(i.to_owned()).is_some() {
            status.mirror.remove(i.to_owned());
        } else {
            warn!("{}", fl!("mirror-not-found", mirror = i.to_string()));
        }
    }
    println!("{}", fl!("remove-mirror", mirror = entry.join(", ")));
    apply_status(&*status)?;

    Ok(())
}

fn add_mirror(entry: Vec<&str>, status: &mut Status) -> Result<()> {
    println!("{}", fl!("add-mirror", mirror = entry.join(", ")));
    for i in entry {
        let mirror_url = get_mirror_url(i)?;
        if status.mirror.get(i).is_some() {
            warn!("{}", fl!("mirror-already-enabled", mirror = i.to_string()));
        } else {
            status.mirror.insert(i.to_string(), mirror_url);
        }
    }
    apply_status(&*status)?;

    Ok(())
}

fn add_custom_mirror(mirror_name: &str, mirror_url: &str) -> Result<()> {
    if read_distro_file::<MirrorsData, _>(&*REPO_MIRROR_FILE)?
        .get(mirror_name)
        .is_some()
    {
        return Err(anyhow!(fl!("custom-mirror-name-error")));
    }
    let url = Url::parse(mirror_url).map_err(|_| anyhow!(fl!("custom-mirror-not-url")))?;
    #[cfg(feature = "aosc")]
    {
        if vec![Some(&"debs"), Some(&"debs-retro")].contains(
            &url.path_segments()
                .map(|c| c.collect::<Vec<_>>())
                .ok_or_else(|| anyhow!(fl!("custom-mirror-not-url")))?
                .last(),
        ) {
            return Err(anyhow!(fl!("debs-path-in-url")));
        }
        println!("{}", fl!("trying-get-mirror"));
        let get_mirror = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?
            .get(url.join("pool/stable/InRelease")?)
            .send();
        if get_mirror.is_err() {
            return Err(anyhow!(fl!("download-mirror-metadata-failed")));
        }
    }
    #[cfg(not(feature = "aosc"))]
    {
        if url.scheme().is_empty() {
            return Err(anyhow!(fl!("custom-mirror-not-url")));
        }
    }
    println!(
        "{}",
        fl!(
            "add-custom-mirror",
            mirror = mirror_name,
            path = CUSTOM_MIRROR_FILE
        )
    );
    let mut custom_mirror_data = match read_distro_file::<CustomMirrorData, _>(CUSTOM_MIRROR_FILE) {
        Ok(v) => v,
        Err(_) => {
            fs::create_dir_all("/etc/apt-gen-list")?;
            fs::File::create(CUSTOM_MIRROR_FILE)?;
            let mut result = HashMap::new();
            result.insert(mirror_name.to_string(), url.to_string());
            fs::write(CUSTOM_MIRROR_FILE, serde_yaml::to_string(&result)?)?;

            result
        }
    };
    if custom_mirror_data.get(mirror_name).is_none() {
        custom_mirror_data.insert(mirror_name.to_string(), url.to_string());
    } else {
        warn!(
            "{}",
            fl!("custom-mirror-already-exist", mirror = mirror_name)
        );
    }
    fs::write(
        CUSTOM_MIRROR_FILE,
        serde_yaml::to_string(&custom_mirror_data)?,
    )?;

    Ok(())
}

fn remove_custom_mirror(mirror_name: &str) -> Result<()> {
    let mut custom_mirror = read_distro_file::<CustomMirrorData, _>(CUSTOM_MIRROR_FILE)?;
    if custom_mirror.get(mirror_name).is_none() {
        return Err(anyhow!(fl!(
            "custom-mirror-not-found",
            mirror = mirror_name
        )));
    } else {
        custom_mirror.remove(mirror_name);
    }
    println!(
        "{}",
        fl!(
            "remove-custom-mirror",
            mirror = mirror_name,
            path = CUSTOM_MIRROR_FILE
        )
    );
    fs::write(CUSTOM_MIRROR_FILE, serde_yaml::to_string(&custom_mirror)?)?;

    Ok(())
}

fn remove_component(entry: Vec<&str>, mut status: Status) -> Result<()> {
    if !entry.contains(&"main") {
        for i in &entry {
            if let Some(index) = status.component.iter().position(|v| v == i) {
                status.component.remove(index);
            } else {
                warn!("{}", fl!("comp-not-enabled", comp = i.to_string()));
            }
        }
    } else {
        return Err(anyhow!(fl!("no-delete-only-comp")));
    }
    println!("{}", fl!("disable-comp", comp = entry.join(", ")));
    apply_status(&status)?;

    Ok(())
}

fn add_component(args: &clap::ArgMatches, status: &mut Status) -> Result<()> {
    let entries: Vec<&str> = args.values_of("COMPONENT").unwrap().collect();
    for entry in entries.iter() {
        let entry_str = entry.to_string();
        if status.component.contains(&entry_str) {
            warn!("{}", fl!("comp-already-enabled", comp = entry_str.clone()));
        } else if read_distro_file::<ComponentData, _>(&*REPO_COMPONENT_FILE)?
            .get(&entry_str)
            .is_some()
        {
            status.component.push(entry_str);
        } else {
            return Err(anyhow!(fl!("comp-not-found", comp = entry_str)));
        }
    }
    println!("{}", fl!("enable-comp", comp = entries.join(", ")));
    apply_status(status)?;

    Ok(())
}

fn read_status() -> Result<Status> {
    if !Path::new(STATUS_FILE).is_file() && !is_root() {
        panic!("{}", fl!("status-file-not-found", path = STATUS_FILE))
    }
    match fs::read(STATUS_FILE) {
        Ok(file) => match serde_json::from_slice(&file) {
            Ok(status) => Ok(status),
            Err(_) => {
                #[cfg(feature = "aosc")]
                {
                    if !is_root() {
                        return Err(anyhow!("{}", fl!("status-file-read-error")));
                    }
                    let status = trans_to_new_status_config(file).unwrap_or_default();
                    fs::write(STATUS_FILE, serde_json::to_string(&status)?)?;

                    Ok(status)
                }
                #[cfg(not(feature = "aosc"))]
                {
                    panic!("{}", fl!("status-file-read-error"));
                }
            }
        },
        Err(_) => {
            #[cfg(feature = "aosc")]
            {
                fs::create_dir_all("/var/lib/apt/gen")?;
                let status = Status::default();
                fs::write(STATUS_FILE, serde_json::to_string(&status)?)?;

                Ok(status)
            }
            #[cfg(not(feature = "aosc"))]
            {
                panic!("{}", fl!("status-file-read-error"));
            }
        }
    }
}

fn is_root() -> bool {
    nix::unistd::geteuid().is_root()
}

#[cfg(feature = "aosc")]
fn trans_to_new_status_config(file: Vec<u8>) -> Result<Status> {
    let status: OldStatus = serde_json::from_slice(&file)?;
    let mut new_mirror: IndexMap<String, String> = IndexMap::new();
    for mirror_name in &status.mirror {
        new_mirror.insert(mirror_name.to_string(), get_mirror_url(mirror_name)?);
    }

    Ok(Status {
        branch: status.branch,
        mirror: new_mirror,
        component: status.component,
    })
}

fn read_distro_file<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(file: P) -> Result<T> {
    Ok(serde_yaml::from_slice(&fs::read(file)?)?)
}

fn apply_status(status: &Status) -> Result<()> {
    println!("{}", fl!("write-status"));
    fs::write(
        STATUS_FILE,
        format!("{}\n", serde_json::to_string(&status)?),
    )?;
    let frontend_status = get_frontend_status();
    #[cfg(feature = "aosc")]
    {
        println!("{}", fl!("run-atm-refresh"));
        Command::new("atm")
            .arg("refresh")
            .spawn()?
            .wait_with_output()?;
    }
    if frontend_status.has_apt {
        let source_list_str = gen_sources_list_string(status)?;
        println!("{}", fl!("write-sources"));
        fs::write(APT_SOURCE_FILE, source_list_str)?;
        println!("{}", fl!("run-apt"));
        Command::new("apt-get")
            .arg("update")
            .spawn()?
            .wait_with_output()?;
    }
    if frontend_status.has_oma {
        let omakase_config_str = gen_omakase_config_string(status)?;
        println!("{}", fl!("write-omakase-config"));
        fs::write(OMAKASE_CONFIG_FILE, omakase_config_str)?;
        println!("{}", fl!("run-oma"));
        Command::new("oma")
            .arg("refresh")
            .spawn()?
            .wait_with_output()?;
    }

    Ok(())
}

fn get_frontend_status() -> FrontendStatus {
    let mut has_oma = false;
    let mut has_apt = false;
    let which_oma_output = Command::new("which").arg("oma").output();
    let which_apt_output = Command::new("which").arg("apt").output();
    if let Ok(which_oma_output) = which_oma_output {
        if which_oma_output.status.success() {
            has_oma = true;
        }
    }
    if let Ok(which_apt_output) = which_apt_output {
        if which_apt_output.status.success() {
            has_apt = true;
        }
    }

    FrontendStatus { has_apt, has_oma }
}

fn gen_sources_list_string(status: &Status) -> Result<String> {
    let mut result = format!("{}\n", fl!("generated"));
    let directory_name = get_directory_name();
    for (_, mirror_url) in &status.mirror {
        let debs_url = Url::parse(mirror_url)?.join(directory_name)?;
        for branch in get_branch_suites(&status.branch)? {
            result.push_str(&format!(
                "deb {} {} {}\n",
                debs_url.as_str(),
                branch,
                status.component.join(" ")
            ));
        }
    }

    Ok(result)
}

fn gen_omakase_config_string(status: &Status) -> Result<String> {
    let mut config_file = std::fs::File::open(OMAKASE_CONFIG_FILE)?;
    let mut buf = vec![];
    config_file.read_to_end(&mut buf)?;
    let mut omakase_config: OmakaseConfig = toml::from_slice(&buf)?;
    let repo_list = omakase_config.repo;
    let mut new_repo_map = HashMap::new();
    fn format_url(url: &str) -> String {
        if url.ends_with("/") {
            url.to_string()
        } else {
            format!("{}/", url)
        }
    }
    for (name, url) in &status.mirror {
        let url = format_url(url);
        new_repo_map.insert(
            format!("repo.{}", name),
            OmakaseMirror {
                url: format!("{}debs", url),
                distribution: status.branch.to_owned(),
                components: status.component.to_owned(),
                keys: vec!["aosc.gpg".to_string()],
            },
        );
    }
    for (name, repo) in repo_list {
        if new_repo_map.get(&name).is_some() {
            continue;
        }
        if let Some((_, url)) = status.mirror.first() {
            let url = format_url(url);
            new_repo_map.insert(
                name.to_string(),
                OmakaseMirror {
                    url: format!("{}debs", url),
                    distribution: repo.distribution,
                    components: repo.components,
                    keys: repo.keys,
                },
            );
        }
    }
    omakase_config.repo = new_repo_map;

    Ok(toml::to_string(&omakase_config)?)
}

async fn get_mirror_speed_score_parallel(mirror_name: &str, client: &Client) -> Result<f32> {
    let download_url = Url::parse(&get_mirror_url(mirror_name)?)?.join(DOWNLOAD_PATH)?;
    let timer = Instant::now();
    let file = client
        .get(download_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .bytes()
        .await;
    let result_time = timer.elapsed().as_secs_f32();
    if let Ok(file) = file {
        if Sha1::from(file).digest().to_string() == SPEEDTEST_FILE_CHECKSUM {
            return Ok(result_time);
        }
    }

    Err(anyhow!(fl!("mirror-error", mirror = mirror_name)))
}

fn get_mirror_speed_score(mirror_name: &str) -> Result<f32> {
    let download_url = Url::parse(&get_mirror_url(mirror_name)?)?.join(DOWNLOAD_PATH)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let timer = Instant::now();
    let file = client.get(download_url).send()?.bytes()?;
    let result_time = timer.elapsed().as_secs_f32();
    if Sha1::from(file).digest().to_string() == SPEEDTEST_FILE_CHECKSUM {
        return Ok(result_time);
    }

    Err(anyhow!(fl!("mirror-error", mirror = mirror_name)))
}

fn get_mirror_url(mirror_name: &str) -> Result<String> {
    if let Some(mirror_info) =
        read_distro_file::<MirrorsData, _>(&*REPO_MIRROR_FILE)?.get(mirror_name)
    {
        return Ok(mirror_info.url.to_owned());
    } else if let Some(mirror_url) =
        read_distro_file::<CustomMirrorData, _>(CUSTOM_MIRROR_FILE)?.get(mirror_name)
    {
        return Ok(mirror_url.to_owned());
    }

    Err(anyhow!(fl!("mirror-not-found", mirror = mirror_name)))
}

fn get_branch_suites(branch_name: &str) -> Result<Vec<String>> {
    Ok(read_distro_file::<BranchesData, _>(&*REPO_BRANCH_FILE)?
        .get(branch_name)
        .ok_or_else(|| anyhow!(fl!("branch-data-error")))?
        .suites
        .to_owned())
}

fn get_directory_name() -> &'static str {
    match OsRelease::new().unwrap().name.as_str() {
        "AOSC OS" => "debs",
        "AOSC OS/Retro" => "debs-retro",
        _ => "",
    }
}
