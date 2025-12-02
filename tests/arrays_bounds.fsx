// Array Bounds Checking Test Script
// Tests various edge cases for Array.get and Array.set operations

// Test 1: Array.get with valid indices (should work)
let test_get_valid_indices =
    let arr = Array.make 5 100 in
    let _ = Array.set arr 0 10 in
    let _ = Array.set arr 1 20 in
    let _ = Array.set arr 2 30 in
    let _ = Array.set arr 3 40 in
    let _ = Array.set arr 4 50 in
    let v0 = Array.get arr 0 in
    let v1 = Array.get arr 1 in
    let v2 = Array.get arr 2 in
    let v3 = Array.get arr 3 in
    let v4 = Array.get arr 4 in
    print_string "Test 1 - Valid get indices: ";
    print_int v0;
    print_string ", ";
    print_int v1;
    print_string ", ";
    print_int v2;
    print_string ", ";
    print_int v3;
    print_string ", ";
    print_int v4;
    print_newline ()

// Test 2: Array.set with valid indices (should work)
let test_set_valid_indices =
    let arr = Array.make 3 0 in
    let _ = Array.set arr 0 111 in
    let _ = Array.set arr 1 222 in
    let _ = Array.set arr 2 333 in
    print_string "Test 2 - Valid set indices: ";
    print_int (Array.get arr 0);
    print_string ", ";
    print_int (Array.get arr 1);
    print_string ", ";
    print_int (Array.get arr 2);
    print_newline ()

// Test 3: Empty array edge case - length check
let test_empty_array_length =
    let arr = Array.make 0 42 in
    let len = Array.length arr in
    print_string "Test 3 - Empty array length: ";
    print_int len;
    print_newline ()

// Test 4: Single element array - boundary test
let test_single_element_array =
    let arr = Array.make 1 999 in
    let _ = Array.set arr 0 777 in
    let v = Array.get arr 0 in
    print_string "Test 4 - Single element array: ";
    print_int v;
    print_newline ()

// Test 5: Large array - test first and last valid indices
let test_large_array_boundaries =
    let arr = Array.make 100 0 in
    let _ = Array.set arr 0 1 in
    let _ = Array.set arr 99 100 in
    let first = Array.get arr 0 in
    let last = Array.get arr 99 in
    print_string "Test 5 - Large array boundaries (0, 99): ";
    print_int first;
    print_string ", ";
    print_int last;
    print_newline ()

// Test 6: Array.get with negative index (should cause runtime error)
let test_get_negative_index =
    print_string "Test 6 - Get with negative index: ";
    let arr = Array.make 5 0 in
    let v = Array.get arr (0 - 1) in
    print_int v;
    print_newline ()

// Test 7: Array.get with index at length (should cause runtime error)
let test_get_index_at_length =
    print_string "Test 7 - Get with index at length: ";
    let arr = Array.make 5 0 in
    let v = Array.get arr 5 in
    print_int v;
    print_newline ()

// Test 8: Array.get with index beyond length (should cause runtime error)
let test_get_index_beyond_length =
    print_string "Test 8 - Get with index beyond length: ";
    let arr = Array.make 5 0 in
    let v = Array.get arr 100 in
    print_int v;
    print_newline ()

// Test 9: Array.set with negative index (should cause runtime error)
let test_set_negative_index =
    print_string "Test 9 - Set with negative index: ";
    let arr = Array.make 5 0 in
    let _ = Array.set arr (0 - 1) 42 in
    print_string "Success";
    print_newline ()

// Test 10: Array.set with index at length (should cause runtime error)
let test_set_index_at_length =
    print_string "Test 10 - Set with index at length: ";
    let arr = Array.make 5 0 in
    let _ = Array.set arr 5 42 in
    print_string "Success";
    print_newline ()

// Test 11: Array.set with index beyond length (should cause runtime error)
let test_set_index_beyond_length =
    print_string "Test 11 - Set with index beyond length: ";
    let arr = Array.make 5 0 in
    let _ = Array.set arr 100 42 in
    print_string "Success";
    print_newline ()

// Test 12: Empty array with get at index 0 (should cause runtime error)
let test_empty_array_get =
    print_string "Test 12 - Empty array get at index 0: ";
    let arr = Array.make 0 0 in
    let v = Array.get arr 0 in
    print_int v;
    print_newline ()

// Test 13: Empty array with set at index 0 (should cause runtime error)
let test_empty_array_set =
    print_string "Test 13 - Empty array set at index 0: ";
    let arr = Array.make 0 0 in
    let _ = Array.set arr 0 42 in
    print_string "Success";
    print_newline ()

// Test 14: Large negative index (should cause runtime error)
let test_large_negative_index =
    print_string "Test 14 - Large negative index: ";
    let arr = Array.make 5 0 in
    let v = Array.get arr (0 - 1000) in
    print_int v;
    print_newline ()

// Main execution
// Run valid tests first (tests 1-5)
let _ = test_get_valid_indices in
let _ = test_set_valid_indices in
let _ = test_empty_array_length in
let _ = test_single_element_array in
let _ = test_large_array_boundaries in

// Separator before error tests
let _ = print_newline () in
let _ = print_string "=== Tests below should cause runtime errors ===" in
let _ = print_newline () in
let _ = print_newline () in

// Run tests that should cause errors (tests 6-14)
// Uncomment ONE test at a time to verify it produces the expected runtime error

// Negative index tests
// let _ = test_get_negative_index in
// let _ = test_set_negative_index in
// let _ = test_large_negative_index in

// Index at or beyond length tests
// let _ = test_get_index_at_length in
// let _ = test_get_index_beyond_length in
// let _ = test_set_index_at_length in
// let _ = test_set_index_beyond_length in

// Empty array tests
// let _ = test_empty_array_get in
// let _ = test_empty_array_set in

print_string "All valid tests completed successfully!";
print_newline ()
