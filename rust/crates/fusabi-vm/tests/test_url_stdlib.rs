// Integration tests for Url stdlib module

use fusabi_vm::stdlib::register_stdlib;
use fusabi_vm::stdlib::url::{url_decode, url_encode, url_is_valid, url_parse};
use fusabi_vm::value::Value;
use fusabi_vm::vm::Vm;

#[test]
fn test_url_parse_integration() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);

    // Check that Url module is registered
    assert!(vm.globals.contains_key("Url"));

    // Check that Url module has the expected functions
    if let Some(Value::Record(url_module)) = vm.globals.get("Url") {
        let module = url_module.lock().unwrap();
        assert!(module.contains_key("parse"));
        assert!(module.contains_key("isValid"));
        assert!(module.contains_key("encode"));
        assert!(module.contains_key("decode"));
    } else {
        panic!("Url module not found in globals");
    }

    // Test that functions are registered in host registry
    let registry = vm.host_registry.lock().unwrap();
    assert!(registry.has_function("Url.parse"));
    assert!(registry.has_function("Url.isValid"));
    assert!(registry.has_function("Url.encode"));
    assert!(registry.has_function("Url.decode"));
}

#[test]
fn test_url_parse_call() {
    // Call Url.parse directly
    let url_str = Value::Str("https://example.com:8080/path?query=value#fragment".to_string());
    let result = url_parse(&url_str).unwrap();

    // Should return Some(UrlInfo)
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields.len(), 1);

            // Check that we got a record back
            if let Value::Record(record) = &fields[0] {
                let r = record.lock().unwrap();
                assert_eq!(r.get("scheme"), Some(&Value::Str("https".to_string())));
                assert_eq!(r.get("host"), Some(&Value::Str("example.com".to_string())));
                assert_eq!(r.get("path"), Some(&Value::Str("/path".to_string())));

                // Check port is Some(8080)
                if let Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) = r.get("port")
                {
                    assert_eq!(variant_name, "Some");
                    assert_eq!(fields[0], Value::Int(8080));
                }

                // Check query is Some("query=value")
                if let Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) = r.get("query")
                {
                    assert_eq!(variant_name, "Some");
                    assert_eq!(fields[0], Value::Str("query=value".to_string()));
                }

                // Check fragment is Some("fragment")
                if let Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) = r.get("fragment")
                {
                    assert_eq!(variant_name, "Some");
                    assert_eq!(fields[0], Value::Str("fragment".to_string()));
                }
            } else {
                panic!("Expected Record");
            }
        }
        _ => panic!("Expected Some variant"),
    }
}

#[test]
fn test_url_is_valid_call() {
    // Valid URL
    let valid_url = Value::Str("https://example.com".to_string());
    let result = url_is_valid(&valid_url).unwrap();
    assert_eq!(result, Value::Bool(true));

    // Invalid URL
    let invalid_url = Value::Str("not-a-url".to_string());
    let result = url_is_valid(&invalid_url).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_url_encode_call() {
    let input = Value::Str("hello world".to_string());
    let result = url_encode(&input).unwrap();
    assert_eq!(result, Value::Str("hello%20world".to_string()));
}

#[test]
fn test_url_decode_call() {
    let input = Value::Str("hello%20world".to_string());
    let result = url_decode(&input).unwrap();

    // Should return Some("hello world")
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Str("hello world".to_string()));
        }
        _ => panic!("Expected Some variant"),
    }
}

#[test]
fn test_url_decode_invalid() {
    let input = Value::Str("invalid%ZZ".to_string());
    let result = url_decode(&input).unwrap();

    // Should return None
    match result {
        Value::Variant { variant_name, .. } => {
            assert_eq!(variant_name, "None");
        }
        _ => panic!("Expected None variant"),
    }
}
