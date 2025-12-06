// Fusabi Config Standard Library
// Provides configuration management with schemas and validation

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ConfigEntry = (ConfigSchema, Option<Value>);
type ConfigRegistryType = Arc<Mutex<HashMap<String, ConfigEntry>>>;

lazy_static::lazy_static! {
    /// Global registry for configuration schemas and values
    /// Key: config name, Value: (schema, current value)
    static ref CONFIG_REGISTRY: ConfigRegistryType =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Internal representation of a configuration schema
#[derive(Clone)]
struct ConfigSchema {
    name: String,
    config_type: String,
    default_value: Option<Value>,
    validator: Option<Value>,
}

impl ConfigSchema {
    /// Parse a ConfigSchema record from a Value
    fn from_value(value: &Value) -> Result<Self, VmError> {
        let record = value.as_record().ok_or_else(|| VmError::TypeMismatch {
            expected: "record",
            got: value.type_name(),
        })?;

        let fields = record.lock().unwrap();

        let name = fields
            .get("name")
            .ok_or_else(|| VmError::Runtime("ConfigSchema missing 'name' field".to_string()))?
            .as_str()
            .ok_or(VmError::TypeMismatch {
                expected: "string",
                got: "other",
            })?
            .to_string();

        let config_type = fields
            .get("configType")
            .ok_or_else(|| VmError::Runtime("ConfigSchema missing 'configType' field".to_string()))?
            .as_str()
            .ok_or(VmError::TypeMismatch {
                expected: "string",
                got: "other",
            })?
            .to_string();

        // Validate configType
        match config_type.as_str() {
            "string" | "int" | "bool" | "list" | "map" => {}
            _ => {
                return Err(VmError::Runtime(format!(
                    "Invalid configType '{}'. Must be one of: string, int, bool, list, map",
                    config_type
                )))
            }
        }

        // Extract default value (Option)
        let default_value = fields.get("default").and_then(|v| match v {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => fields.first().cloned(),
            Value::Variant { variant_name, .. } if variant_name == "None" => None,
            _ => None,
        });

        // Extract validator function (Option)
        let validator = fields.get("validator").and_then(|v| match v {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => fields.first().cloned(),
            Value::Variant { variant_name, .. } if variant_name == "None" => None,
            _ => None,
        });

        Ok(ConfigSchema {
            name,
            config_type,
            default_value,
            validator,
        })
    }

    /// Validate that a ConfigValue matches the expected type
    fn validate_type(&self, config_value: &Value) -> Result<(), VmError> {
        // Extract the variant name and fields from ConfigValue
        let (type_name, variant_name, fields) =
            config_value
                .as_variant()
                .ok_or_else(|| VmError::TypeMismatch {
                    expected: "ConfigValue variant",
                    got: config_value.type_name(),
                })?;

        if type_name != "ConfigValue" {
            return Err(VmError::Runtime(format!(
                "Expected ConfigValue variant, got {}",
                type_name
            )));
        }

        // Validate the variant matches the schema's configType
        match (self.config_type.as_str(), variant_name) {
            ("string", "String") => {
                if fields.is_empty() || !matches!(fields[0], Value::Str(_)) {
                    return Err(VmError::Runtime(
                        "ConfigValue.String expects a string value".to_string(),
                    ));
                }
            }
            ("int", "Int") => {
                if fields.is_empty() || !matches!(fields[0], Value::Int(_)) {
                    return Err(VmError::Runtime(
                        "ConfigValue.Int expects an int value".to_string(),
                    ));
                }
            }
            ("bool", "Bool") => {
                if fields.is_empty() || !matches!(fields[0], Value::Bool(_)) {
                    return Err(VmError::Runtime(
                        "ConfigValue.Bool expects a bool value".to_string(),
                    ));
                }
            }
            ("list", "List") => {
                if fields.is_empty() || (!fields[0].is_cons() && !fields[0].is_nil()) {
                    return Err(VmError::Runtime(
                        "ConfigValue.List expects a list value".to_string(),
                    ));
                }
            }
            ("map", "Map") => {
                if fields.is_empty() || !matches!(fields[0], Value::Map(_)) {
                    return Err(VmError::Runtime(
                        "ConfigValue.Map expects a map value".to_string(),
                    ));
                }
            }
            (expected, got) => {
                return Err(VmError::Runtime(format!(
                    "Type mismatch: expected ConfigValue.{} for configType '{}', got ConfigValue.{}",
                    expected, expected, got
                )));
            }
        }

        Ok(())
    }
}

/// Config.define : ConfigSchema -> unit
/// Register a configuration schema
pub fn config_define(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Config.define expects 1 argument, got {}",
            args.len()
        )));
    }

    let schema = ConfigSchema::from_value(&args[0])?;
    let name = schema.name.clone();
    let default_value = schema.default_value.clone();

    // Validate default value if provided
    if let Some(ref value) = default_value {
        schema.validate_type(value)?;

        // Run validator function if provided
        if let Some(ref validator) = schema.validator {
            let result = vm.call_value(validator.clone(), std::slice::from_ref(value))?;
            match result {
                Value::Bool(true) => {}
                Value::Bool(false) => {
                    return Err(VmError::Runtime(format!(
                        "Default value validation failed for config '{}'",
                        name
                    )));
                }
                _ => {
                    return Err(VmError::Runtime(
                        "Validator function must return bool".to_string(),
                    ));
                }
            }
        }
    }

    // Register schema with default value
    let mut registry = CONFIG_REGISTRY.lock().unwrap();
    registry.insert(name.clone(), (schema, default_value));

    Ok(Value::Unit)
}

/// Config.get : string -> ConfigValue
/// Get a configuration value (throws if not found)
pub fn config_get(name: &Value) -> Result<Value, VmError> {
    let name_str = name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: name.type_name(),
    })?;

    let registry = CONFIG_REGISTRY.lock().unwrap();
    let (_, value) = registry
        .get(name_str)
        .ok_or_else(|| VmError::Runtime(format!("Configuration '{}' not found", name_str)))?;

    value.clone().ok_or_else(|| {
        VmError::Runtime(format!(
            "Configuration '{}' has no value set and no default",
            name_str
        ))
    })
}

/// Config.getOr : string -> ConfigValue -> ConfigValue
/// Get a configuration value with a fallback default
pub fn config_get_or(name: &Value, default: &Value) -> Result<Value, VmError> {
    let name_str = name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: name.type_name(),
    })?;

    let registry = CONFIG_REGISTRY.lock().unwrap();

    match registry.get(name_str) {
        Some((_, Some(value))) => Ok(value.clone()),
        Some((_, None)) => Ok(default.clone()),
        None => Ok(default.clone()),
    }
}

/// Config.set : string -> ConfigValue -> unit
/// Set a configuration value (validates against schema)
pub fn config_set(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Config.set expects 2 arguments, got {}",
            args.len()
        )));
    }

    let name_str = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let value = &args[1];

    let mut registry = CONFIG_REGISTRY.lock().unwrap();
    let (schema, current_value) = registry
        .get_mut(name_str)
        .ok_or_else(|| VmError::Runtime(format!("Configuration '{}' not defined", name_str)))?;

    // Validate type
    schema.validate_type(value)?;

    // Run validator function if provided
    if let Some(ref validator) = schema.validator {
        let result = vm.call_value(validator.clone(), std::slice::from_ref(value))?;
        match result {
            Value::Bool(true) => {}
            Value::Bool(false) => {
                return Err(VmError::Runtime(format!(
                    "Value validation failed for config '{}'",
                    name_str
                )));
            }
            _ => {
                return Err(VmError::Runtime(
                    "Validator function must return bool".to_string(),
                ));
            }
        }
    }

    *current_value = Some(value.clone());
    Ok(Value::Unit)
}

/// Config.has : string -> bool
/// Check if a configuration is defined
pub fn config_has(name: &Value) -> Result<Value, VmError> {
    let name_str = name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: name.type_name(),
    })?;

    let registry = CONFIG_REGISTRY.lock().unwrap();
    Ok(Value::Bool(registry.contains_key(name_str)))
}

/// Config.list : unit -> (string * ConfigValue) list
/// List all defined configurations with their current values
pub fn config_list(_unit: &Value) -> Result<Value, VmError> {
    let registry = CONFIG_REGISTRY.lock().unwrap();
    let mut entries = Vec::new();

    for (name, (_, value)) in registry.iter() {
        if let Some(val) = value {
            let tuple = Value::Tuple(vec![Value::Str(name.clone()), val.clone()]);
            entries.push(tuple);
        }
    }

    // Sort by name for deterministic output
    entries.sort_by(|a, b| {
        let name_a = a.as_tuple().and_then(|t| t[0].as_str()).unwrap_or("");
        let name_b = b.as_tuple().and_then(|t| t[0].as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    Ok(Value::vec_to_cons(entries))
}

/// Config.reset : string -> unit
/// Reset a configuration to its default value
pub fn config_reset(name: &Value) -> Result<Value, VmError> {
    let name_str = name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: name.type_name(),
    })?;

    let mut registry = CONFIG_REGISTRY.lock().unwrap();
    let (schema, current_value) = registry
        .get_mut(name_str)
        .ok_or_else(|| VmError::Runtime(format!("Configuration '{}' not defined", name_str)))?;

    *current_value = schema.default_value.clone();
    Ok(Value::Unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_config_schema(
        name: &str,
        config_type: &str,
        default: Option<Value>,
        validator: Option<Value>,
    ) -> Value {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::Str(name.to_string()));
        fields.insert(
            "configType".to_string(),
            Value::Str(config_type.to_string()),
        );

        let default_option = match default {
            Some(val) => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![val],
            },
            None => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            },
        };
        fields.insert("default".to_string(), default_option);

        let validator_option = match validator {
            Some(val) => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![val],
            },
            None => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            },
        };
        fields.insert("validator".to_string(), validator_option);

        Value::Record(Arc::new(Mutex::new(fields)))
    }

    fn create_config_value_string(s: &str) -> Value {
        Value::Variant {
            type_name: "ConfigValue".to_string(),
            variant_name: "String".to_string(),
            fields: vec![Value::Str(s.to_string())],
        }
    }

    fn create_config_value_int(n: i64) -> Value {
        Value::Variant {
            type_name: "ConfigValue".to_string(),
            variant_name: "Int".to_string(),
            fields: vec![Value::Int(n)],
        }
    }

    fn create_config_value_bool(b: bool) -> Value {
        Value::Variant {
            type_name: "ConfigValue".to_string(),
            variant_name: "Bool".to_string(),
            fields: vec![Value::Bool(b)],
        }
    }

    fn clear_registry() {
        CONFIG_REGISTRY.lock().unwrap().clear();
    }

    #[test]
    fn test_config_define_simple() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema(
            "app.name",
            "string",
            Some(create_config_value_string("MyApp")),
            None,
        );

        let result = config_define(&mut vm, &[schema]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);

        // Verify it was registered
        let registry = CONFIG_REGISTRY.lock().unwrap();
        assert!(registry.contains_key("app.name"));
    }

    #[test]
    fn test_config_define_invalid_type() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema("test", "invalid_type", None, None);

        let result = config_define(&mut vm, &[schema]);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_get_success() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema("port", "int", Some(create_config_value_int(8080)), None);
        config_define(&mut vm, &[schema]).unwrap();

        let result = config_get(&Value::Str("port".to_string()));
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.variant_name().unwrap(), "Int");
    }

    #[test]
    fn test_config_get_not_found() {
        clear_registry();

        let result = config_get(&Value::Str("nonexistent".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_get_no_value_no_default() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema("test", "string", None, None);
        config_define(&mut vm, &[schema]).unwrap();

        let result = config_get(&Value::Str("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_get_or_with_value() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema(
            "name",
            "string",
            Some(create_config_value_string("default")),
            None,
        );
        config_define(&mut vm, &[schema]).unwrap();

        let fallback = create_config_value_string("fallback");
        let result = config_get_or(&Value::Str("name".to_string()), &fallback);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.variant_name().unwrap(), "String");
    }

    #[test]
    fn test_config_get_or_without_value() {
        clear_registry();

        let fallback = create_config_value_string("fallback");
        let result = config_get_or(&Value::Str("nonexistent".to_string()), &fallback);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), fallback);
    }

    #[test]
    fn test_config_has_true() {
        clear_registry();
        let mut vm = Vm::new();

        let schema = create_config_schema("test", "bool", None, None);
        config_define(&mut vm, &[schema]).unwrap();

        let result = config_has(&Value::Str("test".to_string()));
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_config_has_false() {
        clear_registry();

        let result = config_has(&Value::Str("nonexistent".to_string()));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_config_list_empty() {
        clear_registry();

        let result = config_list(&Value::Unit);
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_config_list_with_values() {
        clear_registry();
        let mut vm = Vm::new();

        let schema1 = create_config_schema("a", "int", Some(create_config_value_int(1)), None);
        let schema2 = create_config_schema("b", "int", Some(create_config_value_int(2)), None);
        config_define(&mut vm, &[schema1]).unwrap();
        config_define(&mut vm, &[schema2]).unwrap();

        let result = config_list(&Value::Unit);
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list.is_cons());
    }

    #[test]
    fn test_config_reset() {
        clear_registry();
        let mut vm = Vm::new();
        crate::stdlib::register_stdlib(&mut vm);

        let schema = create_config_schema("test", "int", Some(create_config_value_int(100)), None);
        config_define(&mut vm, &[schema]).unwrap();

        // Set a new value
        config_set(
            &mut vm,
            &[Value::Str("test".to_string()), create_config_value_int(200)],
        )
        .unwrap();

        // Verify new value
        let value = config_get(&Value::Str("test".to_string())).unwrap();
        if let Value::Variant { fields, .. } = value {
            assert_eq!(fields[0], Value::Int(200));
        }

        // Reset to default
        config_reset(&Value::Str("test".to_string())).unwrap();

        // Verify default value
        let value = config_get(&Value::Str("test".to_string())).unwrap();
        if let Value::Variant { fields, .. } = value {
            assert_eq!(fields[0], Value::Int(100));
        }
    }

    #[test]
    fn test_config_type_validation_string() {
        clear_registry();
        let mut vm = Vm::new();
        crate::stdlib::register_stdlib(&mut vm);

        let schema = create_config_schema("test", "string", None, None);
        config_define(&mut vm, &[schema]).unwrap();

        // Valid string
        let result = config_set(
            &mut vm,
            &[
                Value::Str("test".to_string()),
                create_config_value_string("hello"),
            ],
        );
        assert!(result.is_ok());

        // Invalid type (int instead of string)
        let result = config_set(
            &mut vm,
            &[Value::Str("test".to_string()), create_config_value_int(42)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_config_type_validation_bool() {
        clear_registry();
        let mut vm = Vm::new();
        crate::stdlib::register_stdlib(&mut vm);

        let schema = create_config_schema("enabled", "bool", None, None);
        config_define(&mut vm, &[schema]).unwrap();

        // Valid bool
        let result = config_set(
            &mut vm,
            &[
                Value::Str("enabled".to_string()),
                create_config_value_bool(true),
            ],
        );
        assert!(result.is_ok());

        // Invalid type
        let result = config_set(
            &mut vm,
            &[
                Value::Str("enabled".to_string()),
                create_config_value_int(1),
            ],
        );
        assert!(result.is_err());
    }
}
