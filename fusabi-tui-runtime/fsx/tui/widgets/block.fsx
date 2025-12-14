#load "../style.fsx"

// TUI Block Widget
// Represents a bordered container for other widgets

// Border type variations
type BorderType =
    | Plain
    | Rounded
    | Double
    | Thick

// Border flags for which sides to draw
type Borders =
    | None
    | Top
    | Right
    | Bottom
    | Left
    | TopRight
    | TopBottom
    | TopLeft
    | RightBottom
    | RightLeft
    | BottomLeft
    | TopRightBottom
    | TopRightLeft
    | TopBottomLeft
    | RightBottomLeft
    | All

// Option type for strings (for optional title)
type OptionString =
    | NoneString
    | SomeString of string

// Padding represented as left * right * top * bottom
type Padding =
    | Padding of int * int * int * int

// Block widget
// Represented as title * borders * borderType * style * padding
type Block =
    | Block of OptionString * Borders * BorderType * Style * Padding

// Block constructors and utilities
let emptyBlock = Block (NoneString, None, Plain, emptyStyle, Padding (0, 0, 0, 0))

let createBlock = Block (NoneString, All, Plain, emptyStyle, Padding (0, 0, 0, 0))

let withTitle title block =
    match block with
    | Block (_, borders, borderType, style, padding) ->
        Block (SomeString title, borders, borderType, style, padding)

let withBorders borders block =
    match block with
    | Block (title, _, borderType, style, padding) ->
        Block (title, borders, borderType, style, padding)

let withBorderType borderType block =
    match block with
    | Block (title, borders, _, style, padding) ->
        Block (title, borders, borderType, style, padding)

let withBlockStyle style block =
    match block with
    | Block (title, borders, borderType, _, padding) ->
        Block (title, borders, borderType, style, padding)

let withPadding left right top bottom block =
    match block with
    | Block (title, borders, borderType, style, _) ->
        Block (title, borders, borderType, style, Padding (left, right, top, bottom))

// Border type constructors
let plainBorder = Plain
let roundedBorder = Rounded
let doubleBorder = Double
let thickBorder = Thick

// Border flag helpers
let noBorders = None
let allBorders = All
let topBorder = Top
let rightBorder = Right
let bottomBorder = Bottom
let leftBorder = Left
