// Async Demo Debug 5
let log msg =
    print msg
    print "\n"

let main = async {
    log "Inside async"
    return "Done"
}

log "Calling RunSync"
let _ = Async.RunSynchronously main
log "Done"