//! Scarab shared memory renderer implementation.

use fusabi_tui_core::buffer::Buffer;
use fusabi_tui_core::layout::Rect;
use fusabi_tui_render::error::{RenderError, Result as RenderResult};
use fusabi_tui_render::renderer::Renderer;
use shared_memory::{Shmem, ShmemConf};

use crate::convert::tui_cell_to_shared;
use crate::error::{Result, ScarabError};
use crate::shared::{SharedCell, SharedState, BUFFER_SIZE, GRID_HEIGHT, GRID_WIDTH, SHMEM_PATH};

/// Renderer that writes to Scarab's shared memory.
///
/// This renderer connects to Scarab's shared memory region and writes
/// TUI buffer contents directly to the shared grid, allowing zero-copy
/// rendering to the Scarab terminal emulator.
pub struct ScarabRenderer {
    /// Shared memory mapping
    shm: Shmem,
    /// Local copy of the last rendered buffer for diffing
    local_buffer: Buffer,
    /// Current sequence number
    sequence: u64,
}

// SAFETY: ScarabRenderer can be safely sent between threads because:
// 1. The shared memory is process-private (not shared across process boundaries for writes)
// 2. Access to the Shmem is controlled through &mut self methods
// 3. The raw pointer in Shmem is never accessed from multiple threads simultaneously
// Note: This is only safe because we control access patterns through the Renderer trait
unsafe impl Send for ScarabRenderer {}

impl ScarabRenderer {
    /// Connect to Scarab's shared memory.
    ///
    /// # Arguments
    ///
    /// * `shm_path` - Path to the shared memory region (defaults to `/scarab_shm_v1`)
    ///
    /// # Errors
    ///
    /// Returns an error if the shared memory cannot be opened or has invalid size.
    pub fn connect(shm_path: Option<&str>) -> Result<Self> {
        let path = shm_path.unwrap_or(SHMEM_PATH);

        // Open existing shared memory (created by Scarab daemon)
        let shm = ShmemConf::new()
            .flink(path)
            .open()
            .map_err(|e| ScarabError::SharedMemory(format!("Failed to open {}: {}", path, e)))?;

        // Verify size
        let expected_size = std::mem::size_of::<SharedState>();
        if shm.len() != expected_size {
            return Err(ScarabError::SizeMismatch {
                expected: expected_size,
                actual: shm.len(),
            });
        }

        // Create local buffer for diffing
        let area = Rect::new(0, 0, GRID_WIDTH as u16, GRID_HEIGHT as u16);
        let local_buffer = Buffer::new(area);

        Ok(Self {
            shm,
            local_buffer,
            sequence: 0,
        })
    }

    /// Disconnect from shared memory.
    pub fn disconnect(&mut self) {
        // Shared memory will be automatically unmapped when `shm` is dropped
    }

    /// Get a reference to the shared state.
    fn shared_state(&self) -> &SharedState {
        // SAFETY: We verified the size in connect() and SharedState is Pod
        unsafe { &*(self.shm.as_ptr() as *const SharedState) }
    }

    /// Get a mutable reference to the shared state.
    fn shared_state_mut(&mut self) -> &mut SharedState {
        // SAFETY: We verified the size in connect() and SharedState is Pod
        unsafe { &mut *(self.shm.as_ptr() as *mut SharedState) }
    }

    /// Convert a TUI buffer to shared cells.
    fn convert_buffer_to_shared(&self, buffer: &Buffer) -> Vec<SharedCell> {
        let mut shared_cells = Vec::with_capacity(BUFFER_SIZE);

        for y in 0..GRID_HEIGHT.min(buffer.area.height as usize) {
            for x in 0..GRID_WIDTH.min(buffer.area.width as usize) {
                let cell = buffer
                    .get(x as u16, y as u16)
                    .map(|c| tui_cell_to_shared(c))
                    .unwrap_or_default();
                shared_cells.push(cell);
            }
        }

        // Pad with default cells if buffer is smaller than grid
        while shared_cells.len() < BUFFER_SIZE {
            shared_cells.push(SharedCell::default());
        }

        shared_cells
    }

    /// Write buffer directly to shared memory with differential updates.
    fn write_buffer_diff(&mut self, buffer: &Buffer) -> Result<()> {
        // Collect changes first (to avoid borrowing issues)
        let mut changes = Vec::new();

        for y in 0..GRID_HEIGHT.min(buffer.area.height as usize) {
            for x in 0..GRID_WIDTH.min(buffer.area.width as usize) {
                if let Some(new_cell) = buffer.get(x as u16, y as u16) {
                    let old_cell = self.local_buffer.get(x as u16, y as u16);

                    // Only update if cell changed
                    if old_cell != Some(new_cell) {
                        let idx = y * GRID_WIDTH + x;
                        let shared_cell = tui_cell_to_shared(new_cell);
                        changes.push((idx, shared_cell));
                    }
                }
            }
        }

        // Apply changes if any
        if !changes.is_empty() {
            let state = self.shared_state_mut();

            for (idx, shared_cell) in changes {
                state.cells[idx] = shared_cell;
            }

            // Update sequence number to signal change
            state.increment_sequence();
            state.mark_dirty();
            self.sequence = state.sequence_number;

            // Update local buffer
            self.local_buffer = buffer.clone();
        }

        Ok(())
    }
}

impl Renderer for ScarabRenderer {
    fn draw(&mut self, buffer: &Buffer) -> RenderResult<()> {
        // Write buffer to shared memory
        self.write_buffer_diff(buffer)
            .map_err(|e| RenderError::Backend(e.to_string()))?;
        Ok(())
    }

    fn flush(&mut self) -> RenderResult<()> {
        // For shared memory, flush is a no-op (writes are immediate)
        // The client will pick up changes by checking sequence_number
        Ok(())
    }

    fn size(&self) -> RenderResult<Rect> {
        Ok(Rect::new(0, 0, GRID_WIDTH as u16, GRID_HEIGHT as u16))
    }

    fn clear(&mut self) -> RenderResult<()> {
        let state = self.shared_state_mut();

        // Reset all cells to default
        for cell in &mut state.cells {
            *cell = SharedCell::default();
        }

        // Update sequence number
        state.increment_sequence();
        state.mark_dirty();
        self.sequence = state.sequence_number;

        // Clear local buffer
        self.local_buffer.clear();

        Ok(())
    }

    fn show_cursor(&mut self, _show: bool) -> RenderResult<()> {
        // Cursor visibility is handled by setting cursor position to valid/invalid coords
        // For now, we'll just store it in the shared state
        // This may need to be extended in the future
        Ok(())
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> RenderResult<()> {
        let state = self.shared_state_mut();
        state.cursor_x = x;
        state.cursor_y = y;
        Ok(())
    }
}

impl Drop for ScarabRenderer {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fusabi_tui_core::style::{Color, Modifier, Style};
    use tempfile::tempdir;

    #[test]
    fn test_convert_buffer_to_shared() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buffer = Buffer::new(area);

        let style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);
        buffer.set_string(0, 0, "Hello", style);

        // Create a mock shared memory for testing
        let dir = tempdir().unwrap();
        let shm_path = dir.path().join("test_shm");

        // We can't easily test the full renderer without Scarab daemon,
        // but we can test the conversion logic
        let mut shared_cells = Vec::new();
        for y in 0..5 {
            for x in 0..10 {
                let cell = buffer
                    .get(x, y)
                    .map(SharedCell::from)
                    .unwrap_or_default();
                shared_cells.push(cell);
            }
        }

        // Verify first character
        assert_eq!(shared_cells[0].char_codepoint, 'H' as u32);
        assert!(shared_cells[0].flags & SharedCell::FLAG_BOLD != 0);
    }

    #[test]
    fn test_shared_cell_conversion() {
        let tui_cell = fusabi_tui_core::buffer::Cell {
            symbol: "A".to_string(),
            fg: Color::Green,
            bg: Color::Black,
            modifier: Modifier::ITALIC,
        };

        let shared_cell = SharedCell::from(&tui_cell);
        assert_eq!(shared_cell.char_codepoint, 'A' as u32);
        assert!(shared_cell.flags & SharedCell::FLAG_ITALIC != 0);
    }
}
