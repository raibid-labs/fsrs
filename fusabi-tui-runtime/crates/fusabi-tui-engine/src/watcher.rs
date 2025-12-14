//! File watching for hot reload functionality.

use crate::error::{WatchError, WatchResult};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

/// Watches files for changes and provides debounced change notifications.
pub struct FileWatcher {
    /// Receiver for file system events.
    rx: Receiver<notify::Result<Event>>,

    /// The underlying file system watcher.
    _watcher: RecommendedWatcher,

    /// Set of paths currently being watched.
    watched_paths: HashSet<PathBuf>,

    /// Debounce duration in milliseconds.
    debounce_ms: u64,

    /// Accumulator for recent changes (for debouncing).
    pending_changes: HashSet<PathBuf>,

    /// Last time we processed changes.
    last_process_time: std::time::Instant,
}

impl FileWatcher {
    /// Create a new file watcher with the specified debounce time.
    ///
    /// The debounce time determines how long to wait after the last change
    /// before reporting changes. This prevents rapid successive notifications
    /// when files are being saved.
    pub fn new(debounce_ms: u64) -> WatchResult<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(_) = tx.send(res) {
                    // Channel closed, watcher is shutting down
                }
            },
            Config::default(),
        )
        .map_err(|e| WatchError::InitFailed(e.to_string()))?;

        Ok(Self {
            rx,
            _watcher: watcher,
            watched_paths: HashSet::new(),
            debounce_ms,
            pending_changes: HashSet::new(),
            last_process_time: std::time::Instant::now(),
        })
    }

    /// Start watching a file or directory.
    ///
    /// For directories, this will watch all files in the directory recursively.
    pub fn watch(&mut self, path: &Path) -> WatchResult<()> {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| WatchError::WatchFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        // Don't watch if already watching
        if self.watched_paths.contains(&canonical_path) {
            return Ok(());
        }

        self._watcher
            .watch(&canonical_path, RecursiveMode::Recursive)
            .map_err(|e| WatchError::WatchFailed {
                path: canonical_path.clone(),
                reason: e.to_string(),
            })?;

        self.watched_paths.insert(canonical_path);

        Ok(())
    }

    /// Stop watching a file or directory.
    pub fn unwatch(&mut self, path: &Path) -> WatchResult<()> {
        let canonical_path = path
            .canonicalize()
            .map_err(|e| WatchError::UnwatchFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        if !self.watched_paths.contains(&canonical_path) {
            return Ok(());
        }

        self._watcher
            .unwatch(&canonical_path)
            .map_err(|e| WatchError::UnwatchFailed {
                path: canonical_path.clone(),
                reason: e.to_string(),
            })?;

        self.watched_paths.remove(&canonical_path);

        Ok(())
    }

    /// Poll for file changes and return paths that have changed.
    ///
    /// This method implements debouncing - it will only return changes after
    /// the debounce period has elapsed since the last change.
    pub fn poll(&mut self) -> Vec<PathBuf> {
        // Process all pending events
        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => {
                    self.process_event(event);
                }
                Ok(Err(_)) => {
                    // Error from notify, ignore
                }
                Err(TryRecvError::Empty) => {
                    // No more events
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // Channel closed
                    break;
                }
            }
        }

        // Check if debounce period has elapsed
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_process_time);

        if elapsed >= Duration::from_millis(self.debounce_ms) && !self.pending_changes.is_empty()
        {
            let changes: Vec<PathBuf> = self.pending_changes.drain().collect();
            self.last_process_time = now;
            changes
        } else {
            Vec::new()
        }
    }

    /// Process a file system event.
    fn process_event(&mut self, event: Event) {
        // Filter out events we don't care about
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in event.paths {
                    // Only track files, not directories
                    if path.is_file() {
                        self.pending_changes.insert(path);
                    }
                }
            }
            _ => {
                // Ignore other event types (access, metadata, etc.)
            }
        }
    }

    /// Get the list of watched paths.
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.watched_paths.iter().cloned().collect()
    }

    /// Get the number of watched paths.
    pub fn watch_count(&self) -> usize {
        self.watched_paths.len()
    }

    /// Clear all watches.
    pub fn clear(&mut self) -> WatchResult<()> {
        let paths: Vec<PathBuf> = self.watched_paths.iter().cloned().collect();
        for path in paths {
            self.unwatch(&path)?;
        }
        Ok(())
    }

    /// Get the debounce duration in milliseconds.
    pub fn debounce_ms(&self) -> u64 {
        self.debounce_ms
    }

    /// Set a new debounce duration in milliseconds.
    pub fn set_debounce_ms(&mut self, debounce_ms: u64) {
        self.debounce_ms = debounce_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_file_watcher_new() {
        let watcher = FileWatcher::new(100);
        assert!(watcher.is_ok());

        let watcher = watcher.unwrap();
        assert_eq!(watcher.watch_count(), 0);
        assert_eq!(watcher.debounce_ms(), 100);
    }

    #[test]
    fn test_watch_file() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        let result = watcher.watch(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(watcher.watch_count(), 1);
    }

    #[test]
    fn test_watch_directory() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let temp_dir = tempdir().unwrap();
        let result = watcher.watch(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(watcher.watch_count(), 1);
    }

    #[test]
    fn test_watch_duplicate() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        watcher.watch(temp_file.path()).unwrap();
        assert_eq!(watcher.watch_count(), 1);

        // Watch again - should not increase count
        watcher.watch(temp_file.path()).unwrap();
        assert_eq!(watcher.watch_count(), 1);
    }

    #[test]
    fn test_unwatch() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        watcher.watch(temp_file.path()).unwrap();
        assert_eq!(watcher.watch_count(), 1);

        watcher.unwatch(temp_file.path()).unwrap();
        assert_eq!(watcher.watch_count(), 0);
    }

    #[test]
    fn test_clear_watches() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let temp_dir = tempdir().unwrap();
        let mut temp_file1 = NamedTempFile::new_in(temp_dir.path()).unwrap();
        let mut temp_file2 = NamedTempFile::new_in(temp_dir.path()).unwrap();

        writeln!(temp_file1, "test 1").unwrap();
        writeln!(temp_file2, "test 2").unwrap();

        watcher.watch(temp_file1.path()).unwrap();
        watcher.watch(temp_file2.path()).unwrap();
        assert_eq!(watcher.watch_count(), 2);

        watcher.clear().unwrap();
        assert_eq!(watcher.watch_count(), 0);
    }

    #[test]
    fn test_debounce_setting() {
        let mut watcher = FileWatcher::new(100).unwrap();
        assert_eq!(watcher.debounce_ms(), 100);

        watcher.set_debounce_ms(500);
        assert_eq!(watcher.debounce_ms(), 500);
    }

    #[test]
    fn test_poll_no_changes() {
        let mut watcher = FileWatcher::new(100).unwrap();
        let changes = watcher.poll();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_watched_paths() {
        let mut watcher = FileWatcher::new(100).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        watcher.watch(temp_file.path()).unwrap();

        let watched = watcher.watched_paths();
        assert_eq!(watched.len(), 1);
    }
}
