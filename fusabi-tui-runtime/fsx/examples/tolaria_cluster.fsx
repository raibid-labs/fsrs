// Tolaria Kubernetes Cluster Monitor
// Demonstrates a real-world TUI for K8s cluster monitoring
//
// Tolaria is Raibid Labs' Kubernetes cluster orchestration and monitoring
// platform for managing distributed workloads across cloud infrastructure.
//
// This dashboard shows:
// - Node status table with health indicators
// - Pod list with status colors and resource usage
// - Namespace resource allocation gauges
// - Cluster event log stream
// - Deployment rollout status
// - Service endpoint health checks

#load "../tui.fsx"

// ============================================================================
// Data Models
// ============================================================================

// Node status
type NodeStatus =
    | Ready        // Node ready for workloads
    | NotReady     // Node not ready
    | Unknown      // Status unknown
    | Cordoned     // Node cordoned off

// Pod status
type PodStatus =
    | Running      // Pod running normally
    | Pending      // Pod pending scheduling
    | Failed       // Pod failed
    | Succeeded    // Pod completed successfully
    | CrashLoop    // CrashLoopBackOff

// Event severity
type EventSeverity =
    | Normal       // Normal informational event
    | Warning      // Warning event
    | Error        // Error event

// ============================================================================
// Helper Functions
// ============================================================================

// Get node status color
let getNodeStatusColor status =
    match status with
    | Ready -> green
    | NotReady -> red
    | Unknown -> yellow
    | Cordoned -> blue

// Get pod status color
let getPodStatusColor status =
    match status with
    | Running -> green
    | Pending -> yellow
    | Failed -> red
    | Succeeded -> blue
    | CrashLoop -> magenta

// Get event severity color
let getEventColor severity =
    match severity with
    | Normal -> white
    | Warning -> yellow
    | Error -> red

// Format resource (CPU/Memory)
let formatCpu milliCores =
    if milliCores >= 1000 then
        let cores = milliCores / 1000 in
        string cores + " cores"
    else
        string milliCores + "m"

// ============================================================================
// Title Block
// ============================================================================

let createTitleBlock =
    let titleStyle = emptyStyle |> withFg blue |> withBg black in
    createBlock
    |> withTitle "Tolaria Kubernetes Cluster Monitor"
    |> withBorders allBorders
    |> withBorderType doubleBorder
    |> withBlockStyle titleStyle

// ============================================================================
// Cluster Overview
// ============================================================================

let createClusterOverview =
    let overviewBlock = createBlock
                        |> withTitle "Cluster: production-us-east-1"
                        |> withBorders allBorders
                        |> withBorderType roundedBorder in

    let overviewText = textFromString "Nodes: 12 Ready | Pods: 247 Running | Namespaces: 8 Active" in
    let overviewStyle = emptyStyle |> withFg cyan |> withBold in

    createParagraph overviewText
    |> withParagraphBlock overviewBlock
    |> withParagraphStyle overviewStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Node Status Table
// ============================================================================

let createNodeTable =
    let headerCell = styledTableCell "Node Name" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let nodeCell1 = styledTableCell "k8s-node-01 [READY]  16 cores  64GB  10 pods" (emptyStyle |> withFg green) in
    let nodeRow1 = tableRowFromCell nodeCell1 in

    let tableBlock = createBlock
                     |> withTitle "Cluster Nodes"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    createTable nodeRow1
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg white)
    |> withColumnWidths (columnPercentage 100)

// ============================================================================
// Pod Status List
// ============================================================================

let pod1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "webapp-deployment-7d8f9c-abc12    [RUNNING]    200m/2Gi" in
    styledListItem itemText itemStyle

let pod2Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "api-service-5f6d8b-xyz89          [RUNNING]    500m/4Gi" in
    styledListItem itemText itemStyle

let pod3Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "worker-job-9a2b3c-def45            [PENDING]    0m/0Gi" in
    styledListItem itemText itemStyle

let pod4Item =
    let itemStyle = emptyStyle |> withFg magenta in
    let itemText = "cache-redis-4k3j2h-ghi78           [CRASHLOOP]  100m/512Mi" in
    styledListItem itemText itemStyle

let createPodList =
    let listBlock = createBlock
                    |> withTitle "Pods (default namespace)"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder
                    |> withBlockStyle (emptyStyle |> withFg white) in

    let highlightStyle = emptyStyle |> withFg black |> withBg blue in

    createList pod1Item
    |> withListBlock listBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// CPU Usage Gauge (Cluster-wide)
// ============================================================================

let createClusterCpuGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Cluster CPU Usage"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg green in

    gaugeFromPercent 62
    |> withLabel "119 / 192 cores allocated"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Memory Usage Gauge (Cluster-wide)
// ============================================================================

let createClusterMemoryGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Cluster Memory Usage"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg yellow in

    gaugeFromPercent 74
    |> withLabel "568 GB / 768 GB allocated"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Storage Usage Gauge
// ============================================================================

let createStorageGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Persistent Volume Storage"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 58
    |> withLabel "2.9 TB / 5.0 TB used"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Event Log Stream
// ============================================================================

let event1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "[NORMAL]  10:45 Pod webapp-7d8f9c-abc12 started" in
    styledListItem itemText itemStyle

let event2Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "[WARNING] 10:42 Node k8s-node-03 disk pressure" in
    styledListItem itemText itemStyle

let event3Item =
    let itemStyle = emptyStyle |> withFg red in
    let itemText = "[ERROR]   10:38 Pod cache-redis-4k3j2h CrashLoopBackOff" in
    styledListItem itemText itemStyle

let event4Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "[NORMAL]  10:30 Deployment api-service scaled to 5 replicas" in
    styledListItem itemText itemStyle

let createEventLog =
    let eventBlock = createBlock
                     |> withTitle "Cluster Events (Last 1h)"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg white in

    createList event1Item
    |> withListBlock eventBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Namespace Resource Breakdown
// ============================================================================

let ns1Item =
    let itemStyle = emptyStyle |> withFg cyan in
    let itemText = "default       120 pods   45 cores   180GB" in
    styledListItem itemText itemStyle

let ns2Item =
    let itemStyle = emptyStyle |> withFg magenta in
    let itemText = "kube-system    32 pods   12 cores    48GB" in
    styledListItem itemText itemStyle

let ns3Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "monitoring     18 pods    8 cores    32GB" in
    styledListItem itemText itemStyle

let createNamespaceList =
    let nsBlock = createBlock
                  |> withTitle "Namespace Resources"
                  |> withBorders allBorders
                  |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg cyan in

    createList ns1Item
    |> withListBlock nsBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// CPU Utilization Sparkline (24h trend)
// ============================================================================

let createCpuSparkline =
    let sparklineBlock = createBlock
                         |> withTitle "CPU Utilization Trend (24h)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg green in

    sparklineFromData 62
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Memory Utilization Sparkline (24h trend)
// ============================================================================

let createMemorySparkline =
    let sparklineBlock = createBlock
                         |> withTitle "Memory Utilization Trend (24h)"
                         |> withBorders allBorders
                         |> withBorderType plainBorder in
    let sparklineStyle = emptyStyle |> withFg yellow in

    sparklineFromData 74
    |> withMax 100
    |> withSparklineBlock sparklineBlock
    |> withSparklineStyle sparklineStyle
    |> withDirection leftToRight

// ============================================================================
// Deployment Status List
// ============================================================================

let deploy1Item =
    let itemStyle = emptyStyle |> withFg green in
    let itemText = "webapp-deployment     5/5 ready   [UP TO DATE]" in
    styledListItem itemText itemStyle

let deploy2Item =
    let itemStyle = emptyStyle |> withFg yellow in
    let itemText = "api-service           3/5 ready   [ROLLING UPDATE]" in
    styledListItem itemText itemStyle

let deploy3Item =
    let itemStyle = emptyStyle |> withFg red in
    let itemText = "worker-service        0/3 ready   [FAILED]" in
    styledListItem itemText itemStyle

let createDeploymentList =
    let deployBlock = createBlock
                      |> withTitle "Deployments"
                      |> withBorders allBorders
                      |> withBorderType roundedBorder in

    let highlightStyle = emptyStyle |> withFg black |> withBg green in

    createList deploy1Item
    |> withListBlock deployBlock
    |> withHighlightStyle highlightStyle

// ============================================================================
// Service Endpoint Health
// ============================================================================

let createServiceHealth =
    let healthBlock = createBlock
                      |> withTitle "Service Health"
                      |> withBorders allBorders
                      |> withBorderType roundedBorder in

    let healthText = textFromString "webapp-service: HEALTHY (5/5 endpoints)\napi-service: DEGRADED (3/5 endpoints)\nredis-service: DOWN (0/1 endpoints)" in
    let healthStyle = emptyStyle |> withFg white in

    createParagraph healthText
    |> withParagraphBlock healthBlock
    |> withParagraphStyle healthStyle
    |> withAlignment leftAlign
    |> withWrap wrapWord

// ============================================================================
// Navigation Tabs
// ============================================================================

let createNavigationTabs =
    let tabsBlock = createBlock |> withBorders bottomBorder in
    let normalStyle = emptyStyle |> withFg white in
    let highlightStyle = emptyStyle |> withFg blue |> withBold in

    tabsFromTitles "Overview"
    |> withSelected 0
    |> withDivider pipeDivider
    |> withTabsBlock tabsBlock
    |> withTabsStyle normalStyle
    |> withTabsHighlightStyle highlightStyle

// ============================================================================
// Network I/O Gauge
// ============================================================================

let createNetworkGauge =
    let gaugeBlock = createBlock
                     |> withTitle "Network I/O"
                     |> withBorders allBorders
                     |> withBorderType plainBorder in
    let gaugeStyle = emptyStyle |> withFg white in
    let barStyle = emptyStyle |> withFg cyan in

    gaugeFromPercent 45
    |> withLabel "4.5 Gbps / 10 Gbps capacity"
    |> withGaugeBlock gaugeBlock
    |> withGaugeStyle gaugeStyle
    |> withGaugeBarStyle barStyle

// ============================================================================
// Pod Restarts Table
// ============================================================================

let createRestartsTable =
    let headerCell = styledTableCell "Pod Name" (emptyStyle |> withFg yellow |> withBold) in
    let headerRow = createTableRow headerCell (emptyStyle |> withFg yellow) in

    let restartCell = tableCellFromString "cache-redis-4k3j2h - 15 restarts (CrashLoop)" in
    let restartRow = tableRowFromCell restartCell in

    let tableBlock = createBlock
                     |> withTitle "Pods with Restarts"
                     |> withBorders allBorders
                     |> withBorderType roundedBorder in

    createTable restartRow
    |> withTableHeader headerRow
    |> withTableBlock tableBlock
    |> withTableStyle (emptyStyle |> withFg red)
    |> withColumnWidths (columnPercentage 100)

// ============================================================================
// Help Section
// ============================================================================

let createHelpSection =
    let helpBlock = createBlock
                    |> withTitle "Keyboard Shortcuts"
                    |> withBorders allBorders
                    |> withBorderType roundedBorder in

    let helpText = textFromString "n: Nodes | p: Pods | d: Deployments | s: Services | l: Logs | e: Events | q: Quit" in
    let helpStyle = emptyStyle |> withFg blue in

    createParagraph helpText
    |> withParagraphBlock helpBlock
    |> withParagraphStyle helpStyle
    |> withAlignment centerAlign
    |> withWrap wrapWord

// ============================================================================
// Main Dashboard Assembly
// ============================================================================

// This would be rendered using the TUI runtime with proper layout
let tolariaDashboard =
    let mainBlock = createTitleBlock in

    // Widgets ready for layout composition:
    // - Top: Title block, cluster overview, navigation tabs
    // - Left column: Node table, namespace list, deployment list
    // - Middle column: Pod list, event log
    // - Right column: Resource gauges (CPU, memory, storage, network)
    // - Bottom row: Sparklines (CPU/memory trends), service health
    // - Footer: Help section

    mainBlock

// Export main widgets
let clusterOverview = createClusterOverview
let nodeTable = createNodeTable
let podList = createPodList
let clusterCpuGauge = createClusterCpuGauge
let clusterMemoryGauge = createClusterMemoryGauge
let storageGauge = createStorageGauge
let eventLog = createEventLog
let namespaceList = createNamespaceList
let cpuSparkline = createCpuSparkline
let memorySparkline = createMemorySparkline
let deploymentList = createDeploymentList
let serviceHealth = createServiceHealth
let navigationTabs = createNavigationTabs
let networkGauge = createNetworkGauge
let restartsTable = createRestartsTable
let helpSection = createHelpSection

// Dashboard metadata
let dashboardTitle = "Tolaria Kubernetes Cluster Monitor"
let dashboardVersion = "v2.3.0"
let dashboardDescription = "Real-time Kubernetes cluster monitoring and management"
