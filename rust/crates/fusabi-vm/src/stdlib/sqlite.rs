// Fusabi SQLite Standard Library
// Provides SQLite database access functions

#[cfg(feature = "sqlite")]
use crate::value::Value;
#[cfg(feature = "sqlite")]
use crate::vm::VmError;
#[cfg(feature = "sqlite")]
use rusqlite::Connection;
#[cfg(feature = "sqlite")]
use std::collections::HashMap;
#[cfg(feature = "sqlite")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "sqlite")]
lazy_static::lazy_static! {
    static ref CONNECTIONS: Mutex<HashMap<i64, Connection>> = Mutex::new(HashMap::new());
    static ref NEXT_CONN_ID: Mutex<i64> = Mutex::new(1);
}

#[cfg(feature = "sqlite")]
/// Sqlite.open : string -> int
/// Opens a database file and returns a connection handle
pub fn sqlite_open(path: &Value) -> Result<Value, VmError> {
    let path_str = match path {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: path.type_name(),
            })
        }
    };

    let conn = Connection::open(path_str)
        .map_err(|e| VmError::Runtime(format!("Failed to open database '{}': {}", path_str, e)))?;

    let mut next_id = NEXT_CONN_ID.lock().unwrap();
    let conn_id = *next_id;
    *next_id += 1;

    CONNECTIONS.lock().unwrap().insert(conn_id, conn);

    Ok(Value::Int(conn_id))
}

#[cfg(feature = "sqlite")]
/// Sqlite.execute : int -> string -> unit
/// Executes a SQL statement that doesn't return rows
pub fn sqlite_execute(conn_id: &Value, sql: &Value) -> Result<Value, VmError> {
    let id = match conn_id {
        Value::Int(i) => *i,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: conn_id.type_name(),
            })
        }
    };

    let sql_str = match sql {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: sql.type_name(),
            })
        }
    };

    let conns = CONNECTIONS.lock().unwrap();
    let conn = conns
        .get(&id)
        .ok_or_else(|| VmError::Runtime(format!("Invalid connection handle: {}", id)))?;

    conn.execute(sql_str, [])
        .map_err(|e| VmError::Runtime(format!("SQL execute error: {}", e)))?;

    Ok(Value::Unit)
}

#[cfg(feature = "sqlite")]
/// Sqlite.query : int -> string -> list<map>
/// Executes a SQL query and returns rows as a list of maps
pub fn sqlite_query(conn_id: &Value, sql: &Value) -> Result<Value, VmError> {
    let id = match conn_id {
        Value::Int(i) => *i,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: conn_id.type_name(),
            })
        }
    };

    let sql_str = match sql {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: sql.type_name(),
            })
        }
    };

    let conns = CONNECTIONS.lock().unwrap();
    let conn = conns
        .get(&id)
        .ok_or_else(|| VmError::Runtime(format!("Invalid connection handle: {}", id)))?;

    let mut stmt = conn
        .prepare(sql_str)
        .map_err(|e| VmError::Runtime(format!("SQL prepare error: {}", e)))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let rows_result: Result<Vec<Value>, VmError> = stmt
        .query_map([], |row| {
            let mut fields = HashMap::new();
            for i in 0..column_count {
                let col_name = column_names[i].clone();
                let value = row_value_to_fusabi(row, i);
                fields.insert(col_name, value);
            }
            Ok(Value::Record(Arc::new(Mutex::new(fields))))
        })
        .map_err(|e| VmError::Runtime(format!("SQL query error: {}", e)))?
        .map(|r| r.map_err(|e| VmError::Runtime(format!("Row error: {}", e))))
        .collect();

    let rows = rows_result?;

    let mut result = Value::Nil;
    for row in rows.into_iter().rev() {
        result = Value::Cons {
            head: Box::new(row),
            tail: Box::new(result),
        };
    }

    Ok(result)
}

#[cfg(feature = "sqlite")]
fn row_value_to_fusabi(row: &rusqlite::Row, idx: usize) -> Value {
    use rusqlite::types::ValueRef;

    match row.get_ref(idx) {
        Ok(ValueRef::Null) => Value::Unit,
        Ok(ValueRef::Integer(i)) => Value::Int(i),
        Ok(ValueRef::Real(f)) => Value::Float(f),
        Ok(ValueRef::Text(s)) => Value::Str(String::from_utf8_lossy(s).to_string()),
        Ok(ValueRef::Blob(b)) => Value::Str(format!("<blob {} bytes>", b.len())),
        Err(_) => Value::Unit,
    }
}

#[cfg(feature = "sqlite")]
/// Sqlite.close : int -> unit
/// Closes a database connection
pub fn sqlite_close(conn_id: &Value) -> Result<Value, VmError> {
    let id = match conn_id {
        Value::Int(i) => *i,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: conn_id.type_name(),
            })
        }
    };

    let removed = CONNECTIONS.lock().unwrap().remove(&id);

    if removed.is_none() {
        return Err(VmError::Runtime(format!(
            "Invalid connection handle: {}",
            id
        )));
    }

    Ok(Value::Unit)
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use super::*;

    #[test]
    fn test_open_close() {
        let conn = sqlite_open(&Value::Str(":memory:".to_string())).unwrap();
        match conn {
            Value::Int(id) => {
                assert!(id > 0);
                let result = sqlite_close(&Value::Int(id)).unwrap();
                assert_eq!(result, Value::Unit);
            }
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_execute_and_query() {
        let conn = sqlite_open(&Value::Str(":memory:".to_string())).unwrap();
        let id = match conn {
            Value::Int(i) => i,
            _ => panic!("Expected Int"),
        };

        sqlite_execute(
            &Value::Int(id),
            &Value::Str("CREATE TABLE test (id INTEGER, name TEXT)".to_string()),
        )
        .unwrap();

        sqlite_execute(
            &Value::Int(id),
            &Value::Str("INSERT INTO test VALUES (1, 'Alice')".to_string()),
        )
        .unwrap();

        sqlite_execute(
            &Value::Int(id),
            &Value::Str("INSERT INTO test VALUES (2, 'Bob')".to_string()),
        )
        .unwrap();

        let result = sqlite_query(
            &Value::Int(id),
            &Value::Str("SELECT * FROM test ORDER BY id".to_string()),
        )
        .unwrap();

        let mut rows = Vec::new();
        let mut current = result;
        loop {
            match current {
                Value::Nil => break,
                Value::Cons { head, tail } => {
                    rows.push(*head);
                    current = *tail;
                }
                _ => panic!("Expected list"),
            }
        }

        assert_eq!(rows.len(), 2);

        sqlite_close(&Value::Int(id)).unwrap();
    }

    #[test]
    fn test_invalid_connection() {
        let result = sqlite_execute(&Value::Int(99999), &Value::Str("SELECT 1".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_type_errors() {
        let result = sqlite_open(&Value::Int(42));
        assert!(result.is_err());

        let result = sqlite_execute(
            &Value::Str("not an id".to_string()),
            &Value::Str("SELECT 1".to_string()),
        );
        assert!(result.is_err());
    }
}
