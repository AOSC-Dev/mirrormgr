use anyhow::{bail, Result};
use std::{
    fs::{self, File},
    path::Path,
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
        .create(true)
        .read(true)
        .write(true)
        .open(status)?;

    Ok(f)
}

#[cfg(any(feature = "oma-refresh", feature = "oma-refresh-aosc"))]
pub fn refresh() -> Result<()> {
    use std::sync::{Arc, atomic::Ordering};

    use dashmap::DashMap;
    use indicatif::{MultiProgress, ProgressBar};
    use oma_console::{
        pb::{oma_spinner, oma_style_pb},
        writer::Writer,
    };
    use oma_refresh::{
        db::{OmaRefresh, RefreshEvent},
        DownloadEvent,
    };
    use tokio::runtime::Builder;
    use std::sync::atomic::AtomicBool;

    use crate::fl;

    let mb = Arc::new(MultiProgress::new());
    let pb_map: DashMap<usize, ProgressBar> = DashMap::new();
    let global_is_set = Arc::new(AtomicBool::new(false));

    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

    runtime.block_on(OmaRefresh::scan(None, true)?.start(
        move |count, event, total| {
            match event {
                RefreshEvent::ClosingTopic(topic_name) => {
                    mb.println(format!("Closing topic {topic_name}")).unwrap();
                }
                RefreshEvent::DownloadEvent(event) => match event {
                    DownloadEvent::ChecksumMismatchRetry { filename, times } => {
                        mb.println(format!(
                            "{filename} checksum failed, retrying {times} times"
                        ))
                        .unwrap();
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
                    DownloadEvent::CanNotGetSourceNextUrl(e) => {
                        mb.println(format!("Error: {e}")).unwrap();
                    }
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
    use std::process::Command;
    let cmd = Command::new("apt").arg("update").output()?;

    let code = cmd.status.code();

    if code.map(|x| x != 0).unwrap_or(true) {
        bail!("Apt update exited with status code: {code:?}");
    }

    Ok(())
}
