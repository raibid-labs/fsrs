// Math module that depends on utils

#load "utils.fsx"

module Math =
    let pythagorean a b =
        let a2 = Utils.square a
        let b2 = Utils.square b
        Utils.add a2 b2
