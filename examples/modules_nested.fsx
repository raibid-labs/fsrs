// Nested modules example
//
// This example demonstrates:
// - Nested module definitions
// - Qualified access to nested modules
// - Multiple levels of nesting

module Geometry =
    module Point =
        let make x y = (x, y)

        let distance p1 p2 =
            let (x1, y1) = p1
            let (x2, y2) = p2
            let dx = x2 - x1
            let dy = y2 - y1
            // Note: sqrt not implemented yet, using simple calculation
            dx * dx + dy * dy

    let origin = Point.make 0 0

// Create points using qualified access
let p1 = Geometry.Point.make 3 4
let p2 = Geometry.Point.make 6 8

// Calculate distance
let dist = Geometry.Point.distance p1 p2

dist
