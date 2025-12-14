// Hibana GPU Metrics Dashboard
// Demonstrates a real-world TUI for GPU monitoring and metrics
//
// Hibana is Raibid Labs' GPU compute orchestration system for managing
// distributed training workloads, model inference, and GPU resource allocation.
//
// This dashboard shows:
// - Multi-GPU utilization gauges
// - VRAM memory usage per GPU
// - Temperature monitoring with sparklines
// - Power consumption tracking
// - Running process list with GPU assignment
// - Compute performance metrics

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// GPU status
type GpuStatus =
    | Idle           // < 10% utilization
    | Active         // 10-80% utilization
    | HighLoad       // 80-95% utilization
    | Maxed          // > 95% utilization
    | Error          // GPU error state

// Process priority
type ProcessPriority =
    | High
    | Normal
    | Low

// GPU process type
type ProcessType =
    | Training       // Model training
    | Inference      // Model inference
    | Compute        // General compute
    | Graphics       // Graphics rendering

// ============================================================================
// Helper Functions
// ============================================================================

// Get GPU status color
let getGpuStatusColor status =
    match status with
    | Idle -> white
    | Active -> green
    | HighLoad -> yellow
    | Maxed -> red
    | Error -> red

// Get process type color
let getProcessTypeColor processType =
    match processType with
    | Training -> magenta
    | Inference -> cyan
    | Compute -> green
    | Graphics -> yellow

// Format memory size
let formatMemoryMB mb =
    if mb >= 1024 then
        let gb = mb / 1024 in
        string gb + " GB"
    else
        string mb + " MB"

// Format temperature
let formatTemp celsius =
    string celsius + "°C"

// ============================================================================
// Title Block
// ============================================================================

let createTitleBlock =
    let titleStyle = emptyStyle |> withFg red |> withBg black in
    createBlock
    |> withTitle "Hibana GPU Metrics Dashboard"
    |> withBorders allBorders
    |> withBorderType doubleBorder
    |> withBlockStyle titleStyle

// ============================================================================
// GPU 0 Utilization Gauge
// ============================================================================

let createGpu0UtilGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 0 - NVIDIA RTX 4090"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 78
    |> withLabel "78% utilized - Training Active"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// GPU 1 Utilization Gauge
// ============================================================================

let createGpu1UtilGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 1 - NVIDIA RTX 4090"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 92
    |> withLabel "92% utilized - Inference Active"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// GPU 2 Utilization Gauge
// ============================================================================

let createGpu2UtilGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 2 - NVIDIA RTX 4090"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg white in

    gaugeFromPercent 8
    |> withLabel "8% utilized - Idle"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// GPU 3 Utilization Gauge
// ============================================================================

let createGpu3UtilGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 3 - NVIDIA RTX 4090"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg red in

    gaugeFromPercent 99
    |> withLabel "99% utilized - MAXED OUT"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// VRAM Memory Usage - GPU 0
// ============================================================================

let createGpu0VramGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 0 VRAM"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 65
    |> withLabel "15.6 GB / 24 GB"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// VRAM Memory Usage - GPU 1
// ============================================================================

let createGpu1VramGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GPU 1 VRAM"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 82
    |> withLabel "19.7 GB / 24 GB"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Temperature Sparklines
// ============================================================================

let createGpu0TempSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "GPU 0 Temperature (1h)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg yellow in

    sparklineFromData 68
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

let createGpu1TempSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "GPU 1 Temperature (1h)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg red in

    sparklineFromData 78
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Power Consumption Gauge
// ============================================================================

let createPowerGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Total Power Consumption"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 74
    |> withLabel "890W / 1200W (4 GPUs)"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// GPU Process List
// ============================================================================

let proc1Item =
    let itemStyle = emptyStyle |> withFg magenta in
    let itemText = "GPU 0 | PID 12345 | llama-train     15.2GB  [TRAINING]" in
    styledListItem itemText itemStyle

let proc2Item =
    let itemStyle = emptyStyle |> withFg cyan in
    let itemText = "GPU 1 | PID 12346 | stable-diffusion 18.4GB  [INFERENCE]" in
    styledListItem itemText itemStyle

let proc3Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "GPU 3 | PID 12347 | pytorch-train    23.1GB  [TRAINING]" in
    styledListItem itemText itemStyle

let createProcessList =
    let listBlock = createBlock
                    |> withTitle "Active GPU Processes"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder
                    |> withBlockStyle (emptyStyle |> withFg white) in

    let highlightStyle = emptyStyle |> withFg black |> withBg red in

    createList proc1Item
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Compute Performance Table
// ============================================================================

let createPerformanceTable =
    let headerCell = styledTableCell "Metric" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell1 = tableCellFromString "Total TFLOPS: 320.4" in
    let dataRow1 = tableRowFromCell dataCell1 in

    let tableBlock = createBlock
                     |> withTitle "Compute Performance"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    createTable dataRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 50)

// ============================================================================
// Navigation Tabs
// ============================================================================

let createNavigationTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg red |> withBold in

    tabsFromTitles "Overview"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// ============================================================================
// GPU Health Status
// ============================================================================

let createHealthStatus =
    let healthBlock = createBlock
                      |> withTitle "System Health"
                      |> withBorders allBorders
                      |> withBorderType roundedBorder in

    let healthText = textFromString "Status: OPERATIONAL\nWarnings: GPU 1 temp high (78°C)\nErrors: None" in
    let healthStyle = emptyStyle |> withFg green in

    createParagraph healthText
    |> withParagraphBlock healthBlock
    |> withParagraphStyle healthStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// PCIe Bandwidth Sparkline
// ============================================================================

let createPcieBandwidthSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "PCIe Bandwidth Usage (10min)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg cyan in

    sparklineFromData 72
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Fan Speed Gauge
// ============================================================================

let createFanSpeedGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Average Fan Speed"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 68
    |> withLabel "2720 RPM (68%)"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Clock Speed Display
// ============================================================================

let createClockSpeedDisplay =
    let clockBlock = createBlock
                     |> withTitle "GPU Clock Speeds"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    let clockText = textFromString "GPU 0: 2520 MHz\nGPU 1: 2535 MHz\nGPU 2: 1410 MHz\nGPU 3: 2550 MHz" in
    let clockStyle = emptyStyle |> withFg cyan in

    createParagraph clockText
    |> withParagraphBlock clockBlock
    |> withParagraphStyle clockStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Help Section
// ============================================================================

let createHelpSection =
    let helpBlock = createBlock
                    |> withTitle "Keyboard Shortcuts"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let helpText = textFromString "g: GPU Details | p: Processes | m: Memory | t: Temperature | k: Kill Process | q: Quit" in
    let helpStyle = emptyStyle |> withFg red in

    createParagraph helpText
    |> withParagraphBlock helpBlock
    |> withParagraphStyle helpStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Main Dashboard Assembly
// ============================================================================

// This would be rendered using the TUI runtime with proper layout
let hibanaDashboard =
    let mainBlock = createTitleBlock in

    // Widgets ready for layout composition:
    // - Top: Title block, navigation tabs
    // - Grid layout (2x2): GPU utilization gauges for all 4 GPUs
    // - Row: VRAM usage gauges
    // - Row: Temperature sparklines
    // - Left column: Process list, health status
    // - Right column: Performance table, clock speeds
    // - Bottom: Power gauge, help section

    mainBlock

// Export main widgets
let gpu0UtilGauge = createGpu0UtilGauge
let gpu1UtilGauge = createGpu1UtilGauge
let gpu2UtilGauge = createGpu2UtilGauge
let gpu3UtilGauge = createGpu3UtilGauge
let gpu0VramGauge = createGpu0VramGauge
let gpu1VramGauge = createGpu1VramGauge
let gpu0TempSparkline = createGpu0TempSparkline
let gpu1TempSparkline = createGpu1TempSparkline
let powerGauge = createPowerGauge
let processList = createProcessList
let performanceTable = createPerformanceTable
let navigationTabs = createNavigationTabs
let healthStatus = createHealthStatus
let pcieBandwidthSparkline = createPcieBandwidthSparkline
let fanSpeedGauge = createFanSpeedGauge
let clockSpeedDisplay = createClockSpeedDisplay
let helpSection = createHelpSection

// Dashboard metadata
let dashboardTitle = "Hibana GPU Metrics Dashboard"
let dashboardVersion = "v1.5.0"
let dashboardDescription = "Real-time GPU monitoring and process management"
