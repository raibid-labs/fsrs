#load "text.fsx"
#load "block.fsx"

// TUI Table Widget
// Displays data in a tabular format with rows and columns

// TableCell - a single cell in the table
// Represented as text * style
type TableCell =
    | TableCell of Text * Style

// TableRow - a row in the table
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// TableRow represented as a single cell
type TableRow =
    | TableRow of TableCell * Style

// Constraint for column widths
type ColumnConstraint =
    | ColumnPercentage of int
    | ColumnLength of int
    | ColumnMin of int
    | ColumnMax of int
    | ColumnFill of int

// Option type for header
type OptionRow =
    | NoneRow
    | SomeRow of TableRow

// TableState - tracks selection state
// Represented as selected row index
type TableState =
    | TableState of OptionInt

// Table widget
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// Table represented as row * header * block * style * widthConstraint
type Table =
    | Table of TableRow * OptionRow * OptionBlock * Style * ColumnConstraint

// TableCell constructors
let createTableCell text style = TableCell (text, style)

let tableCellFromString str = TableCell (textFromString str, emptyStyle)

let styledTableCell str style = TableCell (textFromString str, style)

// TableRow constructors
let createTableRow cell style = TableRow (cell, style)

let tableRowFromCell cell = TableRow (cell, emptyStyle)

let tableRowFromStrings str = TableRow (tableCellFromString str, emptyStyle)

// TableState constructors
let emptyTableState = TableState NoneInt

let createTableState = TableState NoneInt

let selectTableRow index state = TableState (SomeInt index)

let deselectTableRow state = TableState NoneInt

let nextTableRow state =
    match state with
    | TableState (SomeInt idx) -> TableState (SomeInt (idx + 1))
    | TableState NoneInt -> TableState (SomeInt 0)

let prevTableRow state =
    match state with
    | TableState (SomeInt idx) ->
        let newIdx = if idx > 0 then idx - 1 else 0 in
        TableState (SomeInt newIdx)
    | TableState NoneInt -> TableState (SomeInt 0)

// Table constructors
let emptyTable = Table (tableRowFromStrings "", NoneRow, NoneBlock, emptyStyle, ColumnFill 1)

let createTable row = Table (row, NoneRow, NoneBlock, emptyStyle, ColumnFill 1)

let withTableHeader header table =
    match table with
    | Table (row, _, block, style, widths) ->
        Table (row, SomeRow header, block, style, widths)

let withTableBlock block table =
    match table with
    | Table (row, header, _, style, widths) ->
        Table (row, header, SomeBlock block, style, widths)

let withTableStyle style table =
    match table with
    | Table (row, header, block, _, widths) ->
        Table (row, header, block, style, widths)

let withColumnWidths constraint table =
    match table with
    | Table (row, header, block, style, _) ->
        Table (row, header, block, style, constraint)

// Column constraint constructors
let columnPercentage pct = ColumnPercentage pct
let columnLength len = ColumnLength len
let columnMin min = ColumnMin min
let columnMax max = ColumnMax max
let columnFill weight = ColumnFill weight
