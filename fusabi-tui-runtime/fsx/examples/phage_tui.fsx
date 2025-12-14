// Phage Context Visualization Dashboard
// Demonstrates a real-world TUI for context management and memory monitoring
//
// Phage is Raibid Labs' intelligent context management system that maintains
// semantic memory across conversations, tracks topic evolution, and provides
// context-aware assistance.
//
// This dashboard shows:
// - Memory usage and allocation
// - Active contexts with topic tracking
// - Recent topic events and transitions
// - Context tree visualization (nested context blocks)
// - Memory persistence status

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// Context status
type ContextStatus =
    | Active       // Currently being used
    | Idle         // In memory but not active
    | Archived     // Saved to disk
    | Expired      // Marked for cleanup

// Context type
type ContextType =
    | Conversation  // User conversation context
    | Project      // Project-specific context
    | Topic        // Topic-focused context
    | Session      // Session-scoped context

// Topic event type
type TopicEvent =
    | TopicCreated
    | TopicUpdated
    | TopicMerged
    | TopicSplit

// ============================================================================
// Helper Functions
// ============================================================================

// Get context status color
let getContextStatusColor status =
    match status with
    | Active -> green
    | Idle -> yellow
    | Archived -> blue
    | Expired -> red

// Get context type color
let getContextTypeColor contextType =
    match contextType with
    | Conversation -> cyan
    | Project -> magenta
    | Topic -> yellow
    | Session -> white

// Format memory size
let formatMemorySize bytes =
    if bytes >= 1048576 then
        let mb = bytes / 1048576 in
        string mb + " MB"
    else if bytes >= 1024 then
        let kb = bytes / 1024 in
        string kb + " KB"
    else
        string bytes + " B"

// ============================================================================
// Title Block
// ============================================================================

let createTitleBlock =
    let titleStyle = emptyStyle |> withFg yellow |> withBg black in
    createBlock
    |> withTitle "Phage Context Management Dashboard"
    |> withBorders allBorders
    |> withBorderType doubleBorder
    |> withBlockStyle titleStyle

// ============================================================================
// Memory Usage Gauge
// ============================================================================

let createMemoryGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Memory Usage"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 68
    |> withLabel "272 MB / 400 MB allocated"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Context Allocation Gauge
// ============================================================================

let createContextAllocationGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Active Contexts"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 45
    |> withLabel "18 / 40 contexts active"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Active Contexts List
// ============================================================================

// Conversation context
let context1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "[ACTIVE]      fusabi-tui-runtime     (Conversation)" in
    styledListItem itemText itemStyle

// Project context
let context2Item =
    let itemStyle = emptyStyle |> withFg magenta in
    let itemText = "[ACTIVE]      sigilforge-oauth       (Project)" in
    styledListItem itemText itemStyle

// Topic context
let context3Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "[IDLE]        rust-async-patterns    (Topic)" in
    styledListItem itemText itemStyle

// Archived context
let context4Item =
    let itemStyle = emptyStyle |> withFg blue in
    let itemText = "[ARCHIVED]    gpu-optimization       (Project)" in
    styledListItem itemText itemStyle

let createContextList =
    let listBlock = createBlock
                    |> withTitle "Active Contexts (18)"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder
                    |> withBlockStyle (emptyStyle |> withFg white) in

    let highlightStyle = emptyStyle |> withFg black |> withBg yellow in

    createList context1Item
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Topic Event Stream
// ============================================================================

let event1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "[10:45:23] Created: tui-dashboard-examples" in
    styledListItem itemText itemStyle

let event2Item =
    let itemStyle = emptyStyle |> withFg cyan in
    let itemText = "[10:42:11] Updated: fusabi-fsx-bindings" in
    styledListItem itemText itemStyle

let event3Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "[10:38:05] Merged: oauth-integration + token-mgmt" in
    styledListItem itemText itemStyle

let event4Item =
    let itemStyle = emptyStyle |> withFg magenta in
    let itemText = "[10:30:42] Split: youtube-analytics from api-design" in
    styledListItem itemText itemStyle

let createEventStream =
    let eventBlock = createBlock
                     |> withTitle "Recent Topic Events"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList event1Item
    |> withListBlock eventBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Context Tree Visualization
// ============================================================================

let createContextTree =
    let treeBlock = createBlock
                    |> withTitle "Context Hierarchy"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let treeText = textFromString "raibid-labs/\n  sigilforge/\n    oauth-manager\n    token-refresh\n  scryforge/\n    youtube-api\n    analytics\n  phage/\n    context-mgmt\n    memory-pool" in
    let treeStyle = emptyStyle |> withFg cyan in

    createParagraph treeText
    |> withParagraphBlock treeBlock
    |> withParagraphStyle treeStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Memory Allocation Sparkline
// ============================================================================

let createMemorySparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Memory Usage (24h)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg yellow in

    sparklineFromData 68
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Context Persistence Status
// ============================================================================

let createPersistenceStatus =
    let statusBlock = createBlock
                      |> withTitle "Persistence Status"
                      |> withBorders allBorders
                      |> withBorderType roundedBorder in

    let statusText = textFromString "Last backup: 2 minutes ago\nPending writes: 3 contexts\nSync status: UP TO DATE" in
    let statusStyle = emptyStyle |> withFg green in

    createParagraph statusText
    |> withParagraphBlock statusBlock
    |> withParagraphStyle statusStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Topic Statistics Table
// ============================================================================

let createTopicStatsTable =
    let headerCell = styledTableCell "Metric" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell1 = tableCellFromString "Active Topics: 42" in
    let dataRow1 = tableRowFromCell dataCell1 in

    let tableBlock = createBlock
                     |> withTitle "Context Statistics"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in

    createTable dataRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 50)

// ============================================================================
// Context Activity Sparkline
// ============================================================================

let createActivitySparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Context Activity (7 days)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg green in

    sparklineFromData 82
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Navigation Tabs
// ============================================================================

let createNavigationTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg yellow |> withBold in

    tabsFromTitles "Overview"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// ============================================================================
// Cache Hit Rate Gauge
// ============================================================================

let createCacheGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Cache Hit Rate"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 94
    |> withLabel "94% hit rate (Excellent!)"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Help Section
// ============================================================================

let createHelpSection =
    let helpBlock = createBlock
                    |> withTitle "Keyboard Shortcuts"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let helpText = textFromString "c: Contexts | t: Topics | m: Memory | s: Search | p: Persist | q: Quit" in
    let helpStyle = emptyStyle |> withFg yellow in

    createParagraph helpText
    |> withParagraphBlock helpBlock
    |> withParagraphStyle helpStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Main Dashboard Assembly
// ============================================================================

// This would be rendered using the TUI runtime with proper layout
let phageDashboard =
    let mainBlock = createTitleBlock in

    // Widgets ready for layout composition:
    // - Top: Title block, navigation tabs
    // - Left column: Context list, context tree
    // - Middle column: Memory gauges, sparklines
    // - Right column: Event stream, persistence status, statistics
    // - Bottom: Help section

    mainBlock

// Export main widgets
let memoryGauge = createMemoryGauge
let contextAllocationGauge = createContextAllocationGauge
let contextList = createContextList
let eventStream = createEventStream
let contextTree = createContextTree
let memorySparkline = createMemorySparkline
let persistenceStatus = createPersistenceStatus
let topicStatsTable = createTopicStatsTable
let activitySparkline = createActivitySparkline
let navigationTabs = createNavigationTabs
let cacheGauge = createCacheGauge
let helpSection = createHelpSection

// Dashboard metadata
let dashboardTitle = "Phage Context Management Dashboard"
let dashboardVersion = "v3.0.0"
let dashboardDescription = "Real-time context monitoring and memory management"
