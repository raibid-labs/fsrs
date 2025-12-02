// Higher-Order Function Stress Test
// Tests nested closures, recursive patterns, chained operations, and more

// Test 1: Nested closures (closures that return closures)
let test_nested_closures =
    let make_adder = fun x -> fun y -> fun z -> x + y + z in
    let add5 = make_adder 5 in
    let add5and10 = add5 10 in
    let result = add5and10 20 in
    printfn "Test 1 - Nested closures: %d (expected 35)" result

// Test 2: More complex nested closures with captured variables
let test_closure_capture =
    let multiplier = 3 in
    let offset = 10 in
    let make_transform = fun factor ->
        fun value ->
            (value * factor * multiplier) + offset
    in
    let transform = make_transform 2 in
    let result = transform 5 in
    printfn "Test 2 - Closure capture: %d (expected 40)" result

// Test 3: Recursive patterns with List.fold - sum of list
let test_fold_sum =
    let numbers = [1; 2; 3; 4; 5; 6; 7; 8; 9; 10] in
    let sum = List.fold (fun acc x -> acc + x) 0 numbers in
    printfn "Test 3a - Fold sum: %d (expected 55)" sum

// Test 4: Recursive patterns with List.fold - product of list
let test_fold_product =
    let numbers = [1; 2; 3; 4; 5] in
    let product = List.fold (fun acc x -> acc * x) 1 numbers in
    printfn "Test 3b - Fold product: %d (expected 120)" product

// Test 5: Recursive patterns with List.fold - count elements
let test_fold_count =
    let numbers = [10; 20; 30; 40; 50] in
    let count = List.fold (fun acc _ -> acc + 1) 0 numbers in
    printfn "Test 3c - Fold count: %d (expected 5)" count

// Test 6: Recursive patterns with List.fold - max element
let test_fold_max =
    let numbers = [3; 7; 2; 9; 1; 5] in
    let max_value = List.fold (fun acc x -> if x > acc then x else acc) 0 numbers in
    printfn "Test 3d - Fold max: %d (expected 9)" max_value

// Test 7: Chained HOF operations - map then filter then fold
let test_chained_operations =
    let numbers = [1; 2; 3; 4; 5; 6; 7; 8; 9; 10] in
    let doubled = List.map (fun x -> x * 2) numbers in
    let evens_only = List.filter (fun x -> x > 10) doubled in
    let sum = List.fold (fun acc x -> acc + x) 0 evens_only in
    printfn "Test 4 - Chained operations: %d (expected 60)" sum

// Test 8: Complex chained operations with transformations
let test_complex_chain =
    let numbers = [1; 2; 3; 4; 5] in
    let squared = List.map (fun x -> x * x) numbers in
    let large = List.filter (fun x -> x >= 9) squared in
    let count = List.fold (fun acc _ -> acc + 1) 0 large in
    printfn "Test 4b - Complex chain count: %d (expected 3)" count

// Test 9: Closures capturing multiple outer variables
let test_multi_capture =
    let base = 100 in
    let factor = 2 in
    let offset = 5 in
    let transform = fun x -> (x * factor) + offset - base in
    let numbers = [50; 60; 70] in
    let results = List.map transform numbers in
    let sum = List.fold (fun acc x -> acc + x) 0 results in
    printfn "Test 5 - Multi-capture closure: %d (expected 45)" sum

// Test 10: Closure with conditional logic capturing variables
let test_conditional_capture =
    let threshold = 50 in
    let bonus = 10 in
    let adjust = fun x ->
        if x > threshold then
            x + bonus
        else
            x - bonus
    in
    let values = [40; 60; 30; 70] in
    let adjusted = List.map adjust values in
    let sum = List.fold (fun acc x -> acc + x) 0 adjusted in
    printfn "Test 5b - Conditional capture: %d (expected 170)" sum

// Test 11: List.iter with accumulation-like pattern
let test_iter_side_effects =
    let numbers = [1; 2; 3; 4; 5] in
    let dummy = List.iter (fun x -> printfn "  Processing: %d" x) numbers in
    printfn "Test 6 - List.iter completed (check output above)"

// Test 12: List.iter with complex operations
let test_iter_complex =
    let pairs = [(1, 10); (2, 20); (3, 30)] in
    let dummy = List.iter (fun (a, b) -> printfn "  Pair: %d + %d = %d" a b (a + b)) pairs in
    printfn "Test 6b - List.iter with pairs completed"

// Test 13: List.exists with simple predicate
let test_exists_simple =
    let numbers = [1; 2; 3; 4; 5] in
    let has_three = List.exists (fun x -> x = 3) numbers in
    let has_ten = List.exists (fun x -> x = 10) numbers in
    printfn "Test 7a - Exists (has 3): %b (expected true)" has_three

// Test 14: List.exists with complex predicate
let test_exists_complex =
    let numbers = [2; 4; 6; 8; 10] in
    let has_large_even = List.exists (fun x -> x > 5) numbers in
    printfn "Test 7b - Exists (large value): %b (expected true)" has_large_even

// Test 15: List.exists with captured variables in predicate
let test_exists_capture =
    let threshold = 15 in
    let multiplier = 2 in
    let numbers = [5; 10; 15; 20] in
    let exists_condition = List.exists (fun x -> (x * multiplier) > threshold) numbers in
    printfn "Test 7c - Exists with capture: %b (expected true)" exists_condition

// Test 16: List.find with simple predicate
let test_find_simple =
    let numbers = [1; 2; 3; 4; 5] in
    let found = List.find (fun x -> x > 3) numbers in
    printfn "Test 8a - Find first > 3: %d (expected 4)" found

// Test 17: List.find with complex predicate
let test_find_complex =
    let numbers = [10; 15; 20; 25; 30] in
    let found = List.find (fun x -> x > 18) numbers in
    printfn "Test 8b - Find first > 18: %d (expected 20)" found

// Test 18: List.tryFind with successful search
let test_tryFind_success =
    let numbers = [2; 4; 6; 8; 10] in
    let result = List.tryFind (fun x -> x > 7) numbers in
    match result with
    | Some value -> printfn "Test 8c - TryFind success: %d (expected 8)" value
    | None -> printfn "Test 8c - TryFind failed unexpectedly"

// Test 19: List.tryFind with unsuccessful search
let test_tryFind_failure =
    let numbers = [1; 2; 3; 4; 5] in
    let result = List.tryFind (fun x -> x > 10) numbers in
    match result with
    | Some value -> printfn "Test 8d - TryFind found unexpected: %d" value
    | None -> printfn "Test 8d - TryFind correctly returned None"

// Test 20: List.tryFind with captured variables
let test_tryFind_capture =
    let limit = 25 in
    let numbers = [10; 20; 30; 40] in
    let result = List.tryFind (fun x -> x >= limit) numbers in
    match result with
    | Some value -> printfn "Test 8e - TryFind with capture: %d (expected 30)" value
    | None -> printfn "Test 8e - TryFind failed unexpectedly"

// Test 21: Nested map operations
let test_nested_map =
    let numbers = [1; 2; 3; 4; 5] in
    let doubled = List.map (fun x -> x * 2) numbers in
    let squared = List.map (fun x -> x * x) doubled in
    let sum = List.fold (fun acc x -> acc + x) 0 squared in
    printfn "Test 9a - Nested map: %d (expected 440)" sum

// Test 22: Filter with complex conditions
let test_filter_complex =
    let numbers = [1; 2; 3; 4; 5; 6; 7; 8; 9; 10] in
    let filtered = List.filter (fun x -> x > 3) numbers in
    let filtered2 = List.filter (fun x -> x < 8) filtered in
    let count = List.fold (fun acc _ -> acc + 1) 0 filtered2 in
    printfn "Test 9b - Filter chain count: %d (expected 4)" count

// Test 23: Deep closure nesting
let test_deep_nesting =
    let f1 = fun a ->
        fun b ->
            fun c ->
                fun d ->
                    fun e ->
                        a + b + c + d + e
    in
    let result = f1 1 2 3 4 5 in
    printfn "Test 10a - Deep nesting: %d (expected 15)" result

// Test 24: Closure factory pattern
let test_closure_factory =
    let make_counter = fun start ->
        fun increment ->
            fun times ->
                let rec count n acc =
                    if n = 0 then
                        acc
                    else
                        count (n - 1) (acc + increment)
                in
                count times start
    in
    let counter = make_counter 10 in
    let counter5 = counter 5 in
    let result = counter5 3 in
    printfn "Test 10b - Closure factory: %d (expected 25)" result

// Test 25: Map with closure capturing loop variable pattern
let test_map_capture_pattern =
    let base_values = [10; 20; 30] in
    let multiplier = 3 in
    let transformed = List.map (fun x -> x * multiplier) base_values in
    let sum = List.fold (fun acc x -> acc + x) 0 transformed in
    printfn "Test 11 - Map capture pattern: %d (expected 180)" sum

// Test 26: All/forall pattern using fold
let test_all_pattern =
    let numbers = [2; 4; 6; 8; 10] in
    let all_positive = List.fold (fun acc x -> acc && (x > 0)) true numbers in
    printfn "Test 12a - All positive: %b (expected true)" all_positive

// Test 27: All/forall pattern with failure case
let test_all_pattern_fail =
    let numbers = [2; 4; -1; 8; 10] in
    let all_positive = List.fold (fun acc x -> acc && (x > 0)) true numbers in
    printfn "Test 12b - All positive (with negative): %b (expected false)" all_positive

// Test 28: Reverse using fold
let test_reverse_fold =
    let numbers = [1; 2; 3; 4; 5] in
    let reversed = List.fold (fun acc x -> x :: acc) [] numbers in
    let head = match reversed with
        | h :: _ -> h
        | [] -> 0
    in
    printfn "Test 13 - Reverse with fold (head): %d (expected 5)" head

// Test 29: Complex predicate composition
let test_predicate_composition =
    let is_large = fun x -> x > 10 in
    let is_small = fun x -> x < 100 in
    let numbers = [5; 15; 25; 35; 105] in
    let filtered = List.filter (fun x -> (is_large x) && (is_small x)) numbers in
    let count = List.fold (fun acc _ -> acc + 1) 0 filtered in
    printfn "Test 14 - Predicate composition: %d (expected 3)" count

// Test 30: Stress test with large list
let test_large_list =
    let rec make_list n acc =
        if n = 0 then
            acc
        else
            make_list (n - 1) (n :: acc)
    in
    let large_list = make_list 100 [] in
    let sum = List.fold (fun acc x -> acc + x) 0 large_list in
    printfn "Test 15 - Large list sum: %d (expected 5050)" sum

printfn ""
printfn "=== Higher-Order Function Stress Test Complete ==="
