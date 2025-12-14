// Sigilforge OAuth Token Management Dashboard
// Demonstrates a real-world TUI for managing OAuth providers and tokens
//
// Sigilforge is Raibid Labs' OAuth integration service that manages
// authentication tokens for GitHub, YouTube, and other providers.
//
// This dashboard shows:
// - Connected OAuth providers with status indicators
// - Token expiry monitoring with color-coded warnings
// - Login flow progress
// - Token refresh controls

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// OAuth Provider status
type ProviderStatus =
    | Connected      // Token valid
    | Expiring       // Token expires soon (< 7 days)
    | Expired        // Token expired
    | NotConnected   // No token

// Sample OAuth providers
type Provider = {
    name: string
    status: ProviderStatus
    daysUntilExpiry: int
}

// ============================================================================
// Helper Functions
// ============================================================================

// Get status color based on provider state
let getStatusColor status =
    match status with
    | Connected -> green
    | Expiring -> yellow
    | Expired -> red
    | NotConnected -> white

// Get status text
let getStatusText status =
    match status with
    | Connected -> "ACTIVE"
    | Expiring -> "EXPIRING SOON"
    | Expired -> "EXPIRED"
    | NotConnected -> "NOT CONNECTED"

// ============================================================================
// Title Block
// ============================================================================

let createTitleBlock =
    let titleStyle = emptyStyle |> withFg cyan |> withBg black in
    createBlock
    |> withTitle "Sigilforge OAuth Manager v1.2.0"
    |> withBorders allBorders
    |> withBorderType doubleBorder
    |> withBlockStyle titleStyle

// ============================================================================
// Provider List Widget
// ============================================================================

// GitHub provider
let githubProviderItem =
    let statusColor = getStatusColor Connected in
    let itemStyle = emptyStyle |> withFg statusColor in
    let itemText = "GitHub          [ACTIVE]          Expires: 45 days" in
    styledListItem itemText itemStyle

// YouTube provider
let youtubeProviderItem =
    let statusColor = getStatusColor Expiring in
    let itemStyle = emptyStyle |> withFg statusColor in
    let itemText = "YouTube         [EXPIRING SOON]   Expires: 5 days" in
    styledListItem itemText itemStyle

// Google Drive provider (not connected)
let driveProviderItem =
    let statusColor = getStatusColor NotConnected in
    let itemStyle = emptyStyle |> withFg statusColor in
    let itemText = "Google Drive    [NOT CONNECTED]   -" in
    styledListItem itemText itemStyle

// Spotify provider (expired)
let spotifyProviderItem =
    let statusColor = getStatusColor Expired in
    let itemStyle = emptyStyle |> withFg statusColor in
    let itemText = "Spotify         [EXPIRED]         Expired: 2 days ago" in
    styledListItem itemText itemStyle

let createProviderList =
    let listBlock = createBlock
                    |> withTitle "Connected Providers"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder
                    |> withBlockStyle (emptyStyle |> withFg white) in

    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList githubProviderItem
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Token Status Summary Gauges
// ============================================================================

// GitHub token health gauge
let createGitHubTokenGauge =
    let gaugeBlock = createBlock
                     |> withTitle "GitHub Token Health"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 90
    |> withLabel "90% valid (45 days left)"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// YouTube token health gauge
let createYouTubeTokenGauge =
    let gaugeBlock = createBlock
                     |> withTitle "YouTube Token Health"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 25
    |> withLabel "25% valid (5 days left) - REFRESH SOON"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Login Flow Status
// ============================================================================

let createLoginFlowStatus =
    let flowBlock = createBlock
                    |> withTitle "Active Login Flows"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let flowText = textFromString "No active login flows.\nPress 'a' to add new provider." in
    let flowStyle = emptyStyle |> withFg white in

    createParagraph flowText
    |> withParagraphBlock flowBlock
    |> withParagraphStyle flowStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Recent Activity Log
// ============================================================================

let createActivityLog =
    let logBlock = createBlock
                   |> withTitle "Recent Activity"
                   |> withBorders allBorders
                   |> withBorderType roundedBorder in

    let logItem = listItemFromString "[2025-12-14 10:30] GitHub token refreshed successfully" in
    let highlightStyle = emptyStyle |> withFg black |> withBg white in

    createList logItem
    |> withListBlock logBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Action Tabs
// ============================================================================

let createActionTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg cyan |> withBold in

    tabsFromTitles "Providers"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// ============================================================================
// Statistics Table
// ============================================================================

let createStatsTable =
    let headerCell = styledTableCell "Metric" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell1 = tableCellFromString "Total Providers: 4" in
    let dataRow1 = tableRowFromCell dataCell1 in

    let tableBlock = createBlock
                     |> withTitle "OAuth Statistics"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in

    createTable dataRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 50)

// ============================================================================
// Token Refresh Sparkline
// ============================================================================

let createRefreshSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Token Refresh Activity (7 days)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg green in

    sparklineFromData 65
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Help Section
// ============================================================================

let createHelpSection =
    let helpBlock = createBlock
                    |> withTitle "Keyboard Shortcuts"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let helpText = textFromString "a: Add Provider | r: Refresh Token | d: Disconnect | q: Quit" in
    let helpStyle = emptyStyle |> withFg cyan in

    createParagraph helpText
    |> withParagraphBlock helpBlock
    |> withParagraphStyle helpStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Main Dashboard Assembly
// ============================================================================

// This would be rendered using the TUI runtime with proper layout
let sigilforgeDashboard =
    let mainBlock = createTitleBlock in

    // Widgets ready for layout composition:
    // - Top: Title block and action tabs
    // - Left column: Provider list, login flow status
    // - Middle column: Token health gauges, refresh sparkline
    // - Right column: Activity log, statistics
    // - Bottom: Help section

    mainBlock

// Export main widgets
let providerList = createProviderList
let gitHubGauge = createGitHubTokenGauge
let youTubeGauge = createYouTubeTokenGauge
let loginFlow = createLoginFlowStatus
let activityLog = createActivityLog
let actionTabs = createActionTabs
let statsTable = createStatsTable
let refreshSparkline = createRefreshSparkline
let helpSection = createHelpSection

// Dashboard metadata
let dashboardTitle = "Sigilforge OAuth Token Management Dashboard"
let dashboardVersion = "v1.2.0"
let dashboardDescription = "Monitor and manage OAuth tokens for connected providers"
