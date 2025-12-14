//! Demonstration of the Block widget with various configurations.

use fusabi_tui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use fusabi_tui_widgets::{
    block::{Block, Padding, Title, TitleAlignment, TitlePosition},
    borders::{BorderType, Borders},
    widget::Widget,
};

fn main() {
    // Create a simple block with all borders
    let block1 = Block::default()
        .title("Simple Block")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    // Create a block with rounded borders and centered title
    let block2 = Block::default()
        .title("Rounded Block")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_alignment(TitleAlignment::Center)
        .border_style(Style::default().fg(Color::Cyan));

    // Create a block with double borders, bottom title, and padding
    let block3 = Block::default()
        .title(
            Title::new("Bottom Title")
                .position(TitlePosition::Bottom)
                .alignment(TitleAlignment::Right)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green))
        .padding(Padding::uniform(1));

    // Render blocks to buffers
    let area = Rect::new(0, 0, 30, 5);
    let mut buffer1 = Buffer::new(area);
    let mut buffer2 = Buffer::new(area);
    let mut buffer3 = Buffer::new(area);

    block1.render(area, &mut buffer1);
    block2.render(area, &mut buffer2);
    block3.render(area, &mut buffer3);

    println!("Block 1 - Simple:");
    print_buffer(&buffer1);

    println!("\nBlock 2 - Rounded with centered title:");
    print_buffer(&buffer2);

    println!("\nBlock 3 - Double borders with bottom right title and padding:");
    print_buffer(&buffer3);

    // Demonstrate inner area calculation
    let inner = block3.inner(area);
    println!("\nInner area for block3: x={}, y={}, w={}, h={}",
             inner.x, inner.y, inner.width, inner.height);
}

fn print_buffer(buffer: &Buffer) {
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(cell) = buffer.get(x, y) {
                print!("{}", cell.symbol);
            }
        }
        println!();
    }
}
