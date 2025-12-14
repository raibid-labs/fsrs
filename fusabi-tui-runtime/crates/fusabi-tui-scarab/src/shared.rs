//! Shared memory types matching Scarab's protocol.
//!
//! These types must match the layout in scarab-protocol exactly for zero-copy access.

use bytemuck::{Pod, Zeroable};
use std::sync::atomic::{AtomicU64, Ordering};

/// Default shared memory path (matches Scarab protocol)
pub const SHMEM_PATH: &str = "/scarab_shm_v1";

/// Grid dimensions (matches Scarab protocol)
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 100;
pub const BUFFER_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

/// A single cell in the shared memory grid.
///
/// This must exactly match the layout in `scarab-protocol::Cell`.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct SharedCell {
    /// Unicode codepoint (UTF-32)
    pub char_codepoint: u32,
    /// Foreground color in RGBA format (0xRRGGBBAA)
    pub fg: u32,
    /// Background color in RGBA format (0xRRGGBBAA)
    pub bg: u32,
    /// Text attribute flags (bold, italic, etc.)
    pub flags: u8,
    /// Padding for alignment to 16 bytes
    pub _padding: [u8; 3],
}

impl Default for SharedCell {
    fn default() -> Self {
        Self {
            char_codepoint: b' ' as u32,
            fg: 0xFFA8DF5A, // Slime green foreground (ARGB: #a8df5a)
            bg: 0xFF0D1208, // Slime dark background (ARGB: #0d1208)
            flags: 0,
            _padding: [0; 3],
        }
    }
}

/// Cell attribute flags
impl SharedCell {
    pub const FLAG_BOLD: u8 = 0b0000_0001;
    pub const FLAG_DIM: u8 = 0b0000_0010;
    pub const FLAG_ITALIC: u8 = 0b0000_0100;
    pub const FLAG_UNDERLINE: u8 = 0b0000_1000;
    pub const FLAG_BLINK: u8 = 0b0001_0000;
    pub const FLAG_REVERSE: u8 = 0b0010_0000;
    pub const FLAG_HIDDEN: u8 = 0b0100_0000;
    pub const FLAG_STRIKETHROUGH: u8 = 0b1000_0000;
}

/// Shared state in shared memory.
///
/// This must exactly match the layout in `scarab-protocol::SharedState`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SharedState {
    /// Atomic sequence number for synchronization
    pub sequence_number: u64,
    /// Dirty flag (1 if buffer needs redraw)
    pub dirty_flag: u8,
    /// Error mode (0 = normal, 1 = error/unavailable)
    pub error_mode: u8,
    /// Cursor X position
    pub cursor_x: u16,
    /// Cursor Y position
    pub cursor_y: u16,
    /// Padding for alignment
    pub _padding2: [u8; 2],
    /// Grid cells (200x100)
    pub cells: [SharedCell; BUFFER_SIZE],
}

// Manual Pod/Zeroable implementation for large array
unsafe impl Pod for SharedState {}
unsafe impl Zeroable for SharedState {}

impl SharedState {
    /// Create a new empty shared state
    pub fn new() -> Self {
        Self {
            sequence_number: 0,
            dirty_flag: 0,
            error_mode: 0,
            cursor_x: 0,
            cursor_y: 0,
            _padding2: [0; 2],
            cells: [SharedCell::default(); BUFFER_SIZE],
        }
    }

    /// Get a reference to the cell at the given coordinates
    pub fn get_cell(&self, x: u16, y: u16) -> Option<&SharedCell> {
        if x >= GRID_WIDTH as u16 || y >= GRID_HEIGHT as u16 {
            return None;
        }
        let idx = (y as usize) * GRID_WIDTH + (x as usize);
        self.cells.get(idx)
    }

    /// Get a mutable reference to the cell at the given coordinates
    pub fn get_cell_mut(&mut self, x: u16, y: u16) -> Option<&mut SharedCell> {
        if x >= GRID_WIDTH as u16 || y >= GRID_HEIGHT as u16 {
            return None;
        }
        let idx = (y as usize) * GRID_WIDTH + (x as usize);
        self.cells.get_mut(idx)
    }

    /// Increment the sequence number atomically
    pub fn increment_sequence(&mut self) {
        self.sequence_number = self.sequence_number.wrapping_add(1);
    }

    /// Mark the buffer as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty_flag = 1;
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty_flag = 0;
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for accessing SharedState through an atomic sequence number
pub struct SharedStateReader<'a> {
    state: &'a SharedState,
    last_sequence: AtomicU64,
}

impl<'a> SharedStateReader<'a> {
    /// Create a new reader for the shared state
    pub fn new(state: &'a SharedState) -> Self {
        Self {
            state,
            last_sequence: AtomicU64::new(0),
        }
    }

    /// Check if the state has been updated since the last read
    pub fn has_update(&self) -> bool {
        let current = self.state.sequence_number;
        let last = self.last_sequence.load(Ordering::Acquire);
        current != last
    }

    /// Update the last sequence number
    pub fn update_sequence(&self) {
        let current = self.state.sequence_number;
        self.last_sequence.store(current, Ordering::Release);
    }

    /// Get the current sequence number
    pub fn sequence(&self) -> u64 {
        self.state.sequence_number
    }

    /// Get a reference to the underlying state
    pub fn state(&self) -> &SharedState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_cell_size() {
        // Verify the size matches Scarab's expectation (16 bytes)
        assert_eq!(std::mem::size_of::<SharedCell>(), 16);
    }

    #[test]
    fn test_shared_cell_default() {
        let cell = SharedCell::default();
        assert_eq!(cell.char_codepoint, b' ' as u32);
        assert_eq!(cell.flags, 0);
    }

    #[test]
    fn test_shared_state_new() {
        let state = SharedState::new();
        assert_eq!(state.sequence_number, 0);
        assert_eq!(state.cursor_x, 0);
        assert_eq!(state.cursor_y, 0);
        assert_eq!(state.cells.len(), BUFFER_SIZE);
    }

    #[test]
    fn test_shared_state_get_cell() {
        let state = SharedState::new();

        // Valid coordinates
        assert!(state.get_cell(0, 0).is_some());
        assert!(state.get_cell(GRID_WIDTH as u16 - 1, GRID_HEIGHT as u16 - 1).is_some());

        // Invalid coordinates
        assert!(state.get_cell(GRID_WIDTH as u16, 0).is_none());
        assert!(state.get_cell(0, GRID_HEIGHT as u16).is_none());
    }

    #[test]
    fn test_shared_state_sequence() {
        let mut state = SharedState::new();
        assert_eq!(state.sequence_number, 0);

        state.increment_sequence();
        assert_eq!(state.sequence_number, 1);

        state.increment_sequence();
        assert_eq!(state.sequence_number, 2);
    }

    #[test]
    fn test_shared_state_dirty() {
        let mut state = SharedState::new();
        assert_eq!(state.dirty_flag, 0);

        state.mark_dirty();
        assert_eq!(state.dirty_flag, 1);

        state.clear_dirty();
        assert_eq!(state.dirty_flag, 0);
    }
}
