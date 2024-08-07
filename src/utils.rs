use eyre::{anyhow, bail, Result};
use rustix::process;
use std::{
    borrow::Cow,
    fs::{self, File},
    path::Path,
    process::{exit, Command},
};

use crate::{
    fl,
    mgr::{CustomMirrors, DistroConfig, Mirrors},
    CUSTOM_MIRRORS, MIRRORS_PATH,
};

pub fn create_status<P: AsRef<Path>>(status: P) -> Result<File> {
    let status = status.as_ref();
    if let Some(parent) = status.parent() {
        if !parent.is_dir() {
            fs::create_dir_all(parent)?;
        }
    } else {
        bail!("Unexpected path: {}", status.display())
    }

    let f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(status)?;

    Ok(f)
}

#[cfg(any(feature = "oma-refresh", feature = "oma-refresh-aosc"))]
pub fn refresh() -> Result<()> {
    use std::{path::PathBuf, sync::{atomic::Ordering, Arc}};

    use dashmap::DashMap;
    use indicatif::{MultiProgress, ProgressBar};
    use oma_console::{
        console,
        pb::{oma_spinner, oma_style_pb},
        writer::{self, Writer},
    };
    use oma_refresh::{
        db::{OmaRefresh, OmaRefreshBuilder, RefreshEvent},
        DownloadEvent,
    };
    use oma_utils::dpkg::dpkg_arch;
    use std::sync::atomic::AtomicBool;
    use tokio::runtime::Builder;

    let mb = Arc::new(MultiProgress::new());
    let pb_map: DashMap<usize, ProgressBar> = DashMap::new();
    let global_is_set = Arc::new(AtomicBool::new(false));

    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

    let client = reqwest::Client::builder().user_agent("oma").build()?;
    let refresh: OmaRefresh = OmaRefreshBuilder {
        source: PathBuf::from("/"),
        limit: Some(4),
        arch: dpkg_arch("/")?,
        download_dir: "/var/lib/apt/lists".into(),
        download_compress: true,
        client: &client,
    }.into();

    let pb = mb.add(ProgressBar::new_spinner());

    let (style, inv) = oma_spinner(false);
    pb.set_style(style);
    pb.enable_steady_tick(inv);
    pb.set_message("Refreshing topics mirror sources file ...");
    pb.finish_and_clear();

    runtime.block_on(refresh.start(
        move |count, event, total| {
            match event {
                RefreshEvent::ClosingTopic(topic_name) => {
                    mb.println(format!("Closing topic {topic_name}")).unwrap();
                }
                RefreshEvent::DownloadEvent(event) => match event {
                    DownloadEvent::ChecksumMismatchRetry { filename, times } => {
                        writer::bar_writeln(
                            |s| {
                                mb.println(s).ok();
                            },
                            &console::style("ERROR").red().bold().to_string(),
                            &format!("{filename} checksum failed, retrying {times} times"),
                        )
                    }
                    DownloadEvent::GlobalProgressSet(size) => {
                        if let Some(pb) = pb_map.get(&0) {
                            pb.set_position(size);
                        }
                    }
                    DownloadEvent::GlobalProgressInc(size) => {
                        if let Some(pb) = pb_map.get(&0) {
                            pb.inc(size);
                        }
                    }
                    DownloadEvent::ProgressDone => {
                        if let Some(pb) = pb_map.get(&(count + 1)) {
                            pb.finish_and_clear();
                        }
                    }
                    DownloadEvent::NewProgressSpinner(msg) => {
                        let (sty, inv) = oma_spinner(false);
                        let pb = mb.insert(count + 1, ProgressBar::new_spinner().with_style(sty));
                        pb.set_message(msg);
                        pb.enable_steady_tick(inv);
                        pb_map.insert(count + 1, pb);
                    }
                    DownloadEvent::NewProgress(size, msg) => {
                        let sty = oma_style_pb(Writer::default(), false);
                        let pb = mb.insert(count + 1, ProgressBar::new(size).with_style(sty));
                        pb.set_message(msg);
                        pb_map.insert(count + 1, pb);
                    }
                    DownloadEvent::ProgressInc(size) => {
                        let pb = pb_map.get(&(count + 1)).unwrap();
                        pb.inc(size);
                    }
                    DownloadEvent::CanNotGetSourceNextUrl(e) => writer::bar_writeln(
                        |s| {
                            mb.println(s).ok();
                        },
                        &console::style("ERROR").red().bold().to_string(),
                        &e,
                    ),
                    DownloadEvent::Done(_) => {
                        return;
                    }
                    DownloadEvent::AllDone => {
                        if let Some(gpb) = pb_map.get(&0) {
                            gpb.finish_and_clear();
                        }
                    }
                    DownloadEvent::ProgressSet(size) => {
                        let pb = pb_map.get(&(count + 1)).unwrap();
                        pb.set_position(size);
                    }
                },
            }

            if let Some(total) = total {
                if !global_is_set.load(Ordering::SeqCst) {
                    let sty = oma_style_pb(Writer::default(), true);
                    let gpb = mb.insert(
                        0,
                        ProgressBar::new(total)
                            .with_style(sty)
                            .with_prefix("Progress"),
                    );
                    pb_map.insert(0, gpb);
                    global_is_set.store(true, Ordering::SeqCst);
                }
            }
        },
        || fl!("generated"),
    ))?;

    Ok(())
}

#[cfg(not(any(feature = "oma-refresh", feature = "oma-refresh-aosc")))]
pub fn refresh() -> Result<()> {
    let cmd = Command::new("apt").arg("update").output()?;

    let code = cmd.status.code();

    if code.map(|x| x != 0).unwrap_or(true) {
        bail!("Apt update exited with status code: {code:?}");
    }

    Ok(())
}

pub fn root() -> Result<()> {
    if process::geteuid().is_root() {
        return Ok(());
    }

    let args = std::env::args().collect::<Vec<_>>();

    let out = Command::new("pkexec")
        .args(args)
        .spawn()
        .and_then(|x| x.wait_with_output())
        .map_err(|e| anyhow!(fl!("execute-pkexec-fail", e = e.to_string())))?;

    exit(
        out.status
            .code()
            .expect("Can not get pkexec oma exit status"),
    );
}

pub fn distro_and_custom_mirrors() -> Result<Mirrors> {
    let mut all_mirrors = Mirrors::from_path(MIRRORS_PATH)?;
    let custom = CustomMirrors::from_path(CUSTOM_MIRRORS);

    if let Ok(custom) = custom {
        all_mirrors.init_custom_mirrors(custom)?;
    }

    Ok(all_mirrors)
}

pub fn url_strip(url: &str) -> Cow<'_, str> {
    if url.ends_with('/') {
        Cow::Borrowed(url)
    } else {
        Cow::Owned(format!("{url}/"))
    }
}
