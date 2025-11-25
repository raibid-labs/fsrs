#!/bin/bash
# Script to convert Rc<RefCell<T>> to Arc<Mutex<T>> in Fusabi

# Function to replace in a file
replace_in_file() {
    local file=$1
    echo "Processing $file..."

    # Replace imports
    sed -i 's/use std::rc::Rc;/use std::sync::Arc;/g' "$file"
    sed -i 's/use std::cell::RefCell;/use std::sync::Mutex;/g' "$file"

    # Add Arc and Mutex imports if missing
    if ! grep -q "use std::sync::{Arc, Mutex}" "$file"; then
        sed -i '/use std::collections::HashMap;/a\use std::sync::{Arc, Mutex};' "$file"
    fi

    # Replace types
    sed -i 's/Rc<RefCell</Arc<Mutex</g' "$file"

    # Replace method calls - be selective to avoid breaking things
    sed -i 's/\.borrow()/.lock().unwrap()/g' "$file"
    sed -i 's/\.borrow_mut()/.lock().unwrap()/g' "$file"

    # Fix comments
    sed -i 's/Rc<RefCell<Vec<Value>>>/Arc<Mutex<Vec<Value>>>/g' "$file"
    sed -i 's/Rc<RefCell<HashMap<String, Value>>>/Arc<Mutex<HashMap<String, Value>>>/g' "$file"

    # Fix Rc:: to Arc::
    sed -i 's/Rc::/Arc::/g' "$file"

    # Fix clone_rc to clone_arc
    sed -i 's/clone_rc/clone_arc/g' "$file"

    echo "Done with $file"
}

# Process the files
replace_in_file "crates/fusabi-vm/src/value.rs"
replace_in_file "crates/fusabi-vm/src/closure.rs"
replace_in_file "crates/fusabi-vm/src/vm.rs"
replace_in_file "crates/fusabi/src/host_api.rs"

echo "All files processed!"
