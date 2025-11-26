// Ardour DAW OSC Control Demo
// Demonstrates the Net.Osc module for controlling Ardour via OSC
// Part of RFD-001: Unified MCP DSL

open Osc

printfn "=== Ardour OSC Control Demo ==="

// Create OSC client connected to Ardour
// Default Ardour OSC port is 3819
let ardour = Osc.client "localhost" 3819

printfn "✅ Connected to Ardour on localhost:3819"

// ===== Transport Controls =====

printfn "\n=== Transport Control ==="

// Start playback
printfn "▶️  Starting playback..."
ardour |> Osc.send "/transport_play"

// Wait a moment (in real code)
// System.Threading.Thread.Sleep(2000)

// Stop playback
printfn "⏹️  Stopping playback..."
ardour |> Osc.send "/transport_stop"

// Start recording
printfn "⏺️  Starting record..."
ardour |> Osc.send "/transport_record"

// Toggle loop mode
printfn "🔁 Toggling loop mode..."
ardour |> Osc.send "/loop_toggle"

// ===== Session Control =====

printfn "\n=== Session Control ==="

// Set session tempo
printfn "🎵 Setting tempo to 120 BPM..."
ardour |> Osc.sendInt "/set_tempo" 120

// Set master volume (0.0 to 1.0)
printfn "🔊 Setting master volume to 0.75..."
ardour |> Osc.sendFloat "/master/fader" 0.75

// Set session name
printfn "📝 Setting session name..."
ardour |> Osc.sendString "/set_session_name" "My Recording Session"

// ===== Track Control =====

printfn "\n=== Track Control ==="

// Select track 1
printfn "🎚️  Selecting track 1..."
ardour |> Osc.sendInt "/select/track" 1

// Set track volume
printfn "🔊 Setting track 1 volume..."
ardour |> Osc.sendFloat "/strip/fader" 0.8

// Pan track (0.0 = left, 0.5 = center, 1.0 = right)
printfn "↔️  Panning track 1 to center..."
ardour |> Osc.sendFloat "/strip/pan_stereo_position" 0.5

// Mute track
printfn "🔇 Muting track 1..."
ardour |> Osc.sendInt "/strip/mute" 1

// Unmute track
printfn "🔊 Unmuting track 1..."
ardour |> Osc.sendInt "/strip/mute" 0

// Solo track
printfn "🎧 Soloing track 1..."
ardour |> Osc.sendInt "/strip/solo" 1

// ===== Automation Example =====

printfn "\n=== Automation Example: Fade In ==="

// Simulate a fade-in by gradually increasing volume
let fadeIn client steps duration =
    let stepSize = 1.0 / (float steps)
    printfn "🎚️  Fading in over %d steps..." steps

    // In a real implementation, you'd loop here
    // For conceptual purposes:
    client |> Osc.sendFloat "/strip/fader" 0.0
    client |> Osc.sendFloat "/strip/fader" 0.25
    client |> Osc.sendFloat "/strip/fader" 0.5
    client |> Osc.sendFloat "/strip/fader" 0.75
    client |> Osc.sendFloat "/strip/fader" 1.0
    printfn "✅ Fade in complete"

fadeIn ardour 4 2000

// ===== Advanced: Track Setup Workflow =====

printfn "\n=== Advanced: Track Setup Workflow ==="

let setupVocalTrack trackNum =
    printfn "🎤 Setting up vocal track %d..." trackNum

    // Select the track
    ardour |> Osc.sendInt "/select/track" trackNum

    // Set track name
    ardour |> Osc.sendString "/strip/name" "Vocals"

    // Set initial volume
    ardour |> Osc.sendFloat "/strip/fader" 0.7

    // Pan to center
    ardour |> Osc.sendFloat "/strip/pan_stereo_position" 0.5

    // Enable record
    ardour |> Osc.sendInt "/strip/recenable" 1

    printfn "✅ Vocal track configured"

setupVocalTrack 2

// ===== Integration with MCP =====

printfn "\n=== MCP Integration Example ==="

// This shows how Ardour control could be exposed as an MCP tool
let createArdourTool () =
    // In the full MCP DSL, this would become:
    //
    // tool "ardour_play" "Start Ardour playback" {
    //     description "Starts playback in Ardour DAW"
    //     execute (fun _ ->
    //         ardour |> Osc.send "/transport_play"
    //         "Playback started"
    //     )
    // }
    //
    // tool "ardour_set_tempo" "Set Ardour tempo" {
    //     description "Sets the session tempo in BPM"
    //     parameter "bpm" "int" "Tempo in beats per minute"
    //     execute (fun args ->
    //         let bpm = args.["bpm"]
    //         ardour |> Osc.sendInt "/set_tempo" bpm
    //         sprintf "Tempo set to %d BPM" bpm
    //     )
    // }

    printfn "📦 MCP tools would be defined here"
    printfn "   - ardour_play: Start/stop playback"
    printfn "   - ardour_set_tempo: Change tempo"
    printfn "   - ardour_record: Start recording"
    printfn "   - ardour_track_volume: Adjust track volume"

createArdourTool ()

printfn "\n✅ Ardour OSC control demo complete!"
printfn ""
printfn "This example demonstrates:"
printfn "  ✓ Connecting to Ardour via OSC"
printfn "  ✓ Transport control (play, stop, record)"
printfn "  ✓ Session parameters (tempo, volume)"
printfn "  ✓ Track manipulation (select, volume, pan, mute, solo)"
printfn "  ✓ Automation workflows"
printfn "  ✓ Integration patterns for MCP servers"
printfn ""
printfn "Compare this concise Fusabi code to the original Python implementation!"
printfn "See RFD-001 for the full vision."
