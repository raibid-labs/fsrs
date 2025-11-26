// Simple test to demonstrate the bytecode API
use fusabi::{compile_to_bytecode, execute_bytecode};
use fusabi_vm::Vm;

fn main() {
    // Test 1: Compile and execute a simple expression
    println!("Test 1: Simple arithmetic");
    let source = "let x = 42 in x * 2";
    let bytecode = compile_to_bytecode(source).expect("Compilation failed");
    let result = execute_bytecode(&bytecode).expect("Execution failed");
    println!("Result: {:?}", result.as_int());
    assert_eq!(result.as_int(), Some(84));

    // Test 2: Use VM::from_bytecode
    println!("\nTest 2: VM from bytecode");
    let source2 = "100 + 23";
    let bytecode2 = compile_to_bytecode(source2).expect("Compilation failed");
    let mut vm = Vm::from_bytecode(&bytecode2).expect("VM creation failed");
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result2 = vm.run().expect("Execution failed");
    println!("Result: {:?}", result2.as_int());
    assert_eq!(result2.as_int(), Some(123));

    // Test 3: Use Vm::execute_bytecode
    println!("\nTest 3: VM::execute_bytecode");
    let source3 = "50 - 8";
    let bytecode3 = compile_to_bytecode(source3).expect("Compilation failed");
    let result3 = Vm::execute_bytecode(&bytecode3).expect("Execution failed");
    println!("Result: {:?}", result3.as_int());
    assert_eq!(result3.as_int(), Some(42));

    println!("\nAll tests passed!");
}
