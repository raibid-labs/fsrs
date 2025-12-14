// Scarab Terminal Status Bar
// Demonstrates a real-world TUI status bar for terminal emulator
//
// Scarab is Raibid Labs' high-performance split-process terminal emulator
// built in Rust with Bevy rendering. This status bar provides essential
// terminal information at a glance.
//
// This status bar shows:
// - Current working directory
// - Git branch and status indicator
// - CPU and memory mini gauges
// - Current time display
// - Active terminal tabs count
// - Process count indicator

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// Git repository status
type GitStatus =
    | Clean        // No changes
    | Modified     // Files modified
    | Staged       // Changes staged
    | Conflict     // Merge conflict
    | NotRepo      // Not a git repository

// Terminal tab state
type TabState =
    | Active
    | Background
    | Bell         // Has unread bell notification

// ============================================================================
// Helper Functions
// ============================================================================

// Get git status color
let getGitStatusColor status =
    match status with
    | Clean -> green
    | Modified -> yellow
    | Staged -> cyan
    | Conflict -> red
    | NotRepo -> white

// Get git status symbol
let getGitStatusSymbol status =
    match status with
    | Clean -> "✓"
    | Modified -> "●"
    | Staged -> "+"
    | Conflict -> "✗"
    | NotRepo -> "-"

// Format directory path (truncate if too long)
let formatPath path =
    // In real implementation, would truncate to fit width
    path

// ============================================================================
// Current Directory Display
// ============================================================================

let createDirectoryDisplay =
    let dirBlock = createBlock
                   |> withBorders rightBorder
                   |> withBorderType plainBorder in

    let dirText = textFromString "/home/user/raibid-labs/scarab" in
    let dirStyle = emptyStyle |> withFg cyan |> withBold in

    createParagraph dirText
    |> withParagraphBlock dirBlock
    |> withParagraphStyle dirStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Git Branch Indicator
// ============================================================================

let createGitBranchDisplay =
    let gitBlock = createBlock
                   |> withBorders rightBorder
                   |> withBorderType plainBorder in

    let gitStatus = Modified in
    let statusColor = getGitStatusColor gitStatus in
    let statusSymbol = getGitStatusSymbol gitStatus in

    let gitText = textFromString ("main " + statusSymbol) in
    let gitStyle = emptyStyle |> withFg statusColor in

    createParagraph gitText
    |> withParagraphBlock gitBlock
    |> withParagraphStyle gitStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// CPU Mini Gauge
// ============================================================================

let createCpuMiniGauge =
    let gaugeBlock = createBlock
                     |> withBorders rightBorder
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 42
    |> withLabel "CPU"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Memory Mini Gauge
// ============================================================================

let createMemoryMiniGauge =
    let gaugeBlock = createBlock
                     |> withBorders rightBorder
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 68
    |> withLabel "MEM"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Time Display
// ============================================================================

let createTimeDisplay =
    let timeBlock = createBlock
                    |> withBorders rightBorder
                    |> withBorderType plainBorder in

    let timeText = textFromString "14:23:45" in
    let timeStyle = emptyStyle |> withFg white in

    createParagraph timeText
    |> withParagraphBlock timeBlock
    |> withParagraphStyle timeStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Tab Count Indicator
// ============================================================================

let createTabCountDisplay =
    let tabBlock = createBlock
                   |> withBorders rightBorder
                   |> withBorderType plainBorder in

    let tabText = textFromString "Tabs: 4" in
    let tabStyle = emptyStyle |> withFg magenta in

    createParagraph tabText
    |> withParagraphBlock tabBlock
    |> withParagraphStyle tabStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Process Count Indicator
// ============================================================================

let createProcessCountDisplay =
    let procBlock = createBlock
                    |> withBorders noBorders in

    let procText = textFromString "Procs: 12" in
    let procStyle = emptyStyle |> withFg cyan in

    createParagraph procText
    |> withParagraphBlock procBlock
    |> withParagraphStyle procStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Session Name Display
// ============================================================================

let createSessionDisplay =
    let sessionBlock = createBlock
                       |> withBorders rightBorder
                       |> withBorderType plainBorder in

    let sessionText = textFromString "dev-session" in
    let sessionStyle = emptyStyle |> withFg green |> withBold in

    createParagraph sessionText
    |> withParagraphBlock sessionBlock
    |> withParagraphStyle sessionStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Shell Indicator
// ============================================================================

let createShellDisplay =
    let shellBlock = createBlock
                     |> withBorders rightBorder
                     |> withBorderType plainBorder in

    let shellText = textFromString "zsh" in
    let shellStyle = emptyStyle |> withFg white in

    createParagraph shellText
    |> withParagraphBlock shellBlock
    |> withParagraphStyle shellStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Network Indicator
// ============================================================================

let createNetworkDisplay =
    let networkBlock = createBlock
                       |> withBorders rightBorder
                       |> withBorderType plainBorder in

    let networkText = textFromString "WiFi ▲12KB ▼45KB" in
    let networkStyle = emptyStyle |> withFg green in

    createParagraph networkText
    |> withParagraphBlock networkBlock
    |> withParagraphStyle networkStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Battery Indicator (for laptops)
// ============================================================================

let createBatteryGauge =
    let gaugeBlock = createBlock
                     |> withBorders rightBorder
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 78
    |> withLabel "BAT"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// PTY Process List (for status expansion)
// ============================================================================

let createPtyProcessList =
    let proc1 = listItemFromString "PTY 0: /bin/zsh - Active" in
    let proc2 = listItemFromString "PTY 1: cargo run - Running" in
    let proc3 = listItemFromString "PTY 2: nvim - Active" in

    let listBlock = createBlock
                    |> withTitle "Active PTY Processes"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList proc1
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Tab List Widget
// ============================================================================

let createTabList =
    let tab1 = styledListItem "1: ~/raibid-labs/scarab" (emptyStyle |> withFg green) in
    let tab2 = styledListItem "2: ~/raibid-labs/sigilforge" (emptyStyle |> withFg white) in
    let tab3 = styledListItem "3: ~/raibid-labs/scryforge" (emptyStyle |> withFg white) in
    let tab4 = styledListItem "4: ~/raibid-labs/phage (bell)" (emptyStyle |> withFg yellow) in

    let listBlock = createBlock
                    |> withTitle "Terminal Tabs"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg green in

    createList tab1
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Status Bar Container
// ============================================================================

let createStatusBarContainer =
    let containerBlock = createBlock
                         |> withBorders topBorder
                         |> withBorderType plainBorder
                         |> withBlockStyle (emptyStyle |> withFg white |> withBg black) in
    containerBlock

// ============================================================================
// Full Status Dashboard (expanded view)
// ============================================================================

let createStatusDashboard =
    let dashboardBlock = createBlock
                         |> withTitle "Scarab Terminal Status"
                         |> withBorders allBorders
                         |> withBorderType doubleBorder
                         |> withBlockStyle (emptyStyle |> withFg cyan) in
    dashboardBlock

// ============================================================================
// Git Status Details Table
// ============================================================================

let createGitStatusTable =
    let headerCell = styledTableCell "File" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell1 = tableCellFromString "src/main.rs - Modified" in
    let dataRow1 = tableRowFromCell dataCell1 in

    let tableBlock = createBlock
                     |> withTitle "Git Status Details"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    createTable dataRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 100)

// ============================================================================
// Main Status Bar Assembly
// ============================================================================

// Compact status bar for single-line display
let scarabStatusBar =
    let container = createStatusBarContainer in

    // Widgets for horizontal layout (left to right):
    // [Session] | [Directory] | [Git Branch] | [CPU] | [MEM] | [Tabs] | [Procs] | [Time]

    container

// Expanded status dashboard for detailed view
let scarabStatusDashboard =
    let dashboard = createStatusDashboard in

    // Widgets for expanded view:
    // - Top row: Session, directory, git, time
    // - Middle row: CPU/MEM gauges, network, battery
    // - Bottom section: Tab list, PTY processes, git details

    dashboard

// Export all status widgets
let directoryDisplay = createDirectoryDisplay
let gitBranchDisplay = createGitBranchDisplay
let cpuMiniGauge = createCpuMiniGauge
let memoryMiniGauge = createMemoryMiniGauge
let timeDisplay = createTimeDisplay
let tabCountDisplay = createTabCountDisplay
let processCountDisplay = createProcessCountDisplay
let sessionDisplay = createSessionDisplay
let shellDisplay = createShellDisplay
let networkDisplay = createNetworkDisplay
let batteryGauge = createBatteryGauge
let ptyProcessList = createPtyProcessList
let tabList = createTabList
let gitStatusTable = createGitStatusTable

// Status bar metadata
let statusBarTitle = "Scarab Terminal Status Bar"
let statusBarVersion = "v0.2.0"
let statusBarDescription = "Compact status information for terminal emulator"
