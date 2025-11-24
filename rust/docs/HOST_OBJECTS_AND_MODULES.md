# Host Objects and Native Modules

This document describes the new features for host interop added to address [Issue #84](https://github.com/beengud/fusabi/issues/84).

## Overview

Fusabi now supports three major host interop features:

1. **Native Module Registration** - Group related host functions under a namespace
2. **Host Objects (HostData)** - Expose stateful Rust objects to scripts
3. **Method-like Syntax** - Call methods on host objects using module pattern

## 1. Native Module Registration

Instead of registering individual functions with manual name prefixing, you can now group related functions into modules:

### Example

```rust
use fusabi::{Engine, Module, Value};
use fusabi_vm::VmError;

let mut engine = Engine::new();

// Create a module for file system operations
let fs_module = Module::new("fs")
    .register_fn1("read", |path: Value| {
        let path_str = path.as_str()
            .ok_or_else(|| VmError::Runtime("Expected string path".into()))?;

        // Read file implementation
        Ok(Value::Str(format!("contents of {}", path_str)))
    })
    .register_fn2("write", |path: Value, contents: Value| {
        let path_str = path.as_str()
            .ok_or_else(|| VmError::Runtime("Expected string path".into()))?;
        let contents_str = contents.as_str()
            .ok_or_else(|| VmError::Runtime("Expected string contents".into()))?;

        // Write file implementation
        Ok(Value::Unit)
    });

// Register the module
engine.register_module(fs_module);

// Now you can call the functions with namespaced names
let result = engine.call_host("fs.read", &[Value::Str("file.txt".to_string())]);
```

### Module Builder API

The `Module` struct provides a fluent API for building modules:

- `Module::new(name)` - Create a new module
- `.register(name, fn)` - Register function with dynamic arity
- `.register_raw(name, fn)` - Register function that needs VM access
- `.register_fn0(name, fn)` - Register nullary function
- `.register_fn1(name, fn)` - Register unary function
- `.register_fn2(name, fn)` - Register binary function
- `.register_fn3(name, fn)` - Register ternary function

## 2. Host Objects (HostData)

You can now expose stateful Rust objects to Fusabi scripts using `HostData`:

### Example

```rust
use fusabi::{Engine, Value};

// Define a Rust struct to expose
#[derive(Debug)]
struct EventStore {
    events: Vec<String>,
}

impl EventStore {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn add_event(&mut self, event: String) {
        self.events.push(event);
    }

    fn count(&self) -> usize {
        self.events.len()
    }
}

let mut engine = Engine::new();

// Create a host object
let store = EventStore::new();
let store_value = engine.create_host_data(store, "EventStore");

// Store it as a global (accessible from scripts)
engine.set_global("event_store", store_value.clone());

// Access the host data from Rust
if let Some(mut store_ref) = store_value.as_host_data_of_mut::<EventStore>() {
    store_ref.add_event("test event".to_string());
}

// Check the type
assert!(store_value.is_host_data());
assert_eq!(store_value.type_name(), "host_data");
assert_eq!(store_value.type_name_string(), "EventStore");
```

### HostData API

- `engine.create_host_data(data, type_name)` - Create a host object value
- `value.is_host_data()` - Check if value is host data
- `value.as_host_data()` - Get reference to HostData wrapper
- `value.as_host_data_of::<T>()` - Get typed immutable reference
- `value.as_host_data_of_mut::<T>()` - Get typed mutable reference
- `value.type_name()` - Returns "host_data" (static lifetime)
- `value.type_name_string()` - Returns specific type name (e.g., "EventStore")

### Type Safety

HostData provides runtime type safety through Rust's `Any` trait:

```rust
struct DifferentType { value: i64 }

let store = EventStore::new();
let store_val = engine.create_host_data(store, "EventStore");

// Try to extract as wrong type - returns None
let wrong = store_val.as_host_data_of::<DifferentType>();
assert!(wrong.is_none());

// Extract as correct type - returns Some
let correct = store_val.as_host_data_of::<EventStore>();
assert!(correct.is_some());
```

## 3. Method-like Syntax Pattern

While Fusabi doesn't have native method call syntax (`obj.method()`), you can achieve similar ergonomics by combining HostData with modules:

### Example: Exposing an Event Store

```rust
use fusabi::{Engine, Module, Value};
use fusabi_vm::VmError;

#[derive(Debug)]
struct EventStore {
    events: Vec<String>,
}

impl EventStore {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn add_event(&mut self, event: String) {
        self.events.push(event);
    }

    fn get_events(&self) -> Vec<String> {
        self.events.clone()
    }

    fn count(&self) -> usize {
        self.events.len()
    }
}

let mut engine = Engine::new();

// Create and register the host object
let store = EventStore::new();
let store_value = engine.create_host_data(store, "EventStore");
engine.set_global("event_store", store_value);

// Register methods as a module
let event_store_module = Module::new("EventStore")
    .register_fn2("add_event", |store_val: Value, event: Value| {
        let event_str = event.as_str()
            .ok_or_else(|| VmError::Runtime("Expected string".into()))?;

        let mut store = store_val.as_host_data_of_mut::<EventStore>()
            .ok_or_else(|| VmError::Runtime("Expected EventStore".into()))?;

        store.add_event(event_str.to_string());
        Ok(Value::Unit)
    })
    .register_fn1("count", |store_val: Value| {
        let store = store_val.as_host_data_of::<EventStore>()
            .ok_or_else(|| VmError::Runtime("Expected EventStore".into()))?;

        Ok(Value::Int(store.count() as i64))
    })
    .register_fn1("get_events", |store_val: Value| {
        let store = store_val.as_host_data_of::<EventStore>()
            .ok_or_else(|| VmError::Runtime("Expected EventStore".into()))?;

        let events = store.get_events();
        let event_values: Vec<Value> = events
            .into_iter()
            .map(Value::Str)
            .collect();
        Ok(Value::vec_to_cons(event_values))
    });

engine.register_module(event_store_module);

// Usage from Rust (simulating script calls)
let store_global = engine.get_global("event_store").unwrap().clone();

// Add events: EventStore.add_event(event_store, "event 1")
engine.call_host(
    "EventStore.add_event",
    &[store_global.clone(), Value::Str("event 1".to_string())]
).unwrap();

// Get count: EventStore.count(event_store)
let count = engine.call_host("EventStore.count", &[store_global.clone()]).unwrap();
assert_eq!(count, Value::Int(1));
```

### From Fusabi Script (Future)

In the future, with syntax support, this could be called from scripts as:

```fsharp
# Get the global event store
let store = event_store in

# Add events
EventStore.add_event store "event 1"
EventStore.add_event store "event 2"

# Get count
let count = EventStore.count store in
print count
```

## Comparison with Old API

### Before: Manual Prefixing

```rust
engine.register_fn1("fs_read", |path| { /* ... */ });
engine.register_fn2("fs_write", |path, contents| { /* ... */ });
engine.register_fn1("db_connect", |conn_str| { /* ... */ });
engine.register_fn1("db_query", |query| { /* ... */ });
```

### After: Organized Modules

```rust
let fs_module = Module::new("fs")
    .register_fn1("read", |path| { /* ... */ })
    .register_fn2("write", |path, contents| { /* ... */ });

let db_module = Module::new("db")
    .register_fn1("connect", |conn_str| { /* ... */ })
    .register_fn1("query", |query| { /* ... */ });

engine.register_module(fs_module);
engine.register_module(db_module);
```

## Limitations and Notes

1. **Serialization**: HostData values cannot be serialized to bytecode files. They exist only for runtime interop.

2. **Lifetime**: HostData objects are reference-counted (`Rc<RefCell<T>>`), so they can be cloned and shared within a single thread.

3. **No Native Syntax**: Currently, you must call module functions with the full namespace (`"EventStore.add_event"`). Future versions may add native syntax sugar.

4. **Type Safety**: Type checking happens at runtime through Rust's `Any` trait. Incorrect type casts return `None`.

## Use Cases

### Observability Agent (Hibana Example)

This was the original motivating use case from Issue #84:

```rust
// High-performance event store exposed to scripts
struct EventStore {
    events: VecDeque<Event>,
    max_size: usize,
}

impl EventStore {
    fn add(&mut self, event: Event) { /* ... */ }
    fn query(&self, filter: &str) -> Vec<Event> { /* ... */ }
    fn count(&self) -> usize { /* ... */ }
}

// Expose to scripts
let store = EventStore::new(10000);
let store_value = engine.create_host_data(store, "EventStore");
engine.set_global("store", store_value);

// Register methods
let store_module = Module::new("EventStore")
    .register_fn2("add", |store, event| { /* ... */ })
    .register_fn2("query", |store, filter| { /* ... */ })
    .register_fn1("count", |store| { /* ... */ });

engine.register_module(store_module);
```

### Other Use Cases

- **Database Connections**: Expose connection pools
- **File Handles**: Manage open files
- **Network Sockets**: Handle TCP/UDP connections
- **Configuration Objects**: Runtime configuration
- **Cache Instances**: Shared caches

## Performance Considerations

- Host object access involves runtime type checking
- Method calls have minimal overhead (just function dispatch)
- Reference counting is used, not atomic reference counting (single-threaded)
- Borrow checking happens at runtime via `RefCell`

## Future Improvements

- Native method call syntax (`obj.method()`)
- Builder pattern for HostData with methods attached
- Automatic method discovery via traits
- Better error messages for type mismatches
