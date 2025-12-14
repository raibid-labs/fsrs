# fusabi-tui-scarab

Scarab shared memory backend for Fusabi TUI framework.

This crate provides zero-copy integration between the Fusabi TUI framework and the Scarab terminal emulator's shared memory protocol.

## Features

- **Zero-copy rendering** via shared memory
- **Lock-free synchronization** using sequence numbers
- **Differential updates** to minimize memory writes
- **Type-safe conversions** between Fusabi and Scarab types
- **Plugin API** for building interactive TUI applications
- **Full Renderer trait implementation**

## Architecture

Scarab uses a split-process architecture:

- **Daemon**: Headless server that owns PTY processes
- **Client**: Bevy-based GUI that renders via shared memory
- **Plugins**: Can run in either process with Fusabi scripting

This crate enables TUI applications to render directly to Scarab's shared memory buffer.

## Quick Start

### Basic Rendering

```rust
use fusabi_tui_scarab::prelude::*;

// Connect to Scarab's shared memory
let mut renderer = ScarabRenderer::connect(None)?;

// Create a buffer and draw to it
let size = renderer.size()?;
let mut buffer = Buffer::new(size);

let style = Style::new().fg(Color::Green);
buffer.set_string(0, 0, "Hello, Scarab!", style);

// Render to shared memory
renderer.draw(&buffer)?;
renderer.flush()?;
```

### Plugin Development

```rust
use fusabi_tui_scarab::prelude::*;

struct MyPlugin {
    counter: u32,
}

impl TuiPlugin for MyPlugin {
    fn on_init(&mut self, ctx: &PluginContext) -> Result<()> {
        println!("Plugin initialized!");
        Ok(())
    }

    fn on_render(&mut self, ctx: &RenderContext) -> Result<Buffer> {
        let mut buffer = Buffer::new(ctx.size);
        let style = Style::new().fg(Color::Cyan);

        let text = format!("Frame: {}", ctx.frame);
        buffer.set_string(0, 0, &text, style);

        Ok(buffer)
    }

    fn on_input(&mut self, event: InputEvent) -> Result<Action> {
        self.counter += 1;
        Ok(Action::Redraw)
    }
}
```

## Shared Memory Layout

The shared memory region contains:

- **Sequence number**: 64-bit atomic counter for synchronization
- **Cursor position**: X/Y coordinates
- **Grid cells**: 200x100 array of 16-byte cells

All structures use `#[repr(C)]` and `bytemuck::Pod` for zero-copy safety.

## Cell Format

Each cell is exactly 16 bytes:

```rust
#[repr(C)]
pub struct SharedCell {
    pub char_codepoint: u32,  // UTF-32 character
    pub fg: u32,              // ARGB foreground color
    pub bg: u32,              // ARGB background color
    pub flags: u8,            // Text attributes (bold, italic, etc.)
    pub _padding: [u8; 3],    // Alignment padding
}
```

## Synchronization

The renderer uses lock-free synchronization:

1. Plugin renders to local buffer
2. Renderer computes diff with previous frame
3. Changed cells are written to shared memory
4. Sequence number is atomically incremented
5. Scarab client detects change and re-renders

## Features

- `plugin`: Enables plugin system with JSON serialization (optional)

## Examples

See `examples/scarab_plugin.rs` for a complete interactive plugin example.

Run with:
```bash
cargo run --example scarab_plugin --features plugin
```

## Testing

Run the test suite:
```bash
cargo test -p fusabi-tui-scarab
```

## Integration with Scarab

This crate matches the exact memory layout of `scarab-protocol`:

- Same cell structure (16 bytes)
- Same grid dimensions (200x100)
- Same ARGB color format
- Same attribute flags

## Thread Safety

`ScarabRenderer` is `Send` but not `Sync`. Each renderer instance should be owned by a single thread, though multiple instances can connect to different shared memory regions.

## License

MIT OR Apache-2.0
