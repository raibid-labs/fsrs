#load "text.fsx"
#load "block.fsx"

// TUI Paragraph Widget
// Displays multi-line text with optional wrapping and alignment

// Wrap mode for text
type Wrap =
    | NoWrap
    | WrapChar
    | WrapWord

// Text alignment
type Alignment =
    | Left
    | Center
    | Right

// Option type for Block (for optional block)
type OptionBlock =
    | NoneBlock
    | SomeBlock of Block

// Paragraph widget
// Represented as text * block * style * alignment * wrap
type Paragraph =
    | Paragraph of Text * OptionBlock * Style * Alignment * Wrap

// Paragraph constructors and utilities
let emptyParagraph = Paragraph (TextEmpty, NoneBlock, emptyStyle, Left, NoWrap)

let createParagraph text = Paragraph (text, NoneBlock, emptyStyle, Left, NoWrap)

let paragraphFromString str =
    let text = textFromString str in
    Paragraph (text, NoneBlock, emptyStyle, Left, NoWrap)

let withParagraphText text paragraph =
    match paragraph with
    | Paragraph (_, block, style, alignment, wrap) ->
        Paragraph (text, block, style, alignment, wrap)

let withParagraphBlock block paragraph =
    match paragraph with
    | Paragraph (text, _, style, alignment, wrap) ->
        Paragraph (text, SomeBlock block, style, alignment, wrap)

let withParagraphStyle style paragraph =
    match paragraph with
    | Paragraph (text, block, _, alignment, wrap) ->
        Paragraph (text, block, style, alignment, wrap)

let withAlignment alignment paragraph =
    match paragraph with
    | Paragraph (text, block, style, _, wrap) ->
        Paragraph (text, block, style, alignment, wrap)

let withWrap wrap paragraph =
    match paragraph with
    | Paragraph (text, block, style, alignment, _) ->
        Paragraph (text, block, style, alignment, wrap)

// Wrap mode constructors
let noWrap = NoWrap
let wrapChar = WrapChar
let wrapWord = WrapWord

// Alignment constructors
let leftAlign = Left
let centerAlign = Center
let rightAlign = Right
