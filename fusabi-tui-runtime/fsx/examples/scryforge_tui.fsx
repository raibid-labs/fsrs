// Scryforge YouTube Analytics Dashboard
// Demonstrates a real-world TUI for YouTube channel analytics
//
// Scryforge is Raibid Labs' YouTube analytics and management service
// that provides insights into channel performance, video metrics, and audience engagement.
//
// This dashboard shows:
// - Video performance list with view counts
// - Subscriber growth sparkline
// - Watch time gauge
// - Recent comments and engagement
// - Upload schedule and planning tabs

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// Video status
type VideoStatus =
    | Published
    | Scheduled
    | Draft
    | Processing

// Video performance metrics
type VideoMetric = {
    title: string
    views: int
    likes: int
    comments: int
    status: VideoStatus
}

// ============================================================================
// Helper Functions
// ============================================================================

// Format view count with K/M suffix
let formatViews views =
    if views >= 1000000 then
        let millions = views / 1000000 in
        string millions + "M"
    else if views >= 1000 then
        let thousands = views / 1000 in
        string thousands + "K"
    else
        string views

// Get status color
let getStatusColor status =
    match status with
    | Published -> green
    | Scheduled -> yellow
    | Draft -> white
    | Processing -> cyan

// ============================================================================
// Title Block
// ============================================================================

let createTitleBlock =
    let titleStyle = emptyStyle |> withFg magenta |> withBg black in
    createBlock
    |> withTitle "Scryforge YouTube Analytics Dashboard"
    |> withBorders allBorders
    |> withBorderType doubleBorder
    |> withBlockStyle titleStyle

// ============================================================================
// Channel Stats Summary
// ============================================================================

let createChannelStats =
    let statsBlock = createBlock
                     |> withTitle "Channel Overview"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    let statsText = textFromString "Subscribers: 125.4K | Total Views: 4.2M | Videos: 87" in
    let statsStyle = emptyStyle |> withFg cyan |> withBold in

    createParagraph statsText
    |> withParagraphBlock statsBlock
    |> withParagraphStyle statsStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Video Performance List
// ============================================================================

// Top performing video
let video1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "Rust Tutorial Series Ep.1          2.4M views   12K likes" in
    styledListItem itemText itemStyle

// Recent upload
let video2Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "GPU Programming with CUDA          142K views   3.2K likes" in
    styledListItem itemText itemStyle

// Older video
let video3Item =
    let itemStyle = emptyStyle |> withFg white in
    let itemText = "F# for Beginners                   78K views    1.8K likes" in
    styledListItem itemText itemStyle

// Scheduled video
let video4Item =
    let itemStyle = emptyStyle |> withFg cyan in
    let itemText = "Docker Compose Advanced            [SCHEDULED]  Dec 20" in
    styledListItem itemText itemStyle

let createVideoList =
    let listBlock = createBlock
                    |> withTitle "Recent Videos"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder
                    |> withBlockStyle (emptyStyle |> withFg white) in

    let highlightStyle = emptyStyle |> withFg black |> withBg magenta in

    createList video1Item
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Subscriber Growth Sparkline
// ============================================================================

let createSubscriberSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Subscriber Growth (30 days)"
                         |> withBorders allBorders
                         |> withBorderType roundedBorder in
    let sparklineStyle = emptyStyle |> withFg green in

    sparklineFromData 85
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Watch Time Gauge
// ============================================================================

let createWatchTimeGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Watch Time Goal (Monthly)"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 72
    |> withLabel "288K hrs / 400K hrs target"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Engagement Rate Gauge
// ============================================================================

let createEngagementGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Engagement Rate"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 86
    |> withLabel "8.6% engagement (Excellent!)"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Recent Comments
// ============================================================================

let comment1Item =
    let itemStyle = emptyStyle |> withFg white in
    let itemText = "@techfan42: Amazing tutorial! Please do more Rust content!" in
    styledListItem itemText itemStyle

let comment2Item =
    let itemStyle = emptyStyle |> withFg white in
    let itemText = "@coder_jane: GPU programming section was super helpful" in
    styledListItem itemText itemStyle

let comment3Item =
    let itemStyle = emptyStyle |> withFg white in
    let itemText = "@rustacean99: When is the next episode coming?" in
    styledListItem itemText itemStyle

let createCommentsList =
    let commentsBlock = createBlock
                        |> withTitle "Recent Comments"
                        |> withBorders allBorders
                        |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList comment1Item
    |> withListBlock commentsBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Upload Schedule Tabs
// ============================================================================

let createScheduleTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg magenta |> withBold in

    tabsFromTitles "Overview"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// ============================================================================
// Top Videos Table
// ============================================================================

let createTopVideosTable =
    let headerCell = styledTableCell "Video Title" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let dataCell1 = tableCellFromString "Rust Tutorial Series Ep.1 - 2.4M views" in
    let dataRow1 = tableRowFromCell dataCell1 in

    let tableBlock = createBlock
                     |> withTitle "Top 10 Videos This Month"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    createTable dataRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 100)

// ============================================================================
// Revenue Gauge (if monetized)
// ============================================================================

let createRevenueGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Estimated Revenue (Monthly)"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 65
    |> withLabel "$2,600 / $4,000 target"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Upload Calendar View
// ============================================================================

let createUploadCalendar =
    let calendarBlock = createBlock
                        |> withTitle "Upload Schedule"
                        |> withBorders allBorders
                        |> withBorderType roundedBorder in

    let calendarText = textFromString "Mon: -\nTue: Rust Tutorial Ep.2\nWed: -\nThu: Live Stream\nFri: Docker Video" in
    let calendarStyle = emptyStyle |> withFg white in

    createParagraph calendarText
    |> withParagraphBlock calendarBlock
    |> withParagraphStyle calendarStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Traffic Source Breakdown
// ============================================================================

let createTrafficSourceList =
    let trafficBlock = createBlock
                       |> withTitle "Traffic Sources"
                       |> withBorders allBorders
                       |> withBorderType roundedBorder in

    let source1 = listItemFromString "YouTube Search: 42%" in
    let highlightStyle = emptyStyle |> withFg black |> withBg yellow in

    createList source1
    |> withListBlock trafficBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// View Duration Sparkline
// ============================================================================

let createDurationSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Avg View Duration (7 days)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg cyan in

    sparklineFromData 78
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

    let helpText = textFromString "v: Videos | s: Schedule | a: Analytics | c: Comments | q: Quit" in
    let helpStyle = emptyStyle |> withFg magenta in

    createParagraph helpText
    |> withParagraphBlock helpBlock
    |> withParagraphStyle helpStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Main Dashboard Assembly
// ============================================================================

// This would be rendered using the TUI runtime with proper layout
let scryforgeDashboard =
    let mainBlock = createTitleBlock in

    // Widgets ready for layout composition:
    // - Top: Title block, channel stats, tabs
    // - Left column: Video list, comments list
    // - Middle column: Subscriber sparkline, watch time gauge, engagement gauge
    // - Right column: Upload calendar, traffic sources, revenue gauge
    // - Bottom: Help section

    mainBlock

// Export main widgets
let channelStats = createChannelStats
let videoList = createVideoList
let subscriberSparkline = createSubscriberSparkline
let watchTimeGauge = createWatchTimeGauge
let engagementGauge = createEngagementGauge
let commentsList = createCommentsList
let scheduleTabs = createScheduleTabs
let topVideosTable = createTopVideosTable
let revenueGauge = createRevenueGauge
let uploadCalendar = createUploadCalendar
let trafficSourceList = createTrafficSourceList
let durationSparkline = createDurationSparkline
let helpSection = createHelpSection

// Dashboard metadata
let dashboardTitle = "Scryforge YouTube Analytics Dashboard"
let dashboardVersion = "v2.1.0"
let dashboardDescription = "Real-time YouTube channel analytics and content management"
