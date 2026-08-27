use eyre::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

/// Thread-safe live file watcher for real-time document reloading.
pub struct LiveFileWatcher {
    watcher: RecommendedWatcher,
    pub receiver: Receiver<PathBuf>,
}

impl LiveFileWatcher {
    /// Create a new live file watcher.
    pub fn new() -> Result<(Self, Sender<PathBuf>)> {
        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        let event_handler = move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths {
                        let _ = tx_clone.send(path);
                    }
                }
            }
        };

        let config = Config::default().with_poll_interval(Duration::from_millis(500));
        let watcher = RecommendedWatcher::new(event_handler, config)
            .with_context(|| "Failed to initialize native file watcher")?;

        Ok((Self {
            watcher,
            receiver: rx,
        }, tx))
    }

    /// Watch a file or directory for live changes.
    pub fn watch_path(&mut self, path: &Path) -> Result<()> {
        let recursive_mode = if path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        self.watcher
            .watch(path, recursive_mode)
            .with_context(|| format!("Failed to start watching path {}", path.display()))
    }

    /// Unwatch a previously watched path.
    #[allow(dead_code)]
    pub fn unwatch_path(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .unwatch(path)
            .with_context(|| format!("Failed to unwatch path {}", path.display()))
    }
}
