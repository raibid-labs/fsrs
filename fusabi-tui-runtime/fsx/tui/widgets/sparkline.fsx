#load "block.fsx"

// TUI Sparkline Widget
// Displays a compact line chart for visualizing data trends

// Direction for sparkline rendering
type SparklineDirection =
    | LeftToRight
    | RightToLeft

// Option type for max value
type OptionMax =
    | NoneMax
    | SomeMax of int

// Sparkline widget
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// Sparkline represented as data value * max * block * style * direction
type Sparkline =
    | Sparkline of int * OptionMax * OptionBlock * Style * SparklineDirection

// Sparkline constructors and utilities
let emptySparkline = Sparkline (0, NoneMax, NoneBlock, emptyStyle, LeftToRight)

let createSparkline data = Sparkline (data, NoneMax, NoneBlock, emptyStyle, LeftToRight)

let sparklineFromData data = Sparkline (data, NoneMax, NoneBlock, emptyStyle, LeftToRight)

let withSparklineData data sparkline =
    match sparkline with
    | Sparkline (_, max, block, style, direction) ->
        Sparkline (data, max, block, style, direction)

let withMax max sparkline =
    match sparkline with
    | Sparkline (data, _, block, style, direction) ->
        Sparkline (data, SomeMax max, block, style, direction)

let withSparklineBlock block sparkline =
    match sparkline with
    | Sparkline (data, max, _, style, direction) ->
        Sparkline (data, max, SomeBlock block, style, direction)

let withSparklineStyle style sparkline =
    match sparkline with
    | Sparkline (data, max, block, _, direction) ->
        Sparkline (data, max, block, style, direction)

let withDirection direction sparkline =
    match sparkline with
    | Sparkline (data, max, block, style, _) ->
        Sparkline (data, max, block, style, direction)

// Direction constructors
let leftToRight = LeftToRight
let rightToLeft = RightToLeft
