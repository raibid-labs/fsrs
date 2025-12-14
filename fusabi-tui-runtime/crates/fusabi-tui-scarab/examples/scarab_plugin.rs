//! Example TUI plugin for Scarab terminal.
//!
//! This example demonstrates how to create a simple TUI application that
//! runs inside Scarab terminal emulator using shared memory rendering.
//!
//! To run this example:
//! 1. Start Scarab daemon: `cargo run -p scarab-daemon`
//! 2. Run this plugin: `cargo run --example scarab_plugin --features plugin`

use fusabi_tui_scarab::prelude::*;
use std::time::{Duration, Instant};

/// A simple counter plugin that increments on each keypress.
struct CounterPlugin {
    /// Current counter value
    counter: u32,
    /// Plugin initialization time
    start_time: Option<Instant>,
    /// Last input event
    last_event: Option<String>,
}

impl CounterPlugin {
    fn new() -> Self {
        Self {
            counter: 0,
            start_time: None,
            last_event: None,
        }
    }
}

impl TuiPlugin for CounterPlugin {
    fn on_init(&mut self, ctx: &PluginContext) -> Result<()> {
        self.start_time = Some(Instant::now());
        eprintln!("CounterPlugin initialized!");
        eprintln!("Shared memory: {}", ctx.shm_path);
        eprintln!("Terminal size: {}x{}", ctx.terminal_size.width, ctx.terminal_size.height);
        Ok(())
    }

    fn on_render(&mut self, ctx: &RenderContext) -> Result<Buffer> {
        let mut buffer = Buffer::new(ctx.size);

        // Title
        let title_style = Style::new()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        buffer.set_string(0, 0, "=== Scarab TUI Plugin Demo ===", title_style);

        // Counter display
        let counter_style = Style::new()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let counter_text = format!("Counter: {}", self.counter);
        buffer.set_string(0, 2, &counter_text, counter_style);

        // Frame info
        let frame_style = Style::new().fg(Color::Yellow);
        let frame_text = format!("Frame: {} | Delta: {}ms", ctx.frame, ctx.delta_time_ms);
        buffer.set_string(0, 3, &frame_text, frame_style);

        // Uptime
        if let Some(start) = self.start_time {
            let uptime = start.elapsed();
            let uptime_text = format!("Uptime: {:.2}s", uptime.as_secs_f64());
            buffer.set_string(0, 4, &uptime_text, frame_style);
        }

        // Last event
        if let Some(ref event) = self.last_event {
            let event_style = Style::new().fg(Color::Magenta);
            let event_text = format!("Last event: {}", event);
            buffer.set_string(0, 6, &event_text, event_style);
        }

        // Instructions
        let help_style = Style::new().fg(Color::DarkGray);
        buffer.set_string(0, 8, "Press any key to increment counter", help_style);
        buffer.set_string(0, 9, "Press 'q' or ESC to exit", help_style);

        // Border at bottom
        let border_y = ctx.size.height.saturating_sub(1);
        let border_style = Style::new().fg(Color::Blue);
        let border = "=".repeat(ctx.size.width as usize);
        buffer.set_string(0, border_y, &border, border_style);

        Ok(buffer)
    }

    fn on_input(&mut self, event: InputEvent) -> Result<Action> {
        match event {
            InputEvent::Key(key_event) => {
                // Exit on 'q' or ESC
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(Action::Exit);
                    }
                    KeyCode::Char(c) => {
                        self.counter += 1;
                        self.last_event = Some(format!("Key: '{}'", c));
                    }
                    KeyCode::Enter => {
                        self.counter += 1;
                        self.last_event = Some("Key: Enter".to_string());
                    }
                    KeyCode::Up => {
                        self.last_event = Some("Key: Up Arrow".to_string());
                    }
                    KeyCode::Down => {
                        self.last_event = Some("Key: Down Arrow".to_string());
                    }
                    KeyCode::Left => {
                        self.last_event = Some("Key: Left Arrow".to_string());
                    }
                    KeyCode::Right => {
                        self.last_event = Some("Key: Right Arrow".to_string());
                    }
                    _ => {
                        self.counter += 1;
                        self.last_event = Some(format!("Key: {:?}", key_event.code));
                    }
                }

                // Show modifier state
                if key_event.modifiers.ctrl {
                    self.last_event = Some(format!("{} (Ctrl)", self.last_event.as_ref().unwrap()));
                }
                if key_event.modifiers.alt {
                    self.last_event = Some(format!("{} (Alt)", self.last_event.as_ref().unwrap()));
                }
                if key_event.modifiers.shift {
                    self.last_event = Some(format!("{} (Shift)", self.last_event.as_ref().unwrap()));
                }

                Ok(Action::Redraw)
            }
            InputEvent::Mouse(mouse_event) => {
                self.last_event = Some(format!(
                    "Mouse: {:?} at ({}, {})",
                    mouse_event.kind, mouse_event.column, mouse_event.row
                ));
                Ok(Action::Redraw)
            }
            InputEvent::Resize { width, height } => {
                self.last_event = Some(format!("Resized to {}x{}", width, height));
                Ok(Action::Redraw)
            }
            InputEvent::FocusGained => {
                self.last_event = Some("Focus gained".to_string());
                Ok(Action::Redraw)
            }
            InputEvent::FocusLost => {
                self.last_event = Some("Focus lost".to_string());
                Ok(Action::Redraw)
            }
        }
    }

    fn on_tick(&mut self) -> Result<()> {
        // Optional: Update animation state here
        Ok(())
    }

    fn on_shutdown(&mut self) {
        eprintln!("CounterPlugin shutting down...");
        eprintln!("Final counter value: {}", self.counter);
    }
}

fn main() -> Result<()> {
    eprintln!("Starting Scarab TUI Plugin Example...");

    // Connect to Scarab's shared memory
    let mut renderer = ScarabRenderer::connect(None)?;
    let size = renderer.size().map_err(|e| ScarabError::Render(e))?;

    // Create plugin
    let mut plugin = CounterPlugin::new();

    // Initialize plugin
    let ctx = PluginContext::new(SHMEM_PATH.to_string(), size);
    plugin.on_init(&ctx)?;

    // Main render loop
    let mut frame_count = 0u64;
    let mut last_render = Instant::now();

    // In a real plugin, this would be driven by Scarab's event loop
    // For this example, we'll just render a few frames
    for _ in 0..60 {
        let now = Instant::now();
        let delta = now.duration_since(last_render);
        last_render = now;

        // Create render context
        let render_ctx = RenderContext::new(size, delta.as_millis() as u64, frame_count);

        // Render
        let buffer = plugin.on_render(&render_ctx)?;
        renderer.draw(&buffer).map_err(|e| ScarabError::Render(e))?;
        renderer.flush().map_err(|e| ScarabError::Render(e))?;

        frame_count += 1;

        // Sleep to simulate frame rate (60 FPS)
        std::thread::sleep(Duration::from_millis(16));
    }

    plugin.on_shutdown();
    eprintln!("Plugin exited successfully!");

    Ok(())
}
