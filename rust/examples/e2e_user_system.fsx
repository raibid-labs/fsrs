//! End-to-End Example: User and Role Management System
//!
//! Demonstrates practical usage of Records and Discriminated Unions together
//! in a realistic user management scenario.

// ============================================================================
// Type Definitions
// ============================================================================

// User roles as a simple enum
type Role =
    | Admin
    | Moderator
    | User
    | Guest

// User status with optional metadata
type UserStatus =
    | Active
    | Suspended of string  // reason
    | Banned of string * int  // reason, until_timestamp
    | Deleted

// Permission levels
type Permission =
    | Read
    | Write
    | Delete
    | ManageUsers

// User profile with mixed record/variant fields
type UserProfile = {
    id: int;
    username: string;
    email: Option<string>;
    role: Role;
    status: UserStatus;
    created_at: int;
    last_login: Option<int>
}

// Authentication result
type AuthResult =
    | Authenticated of UserProfile
    | Failed of string
    | RequiresMFA of string  // user_id

// ============================================================================
// Helper Functions
// ============================================================================

// Get permission level for a role
let get_permissions role =
    match role with
    | Admin -> [Read; Write; Delete; ManageUsers]
    | Moderator -> [Read; Write; Delete]
    | User -> [Read; Write]
    | Guest -> [Read]

// Check if a user has a specific permission
let has_permission user permission =
    let perms = get_permissions user.role in
    List.exists (fun p -> p = permission) perms

// Check if user is active
let is_active user =
    match user.status with
    | Active -> true
    | _ -> false

// Check if user can login
let can_login user =
    match user.status with
    | Active -> true
    | Suspended(_) -> false
    | Banned(_, _) -> false
    | Deleted -> false

// ============================================================================
// User Management Functions
// ============================================================================

// Create a new user
let create_user id username email =
    {
        id = id;
        username = username;
        email = Some(email);
        role = User;
        status = Active;
        created_at = 1234567890;
        last_login = None
    }

// Authenticate a user
let authenticate username password users =
    // Simplified authentication logic
    let user_opt = List.tryFind (fun u -> u.username = username) users in
    match user_opt with
    | Some(user) ->
        if can_login user then
            Authenticated(user)
        else
            Failed("Account suspended or banned")
    | None ->
        Failed("User not found")

// Promote user to a new role
let promote_user user new_role =
    { user with role = new_role }

// Suspend a user with a reason
let suspend_user user reason =
    { user with status = Suspended(reason) }

// Reactivate a suspended user
let reactivate_user user =
    match user.status with
    | Suspended(_) -> { user with status = Active }
    | Banned(_, _) -> user  // Cannot reactivate banned users
    | _ -> user

// Update last login timestamp
let record_login user timestamp =
    { user with last_login = Some(timestamp) }

// ============================================================================
// Authorization Functions
// ============================================================================

// Check if user can perform an action
let authorize user action =
    match action with
    | "read" -> has_permission user Read
    | "write" -> has_permission user Write && is_active user
    | "delete" -> has_permission user Delete && is_active user
    | "manage_users" -> has_permission user ManageUsers && is_active user
    | _ -> false

// Get user display name (username or email)
let get_display_name user =
    match user.email with
    | Some(email) -> email
    | None -> user.username

// Get status description
let get_status_description user =
    match user.status with
    | Active -> "Active"
    | Suspended(reason) -> "Suspended: " + reason
    | Banned(reason, _) -> "Banned: " + reason
    | Deleted -> "Deleted"

// ============================================================================
// Usage Examples
// ============================================================================

// Example 1: Create and authenticate users
let alice = create_user 1 "alice" "alice@example.com"
let bob = create_user 2 "bob" "bob@example.com"
let admin = promote_user (create_user 3 "admin" "admin@example.com") Admin

let users = [alice; bob; admin]

// Authenticate Alice
let auth_result = authenticate "alice" "password123" users
let authenticated_user = match auth_result with
    | Authenticated(u) -> Some(u)
    | Failed(_) -> None
    | RequiresMFA(_) -> None

// Example 2: Check permissions
let alice_can_write = match authenticated_user with
    | Some(u) -> authorize u "write"
    | None -> false

let alice_can_manage = match authenticated_user with
    | Some(u) -> authorize u "manage_users"
    | None -> false

// Example 3: User management operations
let alice_promoted = match authenticated_user with
    | Some(u) -> promote_user u Moderator
    | None -> alice

let alice_suspended = suspend_user alice "Suspicious activity"
let alice_can_login_suspended = can_login alice_suspended

let alice_reactivated = reactivate_user alice_suspended
let alice_can_login_reactivated = can_login alice_reactivated

// Example 4: Record login
let alice_with_login = match authenticated_user with
    | Some(u) -> record_login u 1234567900
    | None -> alice

let alice_last_login = match alice_with_login.last_login with
    | Some(ts) -> ts
    | None -> 0

// Example 5: Get user information
let alice_display = match authenticated_user with
    | Some(u) -> get_display_name u
    | None -> "Unknown"

let alice_status = match authenticated_user with
    | Some(u) -> get_status_description u
    | None -> "Not found"

// ============================================================================
// Results
// ============================================================================

// Return tuple of test results
(
    alice_can_write,           // true (User has Write permission and is Active)
    alice_can_manage,          // false (User doesn't have ManageUsers permission)
    alice_can_login_suspended, // false (Suspended users can't login)
    alice_can_login_reactivated, // true (Reactivated users can login)
    alice_last_login,          // 1234567900 (timestamp)
    List.length users          // 3 (number of users)
)
