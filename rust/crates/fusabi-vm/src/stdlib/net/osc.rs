// Fusabi Net Module
// Provides networking capabilities including OSC and HTTP

// OSC (Open Sound Control) client implementation for Fusabi
// Enables communication with OSC-compatible applications like Ardour

#[cfg(feature = "osc")]
use crate::value::{HostData, Value};
#[cfg(feature = "osc")]
use crate::vm::VmError;
#[cfg(feature = "osc")]
use rosc::{OscMessage, OscPacket, OscType};
#[cfg(feature = "osc")]
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[cfg(feature = "osc")]
/// OSC Client that manages UDP socket connection
pub struct OscClient {
    socket: UdpSocket,
    target: SocketAddr,
}

#[cfg(feature = "osc")]
impl OscClient {
    /// Create a new OSC client connected to the specified host and port
    pub fn new(host: &str, port: u16) -> Result<Self, String> {
        // Bind to any available local port
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        // Resolve hostname to socket address
        let addr_str = format!("{}:{}", host, port);
        let target = addr_str
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?
            .next()
            .ok_or_else(|| format!("No address found for host '{}'", host))?;

        Ok(Self { socket, target })
    }

    /// Send an OSC message to the target
    pub fn send(&self, address: &str, args: Vec<OscType>) -> Result<(), String> {
        let msg = OscMessage {
            addr: address.to_string(),
            args,
        };

        let packet = OscPacket::Message(msg);
        let bytes = rosc::encoder::encode(&packet)
            .map_err(|e| format!("Failed to encode OSC message: {}", e))?;

        self.socket
            .send_to(&bytes, self.target)
            .map_err(|e| format!("Failed to send OSC message: {}", e))?;

        Ok(())
    }
}

#[cfg(feature = "osc")]
fn value_to_osc_type(v: &Value) -> Result<OscType, VmError> {
    match v {
        Value::Int(i) => Ok(OscType::Int(*i as i32)),
        Value::Float(f) => Ok(OscType::Float(*f as f32)),
        Value::Str(s) => Ok(OscType::String(s.clone())),
        Value::Bool(b) => Ok(OscType::Bool(*b)),
        Value::Unit => Ok(OscType::Nil),
        _ => Err(VmError::Runtime(format!(
            "Cannot convert {} to OSC type. Supported types: int, float, string, bool, unit",
            v.type_name()
        ))),
    }
}

#[cfg(feature = "osc")]
/// Osc.client : string -> int -> OscClient
/// Create a new OSC client connected to the specified host and port
/// Example: Osc.client "localhost" 3819
pub fn osc_client(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Osc.client expects 2 arguments (host, port), got {}",
            args.len()
        )));
    }

    let host = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: args[0].type_name(),
            })
        }
    };

    let port = match &args[1] {
        Value::Int(i) => {
            if *i < 0 || *i > 65535 {
                return Err(VmError::Runtime(format!(
                    "Port must be between 0 and 65535, got {}",
                    i
                )));
            }
            *i as u16
        }
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: args[1].type_name(),
            })
        }
    };

    let client = OscClient::new(&host, port)
        .map_err(|e| VmError::Runtime(format!("Failed to create OSC client: {}", e)))?;

    Ok(Value::HostData(HostData::new(client, "OscClient")))
}

#[cfg(feature = "osc")]
/// Osc.send : OscClient -> string -> list<obj> -> unit
/// Send an OSC message with arguments to the specified address
/// Example: client |> Osc.send "/transport_play" [1; "test"]
pub fn osc_send(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::Runtime(format!(
            "Osc.send expects 3 arguments (client, address, args_list), got {}",
            args.len()
        )));
    }

    let client_data = match &args[0] {
        Value::HostData(hd) => hd,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "OscClient",
                got: args[0].type_name(),
            })
        }
    };

    let client = client_data
        .try_borrow::<OscClient>()
        .ok_or_else(|| VmError::Runtime("Invalid OscClient object".to_string()))?;

    let address = match &args[1] {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: args[1].type_name(),
            })
        }
    };

    let args_list = &args[2];
    let mut osc_args = Vec::new();

    // Iterate over the list
    let mut current = args_list;
    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                osc_args.push(value_to_osc_type(head)?);
                current = tail;
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "list",
                    got: current.type_name(),
                })
            }
        }
    }

    client
        .send(&address, osc_args)
        .map_err(|e| VmError::Runtime(format!("OSC send failed: {}", e)))?;

    Ok(Value::Unit)
}

#[cfg(feature = "osc")]
/// Osc.sendInt : OscClient -> string -> int -> unit
/// Deprecated: Use Osc.send instead
pub fn osc_send_int(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    // Implementation kept for reference but not registered
    Err(VmError::Runtime(
        "Osc.sendInt is deprecated, use Osc.send".to_string(),
    ))
}

#[cfg(feature = "osc")]
/// Osc.sendFloat : OscClient -> string -> float -> unit
/// Deprecated: Use Osc.send instead
pub fn osc_send_float(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    // Implementation kept for reference but not registered
    Err(VmError::Runtime(
        "Osc.sendFloat is deprecated, use Osc.send".to_string(),
    ))
}

#[cfg(feature = "osc")]
/// Osc.sendString : OscClient -> string -> string -> unit
/// Deprecated: Use Osc.send instead
pub fn osc_send_string(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    // Implementation kept for reference but not registered
    Err(VmError::Runtime(
        "Osc.sendString is deprecated, use Osc.send".to_string(),
    ))
}

#[cfg(test)]
#[cfg(feature = "osc")]
mod tests {
    use super::*;

    #[test]
    fn test_osc_client_creation() {
        let mut vm = crate::vm::Vm::new();
        let args = vec![Value::Str("localhost".to_string()), Value::Int(3819)];
        let result = osc_client(&mut vm, &args);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::HostData(hd) => {
                assert_eq!(hd.type_name(), "OscClient");
            }
            _ => panic!("Expected HostData"),
        }
    }

    #[test]
    fn test_osc_send_type_checking() {
        let mut vm = crate::vm::Vm::new();

        // Create a mock client first
        let client_args = vec![Value::Str("localhost".to_string()), Value::Int(3819)];
        let client = osc_client(&mut vm, &client_args).unwrap();

        // Test send with wrong type
        let result = osc_send(
            &mut vm,
            &[Value::Int(42), Value::Str("/test".to_string()), Value::Nil],
        );
        assert!(result.is_err());

        // Test send with correct types (empty list)
        let result = osc_send(
            &mut vm,
            &[client.clone(), Value::Str("/test".to_string()), Value::Nil],
        );
        // May succeed or fail depending on network availability, but should not panic
        let _ = result;
    }
}
