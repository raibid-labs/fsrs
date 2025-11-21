// Axum + Fusabi Web Server Example
// Validation logic implemented in F# scripts

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use fusabi_vm::{Vm, Value};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    age: i32,
}

#[derive(Debug, Serialize)]
struct CreateUserResponse {
    id: u64,
    username: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    field: String,
    message: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct ValidationResponse {
    valid: bool,
    errors: Vec<ValidationError>,
}

#[derive(Clone)]
struct AppState {
    validation_script: Arc<RwLock<String>>,
}

#[tokio::main]
async fn main() {
    // Load validation script
    let validation_script = fs::read_to_string("validation.fsx")
        .unwrap_or_else(|_| {
            // Default validation if file not found
            r#"
            // User validation rules
            let validateUser username email age =
                let mutable errors = []

                // Username validation
                if String.length username < 3 then
                    errors <- { field = "username"; message = "Username must be at least 3 characters"; code = "MIN_LENGTH" } :: errors

                if String.length username > 20 then
                    errors <- { field = "username"; message = "Username must be at most 20 characters"; code = "MAX_LENGTH" } :: errors

                // Email validation
                if not (String.contains "@" email) then
                    errors <- { field = "email"; message = "Invalid email format"; code = "INVALID_FORMAT" } :: errors

                if not (String.contains "." email) then
                    errors <- { field = "email"; message = "Email must contain domain"; code = "INVALID_DOMAIN" } :: errors

                // Age validation
                if age < 13 then
                    errors <- { field = "age"; message = "Must be at least 13 years old"; code = "MIN_AGE" } :: errors

                if age > 120 then
                    errors <- { field = "age"; message = "Invalid age"; code = "MAX_AGE" } :: errors

                errors

            validateUser username email age
            "#.to_string()
        });

    let state = AppState {
        validation_script: Arc::new(RwLock::new(validation_script)),
    };

    // Build application routes
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/validate", post(validate_user))
        .route("/reload", post(reload_validation))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");
    println!("📝 Edit validation.fsx to change validation rules");
    println!("🔄 POST /reload to reload validation script");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Fusabi Web Server Example - POST to /users to create a user"
}

async fn health() -> &'static str {
    "OK"
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, Json<ValidationResponse>)> {
    // Run validation
    let validation_result = validate_user_internal(&state, &payload).await;

    if !validation_result.errors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ValidationResponse {
                valid: false,
                errors: validation_result.errors,
            }),
        ));
    }

    // Validation passed - create user
    let response = CreateUserResponse {
        id: 12345, // Would be generated in real app
        username: payload.username,
        message: "User created successfully".to_string(),
    };

    Ok(Json(response))
}

async fn validate_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<ValidationResponse> {
    let result = validate_user_internal(&state, &payload).await;
    Json(result)
}

async fn validate_user_internal(
    state: &AppState,
    user: &CreateUserRequest,
) -> ValidationResponse {
    let script = state.validation_script.read().await;

    // Create VM and run validation
    let mut vm = Vm::new();

    // Register string functions
    vm.register_host_function("String.length", |args| {
        if let Some(s) = args.first().and_then(|v| v.as_str()) {
            Ok(Value::Int(s.len() as i64))
        } else {
            Err("String.length expects a string".to_string())
        }
    });

    vm.register_host_function("String.contains", |args| {
        if args.len() != 2 {
            return Err("String.contains expects 2 arguments".to_string());
        }
        if let (Some(needle), Some(haystack)) =
            (args[0].as_str(), args[1].as_str()) {
            Ok(Value::Bool(haystack.contains(needle)))
        } else {
            Err("String.contains expects strings".to_string())
        }
    });

    // Set user data as globals
    vm.set_global("username", Value::Str(user.username.clone()));
    vm.set_global("email", Value::Str(user.email.clone()));
    vm.set_global("age", Value::Int(user.age as i64));

    // Execute validation script
    // For now, return mock validation results
    // When WS1 is complete, this will execute the actual script

    let mut errors = Vec::new();

    // Mock validation logic (replace with script execution)
    if user.username.len() < 3 {
        errors.push(ValidationError {
            field: "username".to_string(),
            message: "Username must be at least 3 characters".to_string(),
            code: "MIN_LENGTH".to_string(),
        });
    }

    if !user.email.contains('@') {
        errors.push(ValidationError {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
            code: "INVALID_FORMAT".to_string(),
        });
    }

    if user.age < 13 {
        errors.push(ValidationError {
            field: "age".to_string(),
            message: "Must be at least 13 years old".to_string(),
            code: "MIN_AGE".to_string(),
        });
    }

    ValidationResponse {
        valid: errors.is_empty(),
        errors,
    }
}

async fn reload_validation(State(state): State<AppState>) -> StatusCode {
    // Reload validation script from file
    match fs::read_to_string("validation.fsx") {
        Ok(new_script) => {
            let mut script = state.validation_script.write().await;
            *script = new_script;
            println!("✅ Validation script reloaded");
            StatusCode::OK
        }
        Err(e) => {
            println!("❌ Failed to reload validation script: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}