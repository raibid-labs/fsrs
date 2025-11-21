// Fusabi VM - Bytecode Virtual Machine Runtime

pub mod chunk;
pub mod closure;
pub mod conversions;
pub mod gc;
pub mod host;
pub mod instruction;
pub mod stdlib;
pub mod value;
pub mod vm;

// Temporarily disabled - HOF tests incomplete
// #[cfg(test)]
// mod test_hof;

pub use chunk::{Chunk, ChunkBuilder};
pub use closure::{Closure, Upvalue};
pub use host::{HostFn, HostRegistry};
pub use instruction::Instruction;
pub use stdlib::StdlibRegistry;
pub use value::Value;
pub use vm::{Frame, Vm, VmError};
