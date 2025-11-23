module Layouts =
  type Layout =
    | Pane of cmd: string * width: int option
    | Row of Layout list
    | Column of Layout list

  let default =
    layout {
      row {
        pane { cmd "htop"; width 30 }
        column {
          pane { cmd "cargo watch -x test" }
          pane { cmd "cargo watch -x run" }
        }
      }
    }

module Keys =
  type Direction = Left | Right | Up | Down

  type Action =
    | Split of Direction
    | MoveFocus of Direction
    | SendKeys of string
    | RenameTab of string

  type KeyBinding =
    { Key: string
      Action: Action }

  let bindings =
    keys {
      bind "Ctrl-Shift-H" (MoveFocus Left)
      bind "Ctrl-Shift-L" (MoveFocus Right)
      bind "Ctrl-Shift-Enter" (Split Down)
    }

module Config =
  open Layouts
  open Keys

  type Config =
    { Layout : Layout
      KeyBindings : KeyBinding list }

  let config : Config =
    { Layout = Layouts.default
      KeyBindings = Keys.bindings }
