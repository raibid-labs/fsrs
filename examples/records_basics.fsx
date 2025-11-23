(* FSRS Records - Basic Features Demo

   This example demonstrates the fundamental record features in FSRS:
   - Creating record literals with multiple fields
   - Accessing record fields
   - Nesting records
   - Using records with other data structures
*)

(* Example 1: Simple person record *)
let person = { name = "Alice"; age = 30; city = "NYC" } in

(* Example 2: Accessing fields *)
let personName = person.name in
let personAge = person.age in

(* Example 3: Computed fields *)
let point = { x = 5 + 3; y = 10 * 2; sum = (5 + 3) + (10 * 2) } in

(* Example 4: Nested records *)
let config = {
    app = {
        name = "MyApp";
        version = 1
    };
    server = {
        host = "localhost";
        port = 8080
    }
} in

(* Example 5: Accessing nested fields *)
let appName = config.app.name in
let serverPort = config.server.port in

(* Example 6: Records with boolean fields *)
let flags = { active = true; verified = false; admin = true } in
let canAccess = flags.active && flags.admin in

(* Example 7: Records in lists *)
let users = [
    { name = "Bob"; score = 95 };
    { name = "Carol"; score = 87 };
    { name = "Dave"; score = 92 }
] in
let first = List.head users in
first.name

(* Result: "Bob" *)
