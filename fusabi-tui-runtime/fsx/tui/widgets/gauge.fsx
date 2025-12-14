#load "block.fsx"

// TUI Gauge Widget
// Displays a progress bar or percentage indicator

// Option type for label
type OptionLabel =
    | NoneLabel
    | SomeLabel of string

// Gauge widget
// Represented as ratio * label * block * style * gaugeStyle
// ratio is 0-100 representing percentage
type Gauge =
    | Gauge of int * OptionLabel * OptionBlock * Style * Style

// Gauge constructors and utilities
let emptyGauge = Gauge (0, NoneLabel, NoneBlock, emptyStyle, emptyStyle)

let createGauge ratio = Gauge (ratio, NoneLabel, NoneBlock, emptyStyle, emptyStyle)

let gaugeFromPercent percent =
    let clampedPercent = if percent > 100 then 100 else if percent < 0 then 0 else percent in
    Gauge (clampedPercent, NoneLabel, NoneBlock, emptyStyle, emptyStyle)

let withRatio ratio gauge =
    match gauge with
    | Gauge (_, label, block, style, gaugeStyle) ->
        let clampedRatio = if ratio > 100 then 100 else if ratio < 0 then 0 else ratio in
        Gauge (clampedRatio, label, block, style, gaugeStyle)

let withLabel label gauge =
    match gauge with
    | Gauge (ratio, _, block, style, gaugeStyle) ->
        Gauge (ratio, SomeLabel label, block, style, gaugeStyle)

let withGaugeBlock block gauge =
    match gauge with
    | Gauge (ratio, label, _, style, gaugeStyle) ->
        Gauge (ratio, label, SomeBlock block, style, gaugeStyle)

let withGaugeStyle style gauge =
    match gauge with
    | Gauge (ratio, label, block, _, gaugeStyle) ->
        Gauge (ratio, label, block, style, gaugeStyle)

let withGaugeBarStyle style gauge =
    match gauge with
    | Gauge (ratio, label, block, gaugeStyle, _) ->
        Gauge (ratio, label, block, gaugeStyle, style)

// Helper functions
let percentToRatio percent =
    if percent > 100 then 100 else if percent < 0 then 0 else percent

let ratioToPercent ratio = ratio
