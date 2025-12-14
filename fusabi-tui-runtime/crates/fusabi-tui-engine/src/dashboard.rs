//! Dashboard engine for managing hot-reloadable TUI applications.

use crate::error::{EngineError, EngineResult};
use crate::event::{Action, Event};
use crate::loader::FileLoader;
use crate::state::DashboardState;
use crate::watcher::FileWatcher;
use fusabi_tui_core::buffer::Buffer;
use fusabi_tui_render::renderer::Renderer;
use std::path::{Path, PathBuf};

/// The main dashboard engine that orchestrates hot reloading and rendering.
///
/// The DashboardEngine manages:
/// - File loading and caching
/// - Hot reload watching
/// - Rendering to a backend
/// - Event handling
/// - State management
pub struct DashboardEngine<R: Renderer> {
    /// The renderer backend.
    renderer: R,

    /// File loader for loading and caching files.
    loader: FileLoader,

    /// Optional file watcher for hot reloading.
    watcher: Option<FileWatcher>,

    /// Current dashboard state.
    state: DashboardState,

    /// Root path for resolving relative file paths.
    root_path: PathBuf,

    /// The entry file path (main dashboard file).
    entry_file: Option<PathBuf>,
}

impl<R: Renderer> DashboardEngine<R> {
    /// Create a new dashboard engine with the given renderer and root path.
    ///
    /// # Arguments
    ///
    /// * `renderer` - The rendering backend to use
    /// * `root_path` - The root directory for resolving relative file paths
    ///
    /// # Example
    ///
    /// ```no_run
    /// use fusabi_tui_engine::dashboard::DashboardEngine;
    /// use fusabi_tui_render::test::TestRenderer;
    /// use std::path::PathBuf;
    ///
    /// let renderer = TestRenderer::new(80, 24);
    /// let engine = DashboardEngine::new(renderer, PathBuf::from("."));
    /// ```
    pub fn new(renderer: R, root_path: PathBuf) -> Self {
        Self {
            renderer,
            loader: FileLoader::new(),
            watcher: None,
            state: DashboardState::new(),
            root_path,
            entry_file: None,
        }
    }

    /// Load a dashboard file.
    ///
    /// This loads the specified file and all its dependencies. If hot reload
    /// is enabled, it will also start watching the file for changes.
    ///
    /// # Arguments
    ///
    /// * `entry` - The path to the main dashboard file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be loaded or parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fusabi_tui_engine::dashboard::DashboardEngine;
    /// # use fusabi_tui_render::test::TestRenderer;
    /// # use std::path::{Path, PathBuf};
    /// # let renderer = TestRenderer::new(80, 24);
    /// # let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));
    /// engine.load(Path::new("dashboard.fsx")).unwrap();
    /// ```
    pub fn load(&mut self, entry: &Path) -> EngineResult<()> {
        let path = if entry.is_absolute() {
            entry.to_path_buf()
        } else {
            self.root_path.join(entry)
        };

        // Load the file
        let loaded_file = self.loader.load(&path)?;

        // Store the entry file path
        self.entry_file = Some(loaded_file.path.clone());

        // If watcher is enabled, watch this file
        if let Some(watcher) = &mut self.watcher {
            watcher.watch(&loaded_file.path)?;

            // Also watch dependencies
            for dep in &loaded_file.dependencies {
                watcher.watch(dep)?;
            }
        }

        // Mark state as dirty to trigger a render
        self.state.mark_dirty();

        Ok(())
    }

    /// Reload the current dashboard.
    ///
    /// This invalidates the cache for the entry file and all its dependents,
    /// then reloads everything.
    ///
    /// # Errors
    ///
    /// Returns an error if the reload fails.
    pub fn reload(&mut self) -> EngineResult<()> {
        let entry_path = self
            .entry_file
            .clone()
            .ok_or_else(|| EngineError::InvalidState("No entry file loaded".to_string()))?;

        // Invalidate the entry file and all dependents
        let _invalidated = self.loader.invalidate(&entry_path);

        // Reload the entry file
        let loaded_file = self.loader.load(&entry_path)?;

        // Update watches for new dependencies
        if let Some(watcher) = &mut self.watcher {
            for dep in &loaded_file.dependencies {
                watcher.watch(dep)?;
            }
        }

        // Mark state as dirty
        self.state.mark_dirty();

        Ok(())
    }

    /// Render the current dashboard state.
    ///
    /// This creates a buffer, renders the current state to it, and flushes
    /// to the renderer backend.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render(&mut self) -> EngineResult<()> {
        // Get the terminal size
        let size = self.renderer.size()?;

        // Create a buffer for the current frame
        let buffer = Buffer::new(size);

        // TODO: Render widgets from state to buffer
        // This would involve:
        // 1. Iterating over widgets in state
        // 2. Calling widget.render() on each
        // 3. Drawing to the buffer
        //
        // For now, we just have an empty buffer

        // Draw the buffer to the renderer
        self.renderer.draw(&buffer)?;
        self.renderer.flush()?;

        // Clear dirty flag
        self.state.clear_dirty();

        Ok(())
    }

    /// Enable hot reload functionality.
    ///
    /// This creates a file watcher that will monitor files for changes.
    /// Call `poll_changes()` regularly to check for file modifications.
    ///
    /// # Arguments
    ///
    /// * `debounce_ms` - Optional debounce time in milliseconds (default: 100ms)
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    pub fn enable_hot_reload(&mut self) -> EngineResult<()> {
        self.enable_hot_reload_with_debounce(100)
    }

    /// Enable hot reload with a custom debounce time.
    ///
    /// # Arguments
    ///
    /// * `debounce_ms` - Debounce time in milliseconds
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    pub fn enable_hot_reload_with_debounce(&mut self, debounce_ms: u64) -> EngineResult<()> {
        let watcher = FileWatcher::new(debounce_ms)?;
        self.watcher = Some(watcher);

        // If we have an entry file, start watching it
        if let Some(entry_path) = &self.entry_file {
            if let Some(watcher) = &mut self.watcher {
                watcher.watch(entry_path)?;

                // Watch dependencies
                if let Some(loaded) = self.loader.get(entry_path) {
                    for dep in &loaded.dependencies {
                        watcher.watch(dep)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Disable hot reload functionality.
    pub fn disable_hot_reload(&mut self) {
        self.watcher = None;
    }

    /// Poll for file changes and return the list of changed files.
    ///
    /// If hot reload is not enabled, this returns `None`.
    pub fn poll_changes(&mut self) -> Option<Vec<PathBuf>> {
        self.watcher.as_mut().map(|w| w.poll())
    }

    /// Handle an input event and return the resulting action.
    ///
    /// This is where application-specific event handling logic would go.
    /// The default implementation handles basic events like Ctrl+C to quit.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Returns
    ///
    /// Returns an action indicating what should be done in response to the event.
    pub fn handle_event(&mut self, event: Event) -> EngineResult<Action> {
        // Handle file change events
        if let Event::FileChange(path) = &event {
            // Invalidate changed files
            let _invalidated = self.loader.invalidate(path);

            // Reload the dashboard
            self.reload()?;

            return Ok(Action::Render);
        }

        // Handle resize events
        if let Event::Resize(_, _) = event {
            self.state.mark_dirty();
            return Ok(Action::Render);
        }

        // Default event handling
        use crate::event::KeyCode;

        if let Event::Key(key_event) = event {
            // Ctrl+C to quit
            if key_event.code == KeyCode::Char('c') && key_event.modifiers.ctrl {
                return Ok(Action::Quit);
            }

            // Ctrl+R to force reload
            if key_event.code == KeyCode::Char('r') && key_event.modifiers.ctrl {
                self.reload()?;
                return Ok(Action::Render);
            }
        }

        Ok(Action::None)
    }

    /// Get a reference to the dashboard state.
    pub fn state(&self) -> &DashboardState {
        &self.state
    }

    /// Get a mutable reference to the dashboard state.
    pub fn state_mut(&mut self) -> &mut DashboardState {
        &mut self.state
    }

    /// Get a reference to the renderer.
    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    /// Get a mutable reference to the renderer.
    pub fn renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    /// Get the root path.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get the entry file path, if loaded.
    pub fn entry_file(&self) -> Option<&Path> {
        self.entry_file.as_deref()
    }

    /// Check if hot reload is enabled.
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.watcher.is_some()
    }

    /// Clear the renderer and state.
    pub fn clear(&mut self) -> EngineResult<()> {
        self.renderer.clear()?;
        self.state.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{KeyCode, KeyEvent, KeyModifiers};
    use fusabi_tui_render::test::TestRenderer;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_dashboard_engine_new() {
        let renderer = TestRenderer::new(80, 24);
        let engine = DashboardEngine::new(renderer, PathBuf::from("."));

        assert_eq!(engine.root_path(), Path::new("."));
        assert!(!engine.is_hot_reload_enabled());
        assert!(engine.entry_file().is_none());
    }

    #[test]
    fn test_load_file() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();

        let result = engine.load(temp_file.path());
        assert!(result.is_ok());
        assert!(engine.entry_file().is_some());
    }

    #[test]
    fn test_enable_hot_reload() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        assert!(!engine.is_hot_reload_enabled());

        let result = engine.enable_hot_reload();
        assert!(result.is_ok());
        assert!(engine.is_hot_reload_enabled());

        engine.disable_hot_reload();
        assert!(!engine.is_hot_reload_enabled());
    }

    #[test]
    fn test_poll_changes_without_watcher() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let changes = engine.poll_changes();
        assert!(changes.is_none());
    }

    #[test]
    fn test_poll_changes_with_watcher() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        engine.enable_hot_reload().unwrap();

        let changes = engine.poll_changes();
        assert!(changes.is_some());
        assert!(changes.unwrap().is_empty());
    }

    #[test]
    fn test_handle_quit_event() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::ctrl(),
        });

        let action = engine.handle_event(event).unwrap();
        assert_eq!(action, Action::Quit);
    }

    #[test]
    fn test_handle_reload_event() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x = 42").unwrap();

        engine.load(temp_file.path()).unwrap();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::ctrl(),
        });

        let action = engine.handle_event(event).unwrap();
        assert_eq!(action, Action::Render);
    }

    #[test]
    fn test_handle_resize_event() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let event = Event::Resize(100, 30);
        let action = engine.handle_event(event).unwrap();
        assert_eq!(action, Action::Render);
        assert!(engine.state().dirty);
    }

    #[test]
    fn test_render() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let result = engine.render();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear() {
        let renderer = TestRenderer::new(80, 24);
        let mut engine = DashboardEngine::new(renderer, PathBuf::from("."));

        let result = engine.clear();
        assert!(result.is_ok());
    }
}
