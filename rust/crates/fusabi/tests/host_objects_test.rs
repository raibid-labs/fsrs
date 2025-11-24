// Integration tests for Host Objects and Modules
//
// This file tests the new features added in Issue #84:
// 1. Native Module Registration
// 2. Host Objects (HostData)
// 3. Method-like syntax for host objects via modules

use fusabi::{Engine, Module, Value};

// Example host object: EventStore for observability
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

    fn clear(&mut self) {
        self.events.clear();
    }
}

#[test]
fn test_host_data_creation() {
    let engine = Engine::new();
    let store = EventStore::new();
    let store_value = engine.create_host_data(store, "EventStore");

    assert!(store_value.is_host_data());
    assert_eq!(store_value.type_name(), "host_data");
    assert_eq!(store_value.type_name_string(), "EventStore");
}

#[test]
fn test_host_data_extraction() {
    let engine = Engine::new();
    let mut store = EventStore::new();
    store.add_event("test event".to_string());

    let store_value = engine.create_host_data(store, "EventStore");

    // Extract the data
    let borrowed = store_value.as_host_data_of::<EventStore>().unwrap();
    assert_eq!(borrowed.count(), 1);
}

#[test]
fn test_host_data_mutable_access() {
    let engine = Engine::new();
    let store = EventStore::new();
    let store_value = engine.create_host_data(store, "EventStore");

    // Mutate the data
    {
        let mut borrowed = store_value.as_host_data_of_mut::<EventStore>().unwrap();
        borrowed.add_event("event 1".to_string());
        borrowed.add_event("event 2".to_string());
    }

    // Verify the mutation
    let borrowed = store_value.as_host_data_of::<EventStore>().unwrap();
    assert_eq!(borrowed.count(), 2);
}

#[test]
fn test_module_registration() {
    let mut engine = Engine::new();

    let fs_module = Module::new("fs")
        .register_fn1("read", |path: Value| {
            let path_str = path
                .as_str()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string path".into()))?;
            Ok(Value::Str(format!("contents of {}", path_str)))
        })
        .register_fn2("write", |path: Value, contents: Value| {
            let _path_str = path
                .as_str()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string path".into()))?;
            let _contents_str = contents
                .as_str()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string contents".into()))?;
            Ok(Value::Unit)
        });

    engine.register_module(fs_module);

    // Verify the functions are registered with namespaced names
    assert!(engine.has_host_function("fs.read"));
    assert!(engine.has_host_function("fs.write"));
}

#[test]
fn test_module_function_call() {
    let mut engine = Engine::new();

    let math_module = Module::new("math")
        .register_fn1("square", |x: Value| {
            let n = x
                .as_int()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
            Ok(Value::Int(n * n))
        })
        .register_fn2("add", |a: Value, b: Value| {
            let x = a
                .as_int()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
            let y = b
                .as_int()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected int".into()))?;
            Ok(Value::Int(x + y))
        });

    engine.register_module(math_module);

    // Call the functions
    let result = engine.call_host("math.square", &[Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(25));

    let result = engine
        .call_host("math.add", &[Value::Int(10), Value::Int(32)])
        .unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_host_object_with_module_pattern() {
    let mut engine = Engine::new();

    // Create a host object
    let store = EventStore::new();
    let store_value = engine.create_host_data(store, "EventStore");

    // Store it as a global (simulating passing it to scripts)
    engine.set_global("event_store", store_value.clone());

    // Register methods as a module that work on the EventStore
    let event_store_module = Module::new("EventStore")
        .register_fn2("add_event", |store_val: Value, event: Value| {
            let event_str = event
                .as_str()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string event".into()))?;

            let mut store = store_val
                .as_host_data_of_mut::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;

            store.add_event(event_str.to_string());
            Ok(Value::Unit)
        })
        .register_fn1("count", |store_val: Value| {
            let store = store_val
                .as_host_data_of::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;

            Ok(Value::Int(store.count() as i64))
        })
        .register_fn1("get_events", |store_val: Value| {
            let store = store_val
                .as_host_data_of::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;

            let events = store.get_events();
            let event_values: Vec<Value> = events.into_iter().map(Value::Str).collect();
            Ok(Value::vec_to_cons(event_values))
        });

    engine.register_module(event_store_module);

    // Now we can call methods on the event store
    let store_val = engine.get_global("event_store").unwrap().clone();

    // Add some events
    engine
        .call_host(
            "EventStore.add_event",
            &[store_val.clone(), Value::Str("event 1".to_string())],
        )
        .unwrap();

    engine
        .call_host(
            "EventStore.add_event",
            &[store_val.clone(), Value::Str("event 2".to_string())],
        )
        .unwrap();

    // Check the count
    let count = engine
        .call_host("EventStore.count", std::slice::from_ref(&store_val))
        .unwrap();
    assert_eq!(count, Value::Int(2));

    // Get the events
    let events = engine
        .call_host("EventStore.get_events", &[store_val])
        .unwrap();
    let events_vec = events.list_to_vec().unwrap();
    assert_eq!(events_vec.len(), 2);
    assert_eq!(events_vec[0], Value::Str("event 1".to_string()));
    assert_eq!(events_vec[1], Value::Str("event 2".to_string()));
}

#[test]
fn test_multiple_modules() {
    let mut engine = Engine::new();

    let fs_module = Module::new("fs").register_fn1("exists", |_path: Value| Ok(Value::Bool(true)));

    let db_module = Module::new("db").register_fn1("connect", |_conn_str: Value| Ok(Value::Unit));

    let http_module = Module::new("http")
        .register_fn1("get", |_url: Value| Ok(Value::Str("response".to_string())));

    engine.register_module(fs_module);
    engine.register_module(db_module);
    engine.register_module(http_module);

    assert!(engine.has_host_function("fs.exists"));
    assert!(engine.has_host_function("db.connect"));
    assert!(engine.has_host_function("http.get"));

    // Ensure no cross-contamination
    assert!(!engine.has_host_function("fs.connect"));
    assert!(!engine.has_host_function("db.exists"));
}

#[test]
fn test_host_data_equality() {
    let engine = Engine::new();

    let store1 = EventStore::new();
    let store1_val = engine.create_host_data(store1, "EventStore");

    let store2 = EventStore::new();
    let store2_val = engine.create_host_data(store2, "EventStore");

    // Different instances should not be equal
    assert_ne!(store1_val, store2_val);

    // Same instance should be equal
    let store1_val_clone = store1_val.clone();
    assert_eq!(store1_val, store1_val_clone);
}

#[test]
fn test_host_data_type_safety() {
    let engine = Engine::new();

    struct DifferentType {
        #[allow(dead_code)]
        value: i64,
    }

    let store = EventStore::new();
    let store_val = engine.create_host_data(store, "EventStore");

    // Try to extract as wrong type - should fail
    let wrong_type = store_val.as_host_data_of::<DifferentType>();
    assert!(wrong_type.is_none());

    // Extract as correct type - should succeed
    let correct_type = store_val.as_host_data_of::<EventStore>();
    assert!(correct_type.is_some());
}

#[test]
fn test_complex_host_object_scenario() {
    // This test simulates the Hibana use case from the GitHub issue
    let mut engine = Engine::new();

    // Create an event store
    let store = EventStore::new();
    let store_val = engine.create_host_data(store, "EventStore");
    engine.set_global("store", store_val.clone());

    // Register the event store module with all methods
    let event_module = Module::new("EventStore")
        .register_fn2("add", |store: Value, event: Value| {
            let event_str = event
                .as_str()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected string".into()))?;
            let mut s = store
                .as_host_data_of_mut::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;
            s.add_event(event_str.to_string());
            Ok(Value::Unit)
        })
        .register_fn1("count", |store: Value| {
            let s = store
                .as_host_data_of::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;
            Ok(Value::Int(s.count() as i64))
        })
        .register_fn1("clear", |store: Value| {
            let mut s = store
                .as_host_data_of_mut::<EventStore>()
                .ok_or_else(|| fusabi_vm::VmError::Runtime("Expected EventStore".into()))?;
            s.clear();
            Ok(Value::Unit)
        });

    engine.register_module(event_module);

    // Simulate script usage
    let store_global = engine.get_global("store").unwrap().clone();

    // Add events
    for i in 1..=5 {
        engine
            .call_host(
                "EventStore.add",
                &[store_global.clone(), Value::Str(format!("event {}", i))],
            )
            .unwrap();
    }

    // Verify count
    let count = engine
        .call_host("EventStore.count", std::slice::from_ref(&store_global))
        .unwrap();
    assert_eq!(count, Value::Int(5));

    // Clear
    engine
        .call_host("EventStore.clear", std::slice::from_ref(&store_global))
        .unwrap();

    // Verify count after clear
    let count = engine
        .call_host("EventStore.count", &[store_global])
        .unwrap();
    assert_eq!(count, Value::Int(0));
}
