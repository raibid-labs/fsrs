//! End-to-End Example: Result-Based Error Handling
//!
//! Demonstrates practical error handling patterns using Result types
//! combined with records for structured data.

// ============================================================================
// Type Definitions
// ============================================================================

// Generic Result type
type Result<'ok, 'err> =
    | Ok of 'ok
    | Error of 'err

// Error types
type ValidationError =
    | EmptyField of string  // field name
    | InvalidFormat of string * string  // field name, expected format
    | OutOfRange of string * int * int  // field name, min, max
    | Custom of string

type DatabaseError =
    | NotFound of string  // entity id
    | DuplicateKey of string  // key value
    | ConnectionFailed of string  // reason
    | QueryFailed of string  // query

type ApplicationError =
    | Validation of ValidationError
    | Database of DatabaseError
    | Unauthorized of string
    | NotImplemented

// Domain types using records
type User = {
    id: int;
    username: string;
    email: string;
    age: int
}

type CreateUserRequest = {
    username: string;
    email: string;
    age: int
}

type UpdateUserRequest = {
    id: int;
    username: Option<string>;
    email: Option<string>;
    age: Option<int>
}

// ============================================================================
// Validation Functions
// ============================================================================

// Validate username
let validate_username username =
    if String.length username = 0 then
        Error(EmptyField("username"))
    else if String.length username < 3 then
        Error(InvalidFormat("username", "at least 3 characters"))
    else
        Ok(username)

// Validate email
let validate_email email =
    if String.length email = 0 then
        Error(EmptyField("email"))
    else if not (String.contains email "@") then
        Error(InvalidFormat("email", "valid email address"))
    else
        Ok(email)

// Validate age
let validate_age age =
    if age < 13 then
        Error(OutOfRange("age", 13, 150))
    else if age > 150 then
        Error(OutOfRange("age", 13, 150))
    else
        Ok(age)

// Validate create user request
let validate_create_request request =
    match validate_username request.username with
    | Error(e) -> Error(Validation(e))
    | Ok(username) ->
        match validate_email request.email with
        | Error(e) -> Error(Validation(e))
        | Ok(email) ->
            match validate_age request.age with
            | Error(e) -> Error(Validation(e))
            | Ok(age) -> Ok(request)

// ============================================================================
// Database Operations (Simulated)
// ============================================================================

// Find user by ID
let find_user_by_id db_users id =
    let user_opt = List.tryFind (fun u -> u.id = id) db_users in
    match user_opt with
    | Some(user) -> Ok(user)
    | None -> Error(Database(NotFound(String.ofInt id)))

// Create user in database
let create_user_in_db db_users request next_id =
    // Check for duplicate username
    let duplicate = List.exists (fun u -> u.username = request.username) db_users in
    if duplicate then
        Error(Database(DuplicateKey(request.username)))
    else
        let new_user = {
            id = next_id;
            username = request.username;
            email = request.email;
            age = request.age
        } in
        Ok(new_user)

// Update user in database
let update_user_in_db db_users update_request =
    match find_user_by_id db_users update_request.id with
    | Error(e) -> Error(e)
    | Ok(user) ->
        let updated = {
            id = user.id;
            username = match update_request.username with
                       | Some(u) -> u
                       | None -> user.username;
            email = match update_request.email with
                    | Some(e) -> e
                    | None -> user.email;
            age = match update_request.age with
                  | Some(a) -> a
                  | None -> user.age
        } in
        Ok(updated)

// Delete user from database
let delete_user_from_db db_users id =
    match find_user_by_id db_users id with
    | Error(e) -> Error(e)
    | Ok(user) ->
        let remaining = List.filter (fun u -> u.id <> id) db_users in
        Ok(remaining)

// ============================================================================
// Service Layer Functions
// ============================================================================

// Create user with validation
let create_user db_users request next_id =
    match validate_create_request request with
    | Error(e) -> Error(e)
    | Ok(valid_request) ->
        create_user_in_db db_users valid_request next_id

// Update user with validation
let update_user db_users update_request =
    // Validate individual fields if provided
    let validate_optional validator opt_value =
        match opt_value with
        | Some(v) ->
            match validator v with
            | Ok(_) -> Ok(opt_value)
            | Error(e) -> Error(e)
        | None -> Ok(None)
    in

    match validate_optional validate_username update_request.username with
    | Error(e) -> Error(Validation(e))
    | Ok(_) ->
        match validate_optional validate_email update_request.email with
        | Error(e) -> Error(Validation(e))
        | Ok(_) ->
            match validate_optional validate_age update_request.age with
            | Error(e) -> Error(Validation(e))
            | Ok(_) ->
                update_user_in_db db_users update_request

// Get user by ID
let get_user db_users id =
    find_user_by_id db_users id

// Delete user
let delete_user db_users id =
    delete_user_from_db db_users id

// ============================================================================
// Result Combinators
// ============================================================================

// Map over a Result
let result_map f result =
    match result with
    | Ok(v) -> Ok(f v)
    | Error(e) -> Error(e)

// Bind/FlatMap for Result
let result_bind f result =
    match result with
    | Ok(v) -> f v
    | Error(e) -> Error(e)

// Extract value or use default
let result_or_default result default =
    match result with
    | Ok(v) -> v
    | Error(_) -> default

// Convert Result to Option
let result_to_option result =
    match result with
    | Ok(v) -> Some(v)
    | Error(_) -> None

// ============================================================================
// Error Formatting
// ============================================================================

// Get error message
let get_error_message error =
    match error with
    | Validation(EmptyField(field)) -> "Field " + field + " is required"
    | Validation(InvalidFormat(field, format)) ->
        "Field " + field + " has invalid format, expected: " + format
    | Validation(OutOfRange(field, min, max)) ->
        "Field " + field + " must be between " + String.ofInt min + " and " + String.ofInt max
    | Validation(Custom(msg)) -> msg
    | Database(NotFound(id)) -> "Entity with id " + id + " not found"
    | Database(DuplicateKey(key)) -> "Duplicate key: " + key
    | Database(ConnectionFailed(reason)) -> "Database connection failed: " + reason
    | Database(QueryFailed(query)) -> "Query failed: " + query
    | Unauthorized(msg) -> "Unauthorized: " + msg
    | NotImplemented -> "Not implemented"

// ============================================================================
// Usage Examples
// ============================================================================

// Initial database state
let db_users = [
    { id = 1; username = "alice"; email = "alice@example.com"; age = 30 };
    { id = 2; username = "bob"; email = "bob@example.com"; age = 25 }
]

// Example 1: Successful user creation
let create_request_valid = {
    username = "charlie";
    email = "charlie@example.com";
    age = 28
}

let result1 = create_user db_users create_request_valid 3
let user1_id = match result1 with
    | Ok(user) -> user.id
    | Error(_) -> 0

// Example 2: Failed validation - empty username
let create_request_invalid = {
    username = "";
    email = "test@example.com";
    age = 25
}

let result2 = create_user db_users create_request_invalid 4
let error2_msg = match result2 with
    | Ok(_) -> "Success"
    | Error(e) -> get_error_message e

// Example 3: Failed validation - invalid age
let create_request_invalid_age = {
    username = "david";
    email = "david@example.com";
    age = 10
}

let result3 = create_user db_users create_request_invalid_age 5
let error3_msg = match result3 with
    | Ok(_) -> "Success"
    | Error(e) -> get_error_message e

// Example 4: Successful user retrieval
let result4 = get_user db_users 1
let user4_name = match result4 with
    | Ok(user) -> user.username
    | Error(_) -> "not found"

// Example 5: Failed user retrieval
let result5 = get_user db_users 999
let error5_msg = match result5 with
    | Ok(_) -> "Success"
    | Error(e) -> get_error_message e

// Example 6: Successful update
let update_request_valid = {
    id = 1;
    username = Some("alice_updated");
    email = None;
    age = None
}

let result6 = update_user db_users update_request_valid
let user6_name = match result6 with
    | Ok(user) -> user.username
    | Error(_) -> "error"

// Example 7: Duplicate username
let create_request_duplicate = {
    username = "alice";
    email = "alice2@example.com";
    age = 35
}

let result7 = create_user db_users create_request_duplicate 6
let error7_msg = match result7 with
    | Ok(_) -> "Success"
    | Error(e) -> get_error_message e

// Example 8: Using result combinators
let result8 = result_map (fun user -> user.age) (get_user db_users 1)
let age8 = result_or_default result8 0

// Example 9: Chaining operations with bind
let result9 = result_bind
    (fun user ->
        if user.age > 18 then Ok(user) else Error(Unauthorized("Must be 18+")))
    (get_user db_users 1)

let is_authorized9 = match result9 with
    | Ok(_) -> true
    | Error(_) -> false

// ============================================================================
// Results
// ============================================================================

// Return tuple of test results
(
    user1_id,          // 3 (new user created successfully)
    error2_msg,        // "Field username is required"
    error3_msg,        // "Field age must be between 13 and 150"
    user4_name,        // "alice"
    error5_msg,        // "Entity with id 999 not found"
    user6_name,        // "alice_updated"
    error7_msg,        // "Duplicate key: alice"
    age8,              // 30
    is_authorized9     // true (alice is over 18)
)
