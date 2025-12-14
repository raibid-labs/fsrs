// Main file demonstrating multi-file module system

#load "math.fsx"
#load "utils.fsx"

// Use functions from loaded modules
let result = Math.pythagorean 3 4
let simple = Utils.add 10 20

result
