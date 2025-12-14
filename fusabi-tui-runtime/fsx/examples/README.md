# Fusabi TUI Runtime - FSX Examples

This directory contains comprehensive examples demonstrating the Fusabi TUI runtime and widget library. The examples showcase real-world dashboard implementations for the Raibid Labs ecosystem.

## Overview

These examples demonstrate how to build professional TUI applications using Fusabi's F# dialect (FSX) and the TUI widget bindings. Each example is a complete, self-contained dashboard that can be loaded and rendered using the Fusabi TUI runtime.

## Available Examples

### Basic Examples

#### `hello_tui.fsx`
A minimal "Hello World" example demonstrating basic TUI setup.
- **Purpose**: Introduction to TUI concepts
- **Widgets**: Text, basic block
- **Complexity**: Beginner

#### `dashboard.fsx`
A general-purpose dashboard showing all available widgets.
- **Purpose**: Widget showcase and API reference
- **Widgets**: All core widgets (blocks, lists, gauges, sparklines, tables, tabs, paragraphs)
- **Complexity**: Intermediate

---

### Raibid Labs Ecosystem Dashboards

These examples demonstrate real-world TUI dashboards for the Raibid Labs project suite:

#### `sigilforge_tui.fsx` - OAuth Token Management Dashboard
**Sigilforge** is Raibid Labs' OAuth integration service managing authentication tokens for multiple providers.

**Features:**
- Connected OAuth providers list with status indicators
- Token expiry monitoring with color-coded warnings (green=valid, yellow=expiring, red=expired)
- Token health gauges showing remaining validity
- Active login flow status display
- Recent activity log
- Token refresh sparkline showing refresh patterns

**Widgets Used:** List, Gauge, Sparkline, Paragraph, Tabs, Table

**Key Concepts:**
- Status color coding for quick visual feedback
- Real-time token health monitoring
- Multi-provider management UI

---

#### `scryforge_tui.fsx` - YouTube Analytics Dashboard
**Scryforge** is Raibid Labs' YouTube analytics and management service providing channel insights.

**Features:**
- Channel overview with subscriber count, total views, and video count
- Video performance list showing views and engagement
- Subscriber growth sparkline (30-day trend)
- Watch time goal gauge with progress tracking
- Engagement rate gauge
- Recent comments list
- Upload schedule calendar view
- Top videos table
- Revenue tracking (for monetized channels)
- Traffic source breakdown
- Average view duration sparkline

**Widgets Used:** List, Gauge, Sparkline, Paragraph, Tabs, Table

**Key Concepts:**
- Multi-metric dashboard layout
- Time-series data visualization with sparklines
- Hierarchical information display
- Content calendar integration

---

#### `phage_tui.fsx` - Context Visualization Dashboard
**Phage** is Raibid Labs' intelligent context management system maintaining semantic memory across conversations.

**Features:**
- Memory usage gauge showing allocation status
- Active contexts list with type indicators (Conversation, Project, Topic, Session)
- Context allocation gauge showing active vs. total capacity
- Topic event stream with real-time updates
- Context tree visualization showing hierarchical structure
- Memory allocation sparkline (24-hour trend)
- Context persistence status
- Cache hit rate gauge
- Context activity sparkline (7-day trend)
- Topic statistics table

**Widgets Used:** List, Gauge, Sparkline, Paragraph, Tabs, Table

**Key Concepts:**
- Memory management visualization
- Hierarchical context tree representation
- Real-time event streaming
- Performance metrics (cache hit rate)

---

#### `scarab_status.fsx` - Terminal Status Bar
**Scarab** is Raibid Labs' high-performance split-process terminal emulator built in Rust with Bevy rendering.

**Features:**
- Current working directory display
- Git branch indicator with status symbols (✓=clean, ●=modified, +=staged, ✗=conflict)
- CPU and memory mini gauges
- Current time display
- Active terminal tabs count
- Process count indicator
- Session name display
- Shell type indicator (bash, zsh, etc.)
- Network activity display (upload/download)
- Battery gauge (for laptops)
- PTY process list (expanded view)
- Tab list with bell notifications
- Git status details table

**Widgets Used:** Gauge, Paragraph, List, Table

**Key Concepts:**
- Compact status bar design
- Real-time system metrics
- Git integration
- Expandable detailed views

---

#### `hibana_metrics.fsx` - GPU Metrics Dashboard
**Hibana** is Raibid Labs' GPU compute orchestration system for distributed training and inference.

**Features:**
- Multi-GPU utilization gauges (supports 4+ GPUs)
- VRAM memory usage per GPU
- GPU temperature sparklines with historical trends
- Total power consumption gauge
- Active GPU process list with memory usage
- Compute performance table (TFLOPS)
- GPU health status monitoring
- PCIe bandwidth usage sparkline
- Fan speed gauge
- GPU clock speed display
- Process type indicators (Training, Inference, Compute, Graphics)

**Widgets Used:** Gauge, Sparkline, List, Table, Paragraph, Tabs

**Key Concepts:**
- Multi-device monitoring (4 GPUs)
- Real-time performance metrics
- Temperature trend analysis
- Process-to-GPU assignment visualization

---

#### `tolaria_cluster.fsx` - Kubernetes Cluster Monitor
**Tolaria** is Raibid Labs' Kubernetes cluster orchestration and monitoring platform.

**Features:**
- Cluster overview (nodes, pods, namespaces)
- Node status table with resource allocation
- Pod list with status colors and resource usage
- Cluster-wide CPU usage gauge
- Cluster-wide memory usage gauge
- Persistent volume storage gauge
- Event log stream with severity indicators
- Namespace resource breakdown
- CPU utilization sparkline (24-hour trend)
- Memory utilization sparkline (24-hour trend)
- Deployment status list with rollout tracking
- Service endpoint health checks
- Network I/O gauge
- Pod restarts table for troubleshooting

**Widgets Used:** Table, List, Gauge, Sparkline, Paragraph, Tabs

**Key Concepts:**
- Distributed system monitoring
- Multi-namespace resource tracking
- Event-driven status updates
- Health check visualization
- Rolling deployment monitoring

---

## Widget Reference

All examples use widgets from the `/fsx/tui/widgets/` directory:

| Widget | Purpose | Used In |
|--------|---------|---------|
| **Block** | Bordered container with title | All examples |
| **List** | Selectable item list | Providers, videos, contexts, processes, pods |
| **Gauge** | Progress/percentage indicator | CPU, memory, token health, watch time |
| **Sparkline** | Compact trend visualization | Subscribers, temperature, memory, CPU trends |
| **Table** | Tabular data display | Statistics, nodes, git status |
| **Paragraph** | Multi-line text with wrapping | Logs, help text, status messages |
| **Tabs** | Tabbed navigation | Dashboard sections |

## Common Patterns

### Color Coding for Status

All dashboards use consistent color schemes:
- **Green**: Healthy, active, connected
- **Yellow**: Warning, expiring, pending
- **Red**: Error, expired, failed
- **Cyan**: Information, selected
- **Magenta**: Special state, processing
- **Blue**: Archived, inactive, idle
- **White**: Neutral, default

### Layout Composition

Typical dashboard layout:
```
┌─────────────────────────────────────────┐
│         Title Block (Borders: All)      │
├─────────────────────────────────────────┤
│  Tabs (Navigation)                      │
├──────────────┬──────────────┬───────────┤
│              │              │           │
│  Left Column │ Middle Column│ Right Col │
│  (Lists)     │ (Gauges)     │ (Metrics) │
│              │              │           │
├──────────────┴──────────────┴───────────┤
│  Bottom Row (Sparklines, Help)          │
└─────────────────────────────────────────┘
```

### Data Models

Each example includes:
1. Type definitions for domain models (e.g., `ProviderStatus`, `PodStatus`)
2. Helper functions for formatting and color selection
3. Widget constructors using the builder pattern
4. Exported widgets ready for layout composition

## Running the Examples

To load an example in the Fusabi TUI runtime:

```bash
# Using the Fusabi interpreter
fusabi-fsx examples/sigilforge_tui.fsx

# Or compile to bytecode first
fusabi-compile examples/sigilforge_tui.fsx -o sigilforge.fzb
fusabi-run sigilforge.fzb
```

## Integration with Rust TUI Runtime

These FSX files are designed to be loaded by the Rust TUI runtime (`fusabi-tui-runtime`):

```rust
use fusabi_tui_runtime::FusabiTuiRuntime;

let runtime = FusabiTuiRuntime::new();
runtime.load_script("examples/sigilforge_tui.fsx")?;
runtime.render()?;
```

The runtime provides:
- FSX script loading and execution
- Layout management
- Event handling
- Terminal rendering
- Hot-reloading support

## Development Guidelines

When creating new examples:

1. **Start with `#load "../tui.fsx"`** to import all TUI modules
2. **Define data models** using discriminated unions for clarity
3. **Use helper functions** for color selection and formatting
4. **Create widget builders** with descriptive names (e.g., `createProviderList`)
5. **Export main widgets** at the bottom for external use
6. **Include metadata** (title, version, description)
7. **Add comments** explaining the dashboard purpose and features
8. **Follow naming conventions**: `create*` for widget builders, `get*` for helpers

## FSX Language Features Used

These examples demonstrate various Fusabi F# features:

- **Discriminated Unions**: Type-safe status enums
- **Pattern Matching**: Status-to-color mapping
- **Function Composition**: Builder pattern with `|>` operator
- **Let Bindings**: Widget and style definitions
- **String Interpolation**: Dynamic text generation (where supported)
- **Module Loading**: `#load` directives

## Contributing

To add a new example:

1. Create `your_example.fsx` in this directory
2. Follow the structure of existing examples
3. Document all features in this README
4. Test with the Fusabi TUI runtime
5. Ensure all widgets render correctly

## License

These examples are part of the Fusabi TUI Runtime project and follow the same license.

## Related Documentation

- **Fusabi Language**: https://github.com/fusabi-lang/fusabi
- **TUI Widget API**: `/fsx/tui/widgets/`
- **Core TUI Library**: `/fsx/tui.fsx`
- **Raibid Labs Ecosystem**: Internal documentation

## Version History

- **v0.1.0** (2025-12-14): Initial examples
  - Basic examples: hello_tui.fsx, dashboard.fsx
  - Ecosystem dashboards: sigilforge_tui, scryforge_tui, phage_tui, scarab_status, hibana_metrics, tolaria_cluster

---

**Note**: These are demonstration dashboards with simulated data. In production, they would connect to actual backend services via the Fusabi runtime's FFI capabilities.
