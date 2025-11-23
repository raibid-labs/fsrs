// Fusabi VM - Bytecode Virtual Machine Runtime

pub mod chunk;
pub mod closure;
pub mod conversions;
pub mod host;
pub mod instruction;
pub mod stdlib;
pub mod value;
pub mod vm;

pub use chunk::{Chunk, ChunkBuilder};
pub use closure::{Closure, Upvalue};
pub use host::{HostFn, HostRegistry};
pub use instruction::Instruction;
pub use value::Value;
pub use vm::{Frame, Vm, VmError};

/// Magic bytes for Fusabi Bytecode files (.fzb)
pub const FZB_MAGIC: &[u8] = b"FZB\x01";
/// Version of the bytecode format
pub const FZB_VERSION: u8 = 1;

#[cfg(feature = "serde")]
/// Serialize a chunk to bytes with magic header
pub fn serialize_chunk(chunk: &Chunk) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut bytes = Vec::new();
    
    // Write magic bytes
    bytes.extend_from_slice(FZB_MAGIC);
    
    // Write version
    bytes.push(FZB_VERSION);
    
    // Serialize chunk with bincode
    let chunk_bytes = bincode::serialize(chunk)?;
    bytes.extend_from_slice(&chunk_bytes);
    
    Ok(bytes)
}

#[cfg(feature = "serde")]
/// Deserialize a chunk from bytes, checking magic header
pub fn deserialize_chunk(bytes: &[u8]) -> Result<Chunk, Box<dyn std::error::Error + Send + Sync + 'static>> {
    if bytes.len() < 5 {
        return Err("File too short".into());
    }

    // Check magic bytes
    if &bytes[0..4] != FZB_MAGIC {
        return Err("Invalid magic bytes".into());
    }
    
    // Check version
    if bytes[4] != FZB_VERSION {
        return Err(format!("Unsupported version: {}", bytes[4]).into());
    }
    
    // Deserialize chunk
    let chunk: Chunk = bincode::deserialize(&bytes[5..])?;
    Ok(chunk)
}
