// FSRS VM - Bytecode Virtual Machine Runtime
//
// This crate provides the VM runtime for executing F# scripts
// transpiled to bytecode via Fable's Rust backend.

pub mod value;

pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_crate_exports_value() {
        // Verify that Value is accessible from the crate root
        let val = Value::Int(42);
        assert_eq!(val.as_int(), Some(42));
    }
}
