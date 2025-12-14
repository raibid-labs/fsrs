// TUI Dashboard Example
// Demonstrates using multiple widgets to create a dashboard layout

#load "../tui.fsx"

// Create a styled title block
let createTitleBlock title =
    let titleStyle = emptyStyle |> withFg cyan |> withBg black in
    createBlock
    |> withTitle title
    |> withBorders allBorders
    |> withBorderType roundedBorder
    |> withBlockStyle titleStyle

// Example 1: Block with title and borders
let statusBlock =
    createTitleBlock "System Status"
    |> withPadding 1 1 0 0

// Example 2: List with items
let createStatusList =
    let item1 = listItemFromString "CPU Usage: 45%" in
    let item2Style = emptyStyle |> withFg green in
    let item2 = styledListItem "Memory: 2.4GB / 8GB" item2Style in
    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList item1
    |> withListBlock statusBlock
    |> withHighlightStyle highlightStyle

// Example 3: Paragraph with styled text
let createLogParagraph =
    let logText = textFromString "Application started successfully. All systems operational." in
    let logStyle = emptyStyle |> withFg white in
    let logBlock = createTitleBlock "Logs" in

    createParagraph logText
    |> withParagraphBlock logBlock
    |> withParagraphStyle logStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// Example 4: Gauge showing progress
let createCpuGauge =
    let gaugeBlock = createTitleBlock "CPU Load" in
    let gaugeStyle = emptyStyle |> withFg yellow in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 45
    |> withLabel "Processing..."
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// Example 5: Gauge showing memory usage
let createMemoryGauge =
    let gaugeBlock = createTitleBlock "Memory Usage" in
    let gaugeStyle = emptyStyle |> withFg blue in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 30
    |> withLabel "2.4GB / 8GB"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// Example 6: Sparkline for showing trends
let createNetworkSparkline =
    let sparklineBlock = createTitleBlock "Network Activity" in
    let sparklineStyle = emptyStyle |> withFg magenta in

    sparklineFromData 75
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// Example 7: Tabs for navigation
let createNavTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg cyan in

    tabsFromTitles "Dashboard"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// Example 8: Table for displaying data
let createDataTable =
    let headerCell = styledTableCell "Metric" (emptyStyle |> withFg yellow) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell = tableCellFromString "CPU: 45%" in
    let dataRow = tableRowFromCell dataCell in

    let tableBlock = createTitleBlock "System Metrics" in

    createTable dataRow
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 50)

// Main dashboard layout
let createDashboard =
    let mainBlock = createTitleBlock "Fusabi TUI Dashboard v0.1.0" in

    // Create layout areas
    let area = createRect 0 0 80 24 in
    let layoutMargin = 1 in

    // Layout constraints
    let verticalConstraints = [Percentage 30; Percentage 40; Percentage 30] in
    let horizontalConstraints = [Percentage 50; Percentage 50] in

    // This is a simplified example showing widget creation
    // Actual rendering would require integration with the TUI runtime
    mainBlock

// Example usage
let dashboard = createDashboard
let statusList = createStatusList
let logParagraph = createLogParagraph
let cpuGauge = createCpuGauge
let memoryGauge = createMemoryGauge
let networkSparkline = createNetworkSparkline
let navTabs = createNavTabs
let dataTable = createDataTable

// Display message
let dashboardMessage = "Dashboard widgets created successfully!"
