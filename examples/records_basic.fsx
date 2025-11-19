// Basic Record Examples for FSRS
// Demonstrates record creation, access, and updates

// Example 1: Simple record with primitive fields
let person = { name = "Alice"; age = 30; active = true }

// Access fields
let personName = person.name
let personAge = person.age

// Example 2: Record update (immutable)
let updatedPerson = { person with age = 31 }

// Example 3: Nested records
let address = { street = "123 Main St"; city = "Springfield"; zip = 12345 }
let personWithAddress = { name = "Bob"; age = 25; address = address }

// Access nested field
let city = personWithAddress.address.city

// Example 4: Record with list field
let team = { name = "Engineering"; members = ["Alice"; "Bob"; "Charlie"] }

// Example 5: Record with computed values
let point = { x = 10; y = 20 }
let distance = point.x * point.x + point.y * point.y

// Example 6: Multiple updates
let person2 = { name = "Charlie"; age = 40; active = false }
let person3 = { person2 with age = 41; active = true }

// Example 7: Record equality
let point1 = { x = 1; y = 2 }
let point2 = { x = 1; y = 2 }
let areEqual = point1 = point2  // Should be true

// Example 8: Empty record
let emptyRec = { }

// Example 9: Single field record
let singleton = { value = 42 }

// Example 10: Record with all types
let mixed = {
    intField = 42;
    boolField = true;
    strField = "hello";
    listField = [1; 2; 3];
    tupleField = (10, "test")
}
