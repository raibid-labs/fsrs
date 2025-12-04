// Async Demo
// Demonstrates the new Async Computation Expressions

// Helper to print progress
let log msg =
    let t = Time.format "%H:%M:%S" (Time.now()) in
    printfn (sprintf "[%s] %s" [t; msg])

// Simulating an async operation
let sleep ms = async {
    log (sprintf "Sleeping for %d ms..." [ms])
    // In a real implementation this would be non-blocking
    // For now, we just spin/wait but structure it as async
    let _ = Process.runShell (sprintf "sleep %f" [float ms / 1000.0])
    log "Woke up!"
    return ms
}

// Simulating data fetching
let fetchData url = async {
    log (sprintf "Fetching %s..." [url])
    do! sleep 500
    return (sprintf "Content of %s" [url])
}

// The main async workflow
let main = async {
    log "Starting workflow..."

    // Sequential composition using let!
    let! data1 = fetchData "http://example.com/1"
    log (sprintf "Got: %s" [data1])

    let! data2 = fetchData "http://example.com/2"
    log (sprintf "Got: %s" [data2])

    // Control flow (Linear for now as parser doesn't support nested do! in if)
    log "Doing extra work..."
    do! sleep 200

    return "Done!"
}

// Execute the workflow
log "--- RunSynchronously ---"
let result = Async.RunSynchronously main
log (sprintf "Final Result: %s" [result])
