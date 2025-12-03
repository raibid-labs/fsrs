# UIFormatting Module Usage Example

This example demonstrates how to use the UIFormatting stdlib module for status bar and UI element formatting.

## Fusabi Code Example

```fsharp
module MyPlugin =
    // Define a tab formatter
    let formatMyTab (tabInfo: TabInfo) : StatusSegment list =
        let textColor = if tabInfo.active then Some "blue" else Some "gray"
        let boldness = tabInfo.active

        [
            { text = sprintf "[%d]" tabInfo.index
              fgColor = textColor
              bgColor = None
              bold = boldness }
            { text = tabInfo.title
              fgColor = textColor
              bgColor = None
              bold = boldness }
        ]

    // Define a status left formatter
    let formatStatusLeft (statusInfo: StatusInfo) : StatusSegment list =
        [
            { text = sprintf "Tab %d/%d" statusInfo.currentTab statusInfo.totalTabs
              fgColor = Some "green"
              bgColor = None
              bold = false }
        ]

    // Define a status right formatter
    let formatStatusRight (statusInfo: StatusInfo) : StatusSegment list =
        [
            { text = statusInfo.time
              fgColor = Some "yellow"
              bgColor = None
              bold = true }
        ]

    // Register the formatters
    let tabHandlerId = UIFormatting.onFormatTab formatMyTab
    let leftHandlerId = UIFormatting.onFormatStatusLeft formatStatusLeft
    let rightHandlerId = UIFormatting.onFormatStatusRight formatStatusRight

    // Later, if you need to remove a formatter:
    // let removed = UIFormatting.removeFormatter tabHandlerId

    // Or clear all formatters:
    // UIFormatting.clearFormatters ()
```

## Rust Host Code Example

```rust
use fusabi_vm::stdlib::ui_formatting;
use fusabi_vm::vm::Vm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = Vm::new();

    // ... load and run Fusabi script that registers formatters ...

    // When you need to format a tab:
    let tab_info = ui_formatting::create_tab_info(
        0,                      // index
        "Main Tab".to_string(), // title
        true,                   // active
        false,                  // hasActivity
    );

    let tab_results = ui_formatting::invoke_tab_formatters(&mut vm, tab_info)?;

    // Process the results
    for formatter_result in tab_results {
        for segment in formatter_result {
            let (text, fg_color, bg_color, bold) =
                ui_formatting::extract_status_segment(&segment)?;

            println!("Segment: text='{}', fg={:?}, bg={:?}, bold={}",
                text, fg_color, bg_color, bold);
        }
    }

    // When you need to format the status bar:
    let status_info = ui_formatting::create_status_info(
        1,                      // currentTab
        5,                      // totalTabs
        "12:34:56".to_string(), // time
    );

    let left_results = ui_formatting::invoke_status_left_formatters(&mut vm, status_info.clone())?;
    let right_results = ui_formatting::invoke_status_right_formatters(&mut vm, status_info)?;

    // Process left and right status segments...

    Ok(())
}
```

## Data Types

### StatusSegment
```fsharp
type StatusSegment = {
    text: string
    fgColor: string option
    bgColor: string option
    bold: bool
}
```

### TabInfo
```fsharp
type TabInfo = {
    index: int
    title: string
    active: bool
    hasActivity: bool
}
```

### StatusInfo
```fsharp
type StatusInfo = {
    currentTab: int
    totalTabs: int
    time: string
}
```

## API Reference

### Fusabi Functions

- `UIFormatting.onFormatTab : (TabInfo -> StatusSegment list) -> int`
  - Registers a formatter callback for tab rendering
  - Returns a handler ID

- `UIFormatting.onFormatStatusLeft : (StatusInfo -> StatusSegment list) -> int`
  - Registers a formatter callback for left status area
  - Returns a handler ID

- `UIFormatting.onFormatStatusRight : (StatusInfo -> StatusSegment list) -> int`
  - Registers a formatter callback for right status area
  - Returns a handler ID

- `UIFormatting.removeFormatter : int -> bool`
  - Removes a formatter by its handler ID
  - Returns true if removed, false if not found

- `UIFormatting.clearFormatters : unit -> unit`
  - Removes all registered formatters

### Rust Host Functions

- `create_tab_info(index, title, active, has_activity) -> Value`
  - Creates a TabInfo record

- `create_status_info(current_tab, total_tabs, time) -> Value`
  - Creates a StatusInfo record

- `extract_status_segment(segment) -> Result<(String, Option<String>, Option<String>, bool)>`
  - Extracts components from a StatusSegment

- `invoke_tab_formatters(vm, tab_info) -> Result<Vec<Vec<Value>>>`
  - Invokes all registered tab formatters

- `invoke_status_left_formatters(vm, status_info) -> Result<Vec<Vec<Value>>>`
  - Invokes all registered left status formatters

- `invoke_status_right_formatters(vm, status_info) -> Result<Vec<Vec<Value>>>`
  - Invokes all registered right status formatters
