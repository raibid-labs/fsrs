#load "text.fsx"
#load "block.fsx"

// TUI List Widget
// Displays a selectable list of items

// ListItem - a single item in the list
// Represented as text * style
type ListItem =
    | ListItem of Text * Style

// Option type for selected index
type OptionInt =
    | NoneInt
    | SomeInt of int

// ListState - tracks selection state
// Represented as selected index
type ListState =
    | ListState of OptionInt

// List widget
// Note: Since Fusabi doesn't support lists in DUs, we use a simplified representation
// List represented as item * block * style * highlightStyle
// (single item instead of list of items)
type List =
    | List of ListItem * OptionBlock * Style * Style

// ListItem constructors
let createListItem text style = ListItem (text, style)

let listItemFromString str = ListItem (textFromString str, emptyStyle)

let styledListItem str style = ListItem (textFromString str, style)

// ListState constructors
let emptyListState = ListState NoneInt

let createListState = ListState NoneInt

let selectListItem index state =
    ListState (SomeInt index)

let deselectListItem state = ListState NoneInt

let nextListItem state =
    match state with
    | ListState (SomeInt idx) -> ListState (SomeInt (idx + 1))
    | ListState NoneInt -> ListState (SomeInt 0)

let prevListItem state =
    match state with
    | ListState (SomeInt idx) ->
        let newIdx = if idx > 0 then idx - 1 else 0 in
        ListState (SomeInt newIdx)
    | ListState NoneInt -> ListState (SomeInt 0)

// List constructors
let emptyList = List (listItemFromString "", NoneBlock, emptyStyle, emptyStyle)

let createList item = List (item, NoneBlock, emptyStyle, emptyStyle)

let listFromItems item = List (item, NoneBlock, emptyStyle, emptyStyle)

let withListBlock block list =
    match list with
    | List (item, _, style, highlightStyle) ->
        List (item, SomeBlock block, style, highlightStyle)

let withListStyle style list =
    match list with
    | List (item, block, _, highlightStyle) ->
        List (item, block, style, highlightStyle)

let withHighlightStyle style list =
    match list with
    | List (item, block, listStyle, _) ->
        List (item, block, listStyle, style)
