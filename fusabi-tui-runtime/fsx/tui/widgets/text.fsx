#load "../style.fsx"

// TUI Text Primitives
// Represents styled text content: Span, Line, and Text

// Span - a styled text segment
// Represented as content * style
type Span =
    | Span of string * Style

// Line - a sequence of spans
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// Line represented as a single span (for simplicity)
type Line =
    | Line of Span
    | LineEmpty

// Text - a sequence of lines
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// Text represented as a single line (for simplicity)
type Text =
    | Text of Line
    | TextEmpty

// Span constructors and utilities
let createSpan content style = Span (content, style)

let rawSpan content = Span (content, emptyStyle)

let styledSpan content fg bg =
    let style = emptyStyle |> withFg fg |> withBg bg in
    Span (content, style)

// Line constructors
let emptyLine = LineEmpty

let lineFromSpan span = Line span

let lineFromText text = Line (Span (text, emptyStyle))

let styledLine text style = Line (Span (text, style))

// Text constructors
let emptyText = TextEmpty

let textFromLine line = Text line

let textFromString str = Text (Line (Span (str, emptyStyle)))

let styledText str style = Text (Line (Span (str, style)))

// Helper for creating multi-colored text
let textFromSpans span = Text (Line span)
