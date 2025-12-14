#load "block.fsx"

// TUI Tabs Widget
// Displays a tabbed interface with selectable tabs

// Tabs widget
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// Tabs represented as title * selected * divider * block * style * highlightStyle
type Tabs =
    | Tabs of string * int * string * OptionBlock * Style * Style

// Tabs constructors and utilities
let emptyTabs = Tabs ("", 0, " ", NoneBlock, emptyStyle, emptyStyle)

let createTabs title = Tabs (title, 0, " ", NoneBlock, emptyStyle, emptyStyle)

let tabsFromTitles title = Tabs (title, 0, " ", NoneBlock, emptyStyle, emptyStyle)

let withTabTitles title tabs =
    match tabs with
    | Tabs (_, selected, divider, block, style, highlightStyle) ->
        Tabs (title, selected, divider, block, style, highlightStyle)

let withSelected index tabs =
    match tabs with
    | Tabs (title, _, divider, block, style, highlightStyle) ->
        Tabs (title, index, divider, block, style, highlightStyle)

let withDivider divider tabs =
    match tabs with
    | Tabs (title, selected, _, block, style, highlightStyle) ->
        Tabs (title, selected, divider, block, style, highlightStyle)

let withTabsBlock block tabs =
    match tabs with
    | Tabs (title, selected, divider, _, style, highlightStyle) ->
        Tabs (title, selected, divider, SomeBlock block, style, highlightStyle)

let withTabsStyle style tabs =
    match tabs with
    | Tabs (title, selected, divider, block, _, highlightStyle) ->
        Tabs (title, selected, divider, block, style, highlightStyle)

let withTabsHighlightStyle style tabs =
    match tabs with
    | Tabs (title, selected, divider, block, tabsStyle, _) ->
        Tabs (title, selected, divider, block, tabsStyle, style)

// Tab navigation
let nextTab tabs =
    match tabs with
    | Tabs (title, selected, divider, block, style, highlightStyle) ->
        Tabs (title, selected + 1, divider, block, style, highlightStyle)

let prevTab tabs =
    match tabs with
    | Tabs (title, selected, divider, block, style, highlightStyle) ->
        let newSelected = if selected > 0 then selected - 1 else 0 in
        Tabs (title, newSelected, divider, block, style, highlightStyle)

// Common dividers
let pipeDivider = "|"
let slashDivider = "/"
let spaceDivider = " "
let dotDivider = "·"
