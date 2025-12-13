# Type Providers in Fusabi

Type providers bring **compile-time type safety** to external data sources. Inspired by F#'s pioneering type provider system, Fusabi's implementation enables strongly-typed access to APIs, configuration files, databases, and any schema-defined data.

## Overview

### What Are Type Providers?

Type providers are compiler plugins that generate types from external schemas at compile time. Instead of manually defining types that mirror your data sources, type providers automatically create them from the source of truth.

```
┌─────────────────────┐     compile-time      ┌──────────────────┐
│   External Schema   │ ────────────────────► │   Fusabi Types   │
│   (JSON, GraphQL,   │     type provider     │   (records, DUs) │
│    OpenAPI, etc.)   │                       │                  │
└─────────────────────┘                       └──────────────────┘
```

### Why Type Providers?

| Problem | Without Type Providers | With Type Providers |
|---------|----------------------|---------------------|
| API field names | Runtime errors from typos | Compile-time errors |
| Config variables | `env["DATABSE_URL"]` works until prod | `Env.DATABSE_URL` won't compile |
| Schema changes | Silent failures, stale types | Regenerate and see all breakages |
| Documentation | Separate from code | Embedded in types |

## Available Providers

### 1. JSON Schema Provider

Generates types from JSON Schema definitions.

```fsharp
type User = JsonSchemaProvider<"file://./user.schema.json">

// Generated from schema:
// {
//   "type": "object",
//   "properties": {
//     "id": { "type": "integer" },
//     "name": { "type": "string" },
//     "email": { "type": "string" }
//   },
//   "required": ["id", "name"]
// }

let user: User = {
    id = 42
    name = "Alice"
    email = Some "alice@example.com"  // optional field
}
```

**Use cases:**
- Validating API request/response bodies
- Configuration file schemas
- Data interchange formats

---

### 2. Kubernetes Provider

Generates types from Kubernetes OpenAPI specifications.

```fsharp
type K8s = KubernetesProvider<"file://./k8s-openapi-v1.28.json">

// Full intellisense for K8s resources
let pod: K8s.Core.V1.Pod = {
    apiVersion = "v1"
    kind = "Pod"
    metadata = {
        name = "my-app"
        namespace = Some "production"
        labels = Some [("app", "my-app")]
    }
    spec = {
        containers = [{
            name = "main"
            image = "my-app:latest"
            ports = Some [{ containerPort = 8080 }]
        }]
    }
}

// Type errors for invalid fields:
// pod.spec.contianers  // Compile error: did you mean 'containers'?
```

**Use cases:**
- Infrastructure as Code with type safety
- Kubernetes operators and controllers
- GitOps manifest validation

---

### 3. OpenTelemetry Provider

Generates types from OpenTelemetry semantic conventions.

```fsharp
type OTel = OpenTelemetryProvider<"embedded">

// Type-safe span attributes
let httpSpan = OTel.Http.HttpClient {
    requestMethod = "GET"                    // required
    requestUrl = "https://api.example.com"   // required
    responseStatusCode = Some 200            // conditionally required
    requestBodySize = Some 1024              // recommended
}

// Database span with correct attribute names
let dbSpan = OTel.Db.Db {
    system = "postgresql"
    name = Some "users_db"
    operation = Some "SELECT"
    statement = Some "SELECT * FROM users WHERE id = $1"
}

// Compile errors for:
// httpSpan.requstMethod   // typo caught!
// httpSpan.status_code    // wrong name (should be responseStatusCode)
```

**Supported convention categories:**
- HTTP (client & server spans)
- Database (SQL, NoSQL)
- Messaging (Kafka, RabbitMQ, etc.)
- gRPC
- Exceptions/Events
- Resource attributes (service, process, host, cloud)

**Use cases:**
- Consistent observability across services
- Onboarding new team members (autocomplete shows valid attributes)
- Compliance with OTel semantic conventions

---

### 4. GraphQL Provider

Generates types from any GraphQL API via introspection.

```fsharp
// Works with ANY GraphQL endpoint
type GitHub = GraphQLProvider<"file://./github-schema.json">

// All types from the API are available
let query = GitHub.Types.Repository {
    name = "fusabi"
    owner = { login = "anthropics" }
    stargazerCount = Some 1000
    issues = Some {
        nodes = [
            { title = "Feature request"; state = GitHub.Enums.IssueState.Open }
        ]
    }
}

// Enums become discriminated unions
let status: GitHub.Enums.Status = Active

// Input types for mutations
let createIssue: GitHub.Inputs.CreateIssueInput = {
    title = "Bug report"
    body = Some "Description here"
    repositoryId = "repo-123"
}
```

**What gets generated:**
| GraphQL | Fusabi |
|---------|--------|
| `type User { ... }` | `record User = { ... }` |
| `enum Status { ACTIVE, INACTIVE }` | `type Status = Active \| Inactive` |
| `union SearchResult = User \| Repo` | `type SearchResult = User of User \| Repo of Repo` |
| `input CreateUserInput { ... }` | `record CreateUserInput = { ... }` |
| `String!` (non-null) | `string` |
| `String` (nullable) | `string option` |
| `[String!]!` | `string list` |

**Use cases:**
- Type-safe GraphQL clients
- API exploration with autocomplete
- Catching breaking API changes at compile time

---

### 5. Environment Config Provider

Generates types from `.env` files or JSON schemas.

```fsharp
type Env = EnvConfigProvider<"file://.env.example">

// From .env.example:
// DATABASE_URL=postgres://localhost/mydb
// DATABASE_POOL_SIZE=10
// PORT=3000
// DEBUG=false
// API_KEY=              # empty = required at runtime

// Typed access - no more string typos!
let dbUrl: string = Env.DATABASE_URL      // inferred as URL type
let poolSize: int = Env.DATABASE_POOL_SIZE // inferred as int
let port: int = Env.PORT
let debug: bool = Env.DEBUG
let apiKey: string = Env.API_KEY          // required, no default

// Type errors:
// Env.DATABSE_URL   // typo caught at compile time!
// Env.PORT + "x"    // type error: int vs string
```

**Type inference from values:**
| Value | Inferred Type |
|-------|---------------|
| `hello` | `string` |
| `42` | `int` |
| `3.14` | `float` |
| `true`/`false` | `bool` |
| `postgres://...` | `string` (url) |
| `/var/log/app.log` | `string` (path) |
| `a,b,c` | `string list` |
| `1,2,3` | `int list` |
| `` (empty) | required field |

**JSON Schema support:**
```json
{
  "properties": {
    "DATABASE_URL": { "type": "string", "format": "uri" },
    "PORT": { "type": "integer", "default": 3000 },
    "FEATURES": { "type": "array", "items": { "type": "string" } }
  },
  "required": ["DATABASE_URL"]
}
```

**Use cases:**
- 12-factor app configuration
- Environment-specific settings
- Secrets management validation

---

## Architecture

### The TypeProvider Trait

All providers implement this core trait:

```rust
pub trait TypeProvider: Send + Sync {
    /// Unique provider identifier
    fn name(&self) -> &str;

    /// Resolve schema from source (file, URL, version string)
    fn resolve_schema(&self, source: &str, params: &ProviderParams)
        -> ProviderResult<Schema>;

    /// Generate Fusabi types from the resolved schema
    fn generate_types(&self, schema: &Schema, namespace: &str)
        -> ProviderResult<GeneratedTypes>;

    /// Optional: Get documentation for a type path
    fn get_documentation(&self, type_path: &str) -> Option<String> {
        None
    }
}
```

### Type Generation Flow

```
1. Declaration
   type API = GraphQLProvider<"https://api.example.com/graphql">
                    │
                    ▼
2. Schema Resolution
   Provider fetches/reads the external schema
   (introspection query, file read, HTTP fetch)
                    │
                    ▼
3. Parsing
   Schema is parsed into internal representation
   (JSON Schema AST, GraphQL types, etc.)
                    │
                    ▼
4. Type Generation
   Internal schema → Fusabi TypeDefinitions
   (Records, Discriminated Unions)
                    │
                    ▼
5. Type Injection
   Generated types added to compiler's type environment
   (Available for type checking, autocomplete)
```

### Caching

Schemas are cached to avoid repeated fetches:

```rust
pub struct SchemaCache {
    entries: HashMap<String, CacheEntry>,
    default_ttl: Duration,
}
```

- File-based schemas: Cached until file modification
- HTTP schemas: Configurable TTL (default: 1 hour)
- Embedded schemas: No caching needed

---

## Creating Custom Providers

### Example: TOML Config Provider

```rust
use crate::type_provider::{TypeProvider, Schema, GeneratedTypes, ...};

pub struct TomlConfigProvider;

impl TypeProvider for TomlConfigProvider {
    fn name(&self) -> &str {
        "TomlConfigProvider"
    }

    fn resolve_schema(&self, source: &str, _params: &ProviderParams)
        -> ProviderResult<Schema>
    {
        let path = source.strip_prefix("file://").unwrap_or(source);
        let content = std::fs::read_to_string(path)?;
        Ok(Schema::Custom(content))
    }

    fn generate_types(&self, schema: &Schema, namespace: &str)
        -> ProviderResult<GeneratedTypes>
    {
        let content = match schema {
            Schema::Custom(s) => s,
            _ => return Err(ProviderError::ParseError("Expected TOML".into())),
        };

        let toml: toml::Value = toml::from_str(content)?;

        // Convert TOML tables to records
        let types = convert_toml_to_types(&toml, namespace)?;
        Ok(types)
    }
}
```

### Provider Registration

```rust
let mut registry = ProviderRegistry::new();
registry.register(Arc::new(TomlConfigProvider));
registry.register(Arc::new(JsonSchemaProvider::new()));
registry.register(Arc::new(GraphQLProvider::new()));
// ... etc
```

---

## Best Practices

### 1. Version Your Schemas

```fsharp
// Bad: Schema can change unexpectedly
type API = GraphQLProvider<"https://api.example.com/graphql">

// Good: Pin to specific version
type API = GraphQLProvider<"file://./schemas/api-v2.3.json">
```

### 2. Use Embedded/Local for CI

```fsharp
// Development: Live endpoint for latest types
type API = GraphQLProvider<"https://api.dev.example.com/graphql">

// CI/Production: Local schema for reproducibility
type API = GraphQLProvider<"file://./api-schema.json">
```

### 3. Document Required vs Optional

```env
# .env.example

# Required - app won't start without these
DATABASE_URL=
API_SECRET=

# Optional - have sensible defaults
PORT=3000
LOG_LEVEL=info
DEBUG=false
```

### 4. Group Related Config

```env
# Database settings
DATABASE_URL=postgres://localhost/mydb
DATABASE_POOL_SIZE=10
DATABASE_TIMEOUT_MS=5000

# Redis settings
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=5
```

This generates grouped types:
```fsharp
Env.Groups.Database.url
Env.Groups.Database.poolSize
Env.Groups.Redis.url
```

---

## Comparison with Other Approaches

| Approach | Pros | Cons |
|----------|------|------|
| **Type Providers** | Compile-time safety, single source of truth, auto-generated | Requires schema access at compile time |
| **Manual Types** | Full control, no dependencies | Drift from reality, maintenance burden |
| **Runtime Validation** | Works without schemas | Errors only at runtime |
| **Code Generation** | Similar benefits | Separate build step, generated code in repo |

Type providers combine the best aspects: types are generated from schemas but integrated into the normal compilation process.

---

## Future Providers

Planned additions:
- **SQL Provider** - Types from database schemas
- **Protobuf Provider** - Types from .proto files
- **AsyncAPI Provider** - Event-driven API types
- **Terraform Provider** - Infrastructure resource types

---

## Summary

Type providers transform Fusabi from a dynamically-feeling scripting language into one with the type safety of statically-typed systems, while keeping the ergonomics of schema-first development. By deriving types from external sources, you get:

1. **Correctness** - No drift between types and reality
2. **Discoverability** - Autocomplete shows valid options
3. **Refactoring** - Schema changes surface as compile errors
4. **Documentation** - Types are self-documenting

Start with the provider that matches your most error-prone integration, and expand from there.
