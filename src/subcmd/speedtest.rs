use std::cmp::Ordering;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

use crate::fl;
use crate::utils::url_strip;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use oma_console::console;
use reqwest::blocking::Client;
use sha2::Digest;
use sha2::Sha256;
use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

use crate::utils::distro_and_custom_mirrors;
use crate::SPEEDTEST_FILE_CHECKSUM;

const FILE_SIZE_KIB: f32 = 1024.0;

#[derive(Tabled)]
struct MirrorScore {
    mirror_name: String,
    score: String,
}

impl From<(String, String)> for MirrorScore {
    fn from(value: (String, String)) -> Self {
        MirrorScore {
            mirror_name: value.0,
            score: value.1,
        }
    }
}

pub fn execute() -> Result<()> {
    let mirrors = distro_and_custom_mirrors()?;
    let map = mirrors.list_mirrors();

    let bar = ProgressBar::new(map.len() as u64);
    bar.set_style(
        ProgressStyle::with_template("[{wide_bar:.cyan/blue}] ({pos}/{len})")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut all_score = IndexMap::new();
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("AOSC mirrormgr")
        .build()?;

    for (name, url) in map {
        let score = get_score(&client, &name, &url);

        match score {
            Ok(s) => {
                let score = FILE_SIZE_KIB / s;
                bar.println(
                    console::style(format!("{name}: {}", format_speed(score)))
                        .green()
                        .to_string(),
                );
                all_score.insert(name, score);
            }
            Err(e) => {
                bar.println(console::style(format!("{name}: {e}")).red().to_string());
            }
        }
        bar.inc(1);
    }

    bar.finish_and_clear();

    let all_score = all_score
        .sorted_unstable_by(|_, s1, _, s2| s2.partial_cmp(s1).unwrap_or(Ordering::Equal))
        .map(|(x, y)| (x, format_speed(y)))
        .map(|x| MirrorScore::from(x));

    let mut t = Table::new(all_score);
    t.with(Style::psql());

    println!();
    println!("{t}");

    Ok(())
}

fn get_score(client: &Client, name: &str, url: &str) -> Result<f32> {
    let timer = Instant::now();
    let buf = client
        .get(&format!("{}.repotest", url_strip(url)))
        .send()?
        .bytes()?;

    let mut hasher = Sha256::new();
    hasher.write_all(&buf)?;
    let c = hex::encode(hasher.finalize());
    if c == SPEEDTEST_FILE_CHECKSUM {
        let result_time = timer.elapsed().as_secs_f32();
        return Ok(result_time);
    }

    Err(anyhow!(fl!("mirror-error", mirror = name.to_string())))
}

fn format_speed(score: f32) -> String {
    let mut score = score;
    let mut unit = "KiB/s";
    if score > 1000.0 {
        score /= 1024.0;
        unit = "MiB/s";
    }

    format!("{:.2}{}", score, unit)
}
