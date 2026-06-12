use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::config::{self, Config};
use crate::sync::trigger::TriggerState;

pub fn run(watch: bool) -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;
    let dir = Path::new(&cfg.freq_db_dir);

    let state_path = dir.join(".trigger-state.yaml");
    let mut state = TriggerState::load(&state_path.to_string_lossy())?;

    if watch {
        run_watch(&cfg, &mut state, &state_path)
    } else {
        run_once(&cfg, &mut state, &state_path)
    }
}

fn run_once(cfg: &Config, state: &mut TriggerState, state_path: &Path) -> Result<()> {
    if !state.should_sync() {
        let next = state
            .next_sync()
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".into());
        log::info!("cooldown active, next sync at {next}");
        return Ok(());
    }

    TriggerState::run_cycle(cfg)?;
    state.mark_synced();
    state
        .save(&state_path.to_string_lossy())
        .context("failed to save trigger state")?;
    Ok(())
}

fn run_watch(cfg: &Config, state: &mut TriggerState, state_path: &Path) -> Result<()> {
    log::info!("starting daemon in watch mode");

    let rime_dir = cfg.rime_user_dir();
    let (tx, rx) = mpsc::channel();

    let watch_tx = tx.clone();
    thread::spawn(move || {
        let path = Path::new(&rime_dir);
        if let Err(e) = watch_directory(path, watch_tx) {
            log::error!("file watcher error: {e:#}");
        }
    });

    loop {
        // Run initial cycle
        if state.should_sync() {
            log::info!("trigger fired, running sync cycle");
            if let Err(e) = TriggerState::run_cycle(cfg) {
                log::error!("sync cycle failed: {e:#}");
            } else {
                state.mark_synced();
                state.save(&state_path.to_string_lossy()).ok();
            }
        }

        // Wait for next file change event, then loop back
        match rx.recv() {
            Ok(_) => {
                // Brief debounce: coalesce rapid events
                while rx.try_recv().is_ok() {}
                thread::sleep(Duration::from_secs(5));
            }
            Err(mpsc::RecvError) => {
                log::error!("watcher channel closed, exiting");
                break;
            }
        }
    }

    Ok(())
}

fn watch_directory(path: &Path, tx: mpsc::Sender<()>) -> Result<()> {
    use inotify::{Inotify, WatchMask};

    if !path.exists() {
        anyhow::bail!("path does not exist: {}", path.display());
    }

    let mut inotify = Inotify::init().context("failed to init inotify")?;
    inotify
        .watches()
        .add(path, WatchMask::MODIFY | WatchMask::CREATE)
        .with_context(|| format!("failed to watch {}", path.display()))?;

    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .context("inotify read failed")?;
        if events.count() > 0 {
            if tx.send(()).is_err() {
                break;
            }
        }
    }
    Ok(())
}
