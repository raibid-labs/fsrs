// Commander - A TUI File Explorer for Fusabi
// Demonstrates Events, TerminalControl, TerminalInfo, Process, List, and String modules

// ============================================================================
// MODEL
// ============================================================================

let initialModel = {
    currentDir = Process.cwd ();
    files = [];
    selectedIndex = 0;
    running = true
}

// ============================================================================
// HELPERS
// ============================================================================

let parseFiles stdout =
    let lines = String.split "\n" stdout
    let nonEmpty = List.filter (fun line -> String.length line > 0) lines
    // Skip the first line (total count from ls -l)
    match List.tail nonEmpty with
    | Some tail -> tail
    | None -> []

let getFileName line =
    let parts = String.split " " line
    let filtered = List.filter (fun p -> String.length p > 0) parts
    // Get the last part (filename)
    match List.nth 8 filtered with
    | Some name -> name
    | None -> line

let loadDirectory dir =
    let result = Process.runShell (sprintf "ls -la \"%s\"" dir)
    if result.exitCode == 0 then
        parseFiles result.stdout
    else
        []

let clamp min max value =
    if value < min then min
    else if value > max then max
    else value

// ============================================================================
// VIEW
// ============================================================================

let clearScreen () =
    TerminalControl.sendText "\x1b[2J\x1b[H"

let renderHeader model =
    printfn (sprintf "=== Commander TUI ===")
    printfn (sprintf "Directory: %s" model.currentDir)
    printfn (sprintf "")

let renderFile index selected fileName =
    let marker = if index == selected then "> " else "  "
    printfn (sprintf "%s%s" marker fileName)

let renderFiles model =
    let renderWithIndex = fun (index, fileName) ->
        renderFile index model.selectedIndex fileName
    let indexed = List.mapi (fun i f -> (i, f)) model.files
    List.iter renderWithIndex indexed

let renderFooter () =
    printfn (sprintf "")
    printfn (sprintf "Controls: j/k (navigate) | Enter (select) | q (quit)")

let render model =
    clearScreen ()
    renderHeader model
    renderFiles model
    renderFooter ()

// ============================================================================
// UPDATE
// ============================================================================

let handleKeyDown model =
    let maxIndex = List.length model.files - 1
    let newIndex = clamp 0 maxIndex (model.selectedIndex + 1)
    { model with selectedIndex = newIndex }

let handleKeyUp model =
    let maxIndex = List.length model.files - 1
    let newIndex = clamp 0 maxIndex (model.selectedIndex - 1)
    { model with selectedIndex = newIndex }

let handleEnter model =
    match List.nth model.selectedIndex model.files with
    | Some selectedFile ->
        let fileName = getFileName selectedFile
        let newPath =
            if fileName == "." then
                model.currentDir
            else if fileName == ".." then
                let result = Process.runShell (sprintf "cd \"%s\" && cd .. && pwd" model.currentDir)
                if result.exitCode == 0 then
                    String.trim result.stdout
                else
                    model.currentDir
            else
                sprintf "%s/%s" model.currentDir fileName

        // Check if it's a directory
        let checkResult = Process.runShell (sprintf "test -d \"%s\" && echo \"dir\"" newPath)
        if String.contains "dir" checkResult.stdout then
            let files = loadDirectory newPath
            { model with currentDir = newPath; files = files; selectedIndex = 0 }
        else
            TerminalControl.showToast (sprintf "Not a directory: %s" fileName)
            model
    | None -> model

let handleQuit model =
    { model with running = false }

let update event model =
    match event with
    | "key:j" -> handleKeyDown model
    | "key:k" -> handleKeyUp model
    | "key:enter" -> handleEnter model
    | "key:q" -> handleQuit model
    | _ -> model

// ============================================================================
// EVENT LOOP
// ============================================================================

let rec eventLoop model =
    if model.running then
        // Render current state
        render model

        // Simulate getting input (in a real TUI, this would be async)
        // For this demo, we'll show the concept
        printfn (sprintf "\nEnter command (j/k/enter/q): ")

        // In a real implementation, we'd have:
        // let input = Console.ReadLine()
        // let eventName = sprintf "key:%s" input
        // let newModel = update eventName model
        // eventLoop newModel

        // For demo purposes, show how it would work
        let demoCommands = ["key:j"; "key:j"; "key:k"; "key:enter"; "key:q"]
        let newModel = List.fold (fun m event -> update event m) model demoCommands

        TerminalControl.showToast "Demo complete - in real usage, this would be interactive"
        newModel
    else
        printfn (sprintf "\nExiting Commander...")
        model

// ============================================================================
// MAIN
// ============================================================================

let main () =
    printfn (sprintf "Starting Commander TUI...")
    printfn (sprintf "")

    // Get terminal size
    let (cols, rows) = TerminalInfo.getTerminalSize ()
    printfn (sprintf "Terminal size: %d cols x %d rows" cols rows)

    // Get current directory
    let cwd = Process.cwd ()
    printfn (sprintf "Starting directory: %s" cwd)
    printfn (sprintf "")

    // Load initial files
    let files = loadDirectory cwd
    let startModel = { initialModel with currentDir = cwd; files = files }

    // Register event handlers
    let handlerJ = Events.on "key:j" (fun _ -> printfn (sprintf "Down pressed"))
    let handlerK = Events.on "key:k" (fun _ -> printfn (sprintf "Up pressed"))
    let handlerEnter = Events.on "key:enter" (fun _ -> printfn (sprintf "Enter pressed"))
    let handlerQ = Events.on "key:q" (fun _ -> printfn (sprintf "Quit pressed"))

    printfn (sprintf "Event handlers registered: %d, %d, %d, %d" handlerJ handlerK handlerEnter handlerQ)
    printfn (sprintf "")

    // Start event loop
    let finalModel = eventLoop startModel

    // Cleanup event handlers
    let _ = Events.off handlerJ
    let _ = Events.off handlerK
    let _ = Events.off handlerEnter
    let _ = Events.off handlerQ

    printfn (sprintf "Commander TUI exited")
    printfn (sprintf "Final directory: %s" finalModel.currentDir)

// Run the application
main ()
