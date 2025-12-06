// Fusabi Url Standard Library
// Provides URL parsing and encoding/decoding functionality

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Url.parse : string -> UrlInfo option
/// Parses a URL string into its components
/// Returns None if the URL is invalid
pub fn url_parse(url_str: &Value) -> Result<Value, VmError> {
    let url = match url_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: url_str.type_name(),
            })
        }
    };

    match parse_url_internal(url) {
        Some(url_info) => {
            // Return Some(UrlInfo)
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![url_info],
            })
        }
        None => {
            // Return None
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            })
        }
    }
}

/// Url.isValid : string -> bool
/// Check if a string is a valid URL
pub fn url_is_valid(url_str: &Value) -> Result<Value, VmError> {
    let url = match url_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: url_str.type_name(),
            })
        }
    };

    Ok(Value::Bool(parse_url_internal(url).is_some()))
}

/// Url.encode : string -> string
/// URL-encode a string (percent encoding)
pub fn url_encode(s: &Value) -> Result<Value, VmError> {
    let string = match s {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: s.type_name(),
            })
        }
    };

    let encoded = percent_encode(string);
    Ok(Value::Str(encoded))
}

/// Url.decode : string -> string option
/// URL-decode a string (percent decoding)
/// Returns None if the string contains invalid percent encoding
pub fn url_decode(s: &Value) -> Result<Value, VmError> {
    let string = match s {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: s.type_name(),
            })
        }
    };

    match percent_decode(string) {
        Some(decoded) => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Str(decoded)],
        }),
        None => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        }),
    }
}

// Internal helper functions

/// UrlInfo record type
/// Contains: scheme, host, port (option), path, query (option), fragment (option)
fn create_url_info(
    scheme: String,
    host: String,
    port: Option<i64>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
) -> Value {
    let mut fields = HashMap::new();

    fields.insert("scheme".to_string(), Value::Str(scheme));
    fields.insert("host".to_string(), Value::Str(host));

    // port: int option
    let port_value = match port {
        Some(p) => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Int(p)],
        },
        None => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        },
    };
    fields.insert("port".to_string(), port_value);

    fields.insert("path".to_string(), Value::Str(path));

    // query: string option
    let query_value = match query {
        Some(q) => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Str(q)],
        },
        None => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        },
    };
    fields.insert("query".to_string(), query_value);

    // fragment: string option
    let fragment_value = match fragment {
        Some(f) => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Str(f)],
        },
        None => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        },
    };
    fields.insert("fragment".to_string(), fragment_value);

    Value::Record(Arc::new(Mutex::new(fields)))
}

/// Parse URL string into components
/// Returns None if URL is invalid
fn parse_url_internal(url: &str) -> Option<Value> {
    // Basic URL parsing: scheme://host[:port][/path][?query][#fragment]

    // Empty URL is invalid
    if url.is_empty() {
        return None;
    }

    let mut remaining = url;

    // 1. Parse scheme
    let scheme_end = remaining.find("://")?;
    let scheme = remaining[..scheme_end].to_string();

    // Validate scheme (must be alphanumeric with +, -, .)
    if !is_valid_scheme(&scheme) {
        return None;
    }

    remaining = &remaining[scheme_end + 3..];

    // 2. Extract fragment first (if present)
    let (remaining, fragment) = if let Some(hash_pos) = remaining.find('#') {
        let frag = remaining[hash_pos + 1..].to_string();
        (&remaining[..hash_pos], Some(frag))
    } else {
        (remaining, None)
    };

    // 3. Extract query (if present)
    let (remaining, query) = if let Some(question_pos) = remaining.find('?') {
        let q = remaining[question_pos + 1..].to_string();
        (&remaining[..question_pos], Some(q))
    } else {
        (remaining, None)
    };

    // 4. Extract path (if present)
    let (authority, path) = if let Some(slash_pos) = remaining.find('/') {
        let p = remaining[slash_pos..].to_string();
        (&remaining[..slash_pos], p)
    } else {
        (remaining, "/".to_string())
    };

    // 5. Parse authority (host[:port])
    let (host, port) = if let Some(colon_pos) = authority.rfind(':') {
        // Check if this colon is for port (not part of IPv6 address)
        // Simple heuristic: if there's a [ before ], it's IPv6
        let has_ipv6_bracket = authority.contains('[');

        if has_ipv6_bracket {
            // IPv6 address - only parse port if colon comes after ]
            if let Some(bracket_pos) = authority.rfind(']') {
                if colon_pos > bracket_pos {
                    // Port after IPv6 address
                    let h = authority[..colon_pos].to_string();
                    let port_str = &authority[colon_pos + 1..];
                    match port_str.parse::<i64>() {
                        Ok(p) if p > 0 && p <= 65535 => (h, Some(p)),
                        _ => return None, // Invalid port
                    }
                } else {
                    // Colon is part of IPv6 address
                    (authority.to_string(), None)
                }
            } else {
                // Malformed IPv6
                return None;
            }
        } else {
            // Regular host with port
            let h = authority[..colon_pos].to_string();
            let port_str = &authority[colon_pos + 1..];
            match port_str.parse::<i64>() {
                Ok(p) if p > 0 && p <= 65535 => (h, Some(p)),
                _ => return None, // Invalid port
            }
        }
    } else {
        (authority.to_string(), None)
    };

    // Validate host is not empty
    if host.is_empty() {
        return None;
    }

    Some(create_url_info(scheme, host, port, path, query, fragment))
}

/// Check if scheme is valid (alphanumeric with +, -, .)
fn is_valid_scheme(scheme: &str) -> bool {
    if scheme.is_empty() {
        return false;
    }

    // First character must be alphabetic
    let mut chars = scheme.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() {
            return false;
        }
    } else {
        return false;
    }

    // Remaining characters can be alphanumeric or +, -, .
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '+' && c != '-' && c != '.' {
            return false;
        }
    }

    true
}

/// Percent-encode a string for use in URLs
/// Encodes all characters except unreserved: A-Z a-z 0-9 - _ . ~
fn percent_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);

    for byte in s.bytes() {
        match byte {
            // Unreserved characters (RFC 3986)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            // Everything else gets percent-encoded
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }

    result
}

/// Percent-decode a URL-encoded string
/// Returns None if the string contains invalid percent encoding
fn percent_decode(s: &str) -> Option<String> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            // Need at least 2 more characters
            if i + 2 >= bytes.len() {
                return None;
            }

            // Parse hex digits
            let hex_str = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            let decoded_byte = u8::from_str_radix(hex_str, 16).ok()?;
            result.push(decoded_byte);
            i += 3;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    // Convert bytes to UTF-8 string
    String::from_utf8(result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse_simple() {
        let url = Value::Str("https://example.com".to_string());
        let result = url_parse(&url).unwrap();

        // Should be Some(UrlInfo)
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);

                // Check the UrlInfo record
                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(r.get("scheme"), Some(&Value::Str("https".to_string())));
                    assert_eq!(r.get("host"), Some(&Value::Str("example.com".to_string())));
                    assert_eq!(r.get("path"), Some(&Value::Str("/".to_string())));

                    // port should be None
                    if let Some(Value::Variant { variant_name, .. }) = r.get("port") {
                        assert_eq!(variant_name, "None");
                    } else {
                        panic!("port field missing or wrong type");
                    }
                } else {
                    panic!("Expected Record");
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_with_port() {
        let url = Value::Str("http://localhost:8080".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(r.get("scheme"), Some(&Value::Str("http".to_string())));
                    assert_eq!(r.get("host"), Some(&Value::Str("localhost".to_string())));

                    // port should be Some(8080)
                    if let Some(Value::Variant {
                        variant_name,
                        fields,
                        ..
                    }) = r.get("port")
                    {
                        assert_eq!(variant_name, "Some");
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0], Value::Int(8080));
                    } else {
                        panic!("port field missing or wrong type");
                    }
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_with_path() {
        let url = Value::Str("https://example.com/api/v1/users".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(
                        r.get("path"),
                        Some(&Value::Str("/api/v1/users".to_string()))
                    );
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_with_query() {
        let url = Value::Str("https://example.com/search?q=fusabi&lang=en".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();

                    // query should be Some("q=fusabi&lang=en")
                    if let Some(Value::Variant {
                        variant_name,
                        fields,
                        ..
                    }) = r.get("query")
                    {
                        assert_eq!(variant_name, "Some");
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0], Value::Str("q=fusabi&lang=en".to_string()));
                    } else {
                        panic!("query field missing or wrong type");
                    }
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_with_fragment() {
        let url = Value::Str("https://example.com/page#section".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();

                    // fragment should be Some("section")
                    if let Some(Value::Variant {
                        variant_name,
                        fields,
                        ..
                    }) = r.get("fragment")
                    {
                        assert_eq!(variant_name, "Some");
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0], Value::Str("section".to_string()));
                    } else {
                        panic!("fragment field missing or wrong type");
                    }
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_complex() {
        let url = Value::Str(
            "https://user:pass@example.com:8443/path/to/resource?key=value#anchor".to_string(),
        );
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(r.get("scheme"), Some(&Value::Str("https".to_string())));
                    // Note: our simple parser doesn't separate user:pass from host
                    assert_eq!(
                        r.get("host"),
                        Some(&Value::Str("user:pass@example.com".to_string()))
                    );
                    assert_eq!(
                        r.get("path"),
                        Some(&Value::Str("/path/to/resource".to_string()))
                    );
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_invalid() {
        // No scheme
        let url = Value::Str("example.com".to_string());
        let result = url_parse(&url).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }

        // Empty string
        let url = Value::Str("".to_string());
        let result = url_parse(&url).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }

        // Invalid port
        let url = Value::Str("http://example.com:99999".to_string());
        let result = url_parse(&url).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_url_is_valid() {
        assert_eq!(
            url_is_valid(&Value::Str("https://example.com".to_string())).unwrap(),
            Value::Bool(true)
        );

        assert_eq!(
            url_is_valid(&Value::Str("http://localhost:8080/api".to_string())).unwrap(),
            Value::Bool(true)
        );

        assert_eq!(
            url_is_valid(&Value::Str("example.com".to_string())).unwrap(),
            Value::Bool(false)
        );

        assert_eq!(
            url_is_valid(&Value::Str("".to_string())).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_url_encode() {
        let input = Value::Str("hello world".to_string());
        let result = url_encode(&input).unwrap();
        assert_eq!(result, Value::Str("hello%20world".to_string()));

        let input = Value::Str("test@example.com".to_string());
        let result = url_encode(&input).unwrap();
        assert_eq!(result, Value::Str("test%40example.com".to_string()));

        let input = Value::Str("a+b=c&d".to_string());
        let result = url_encode(&input).unwrap();
        assert_eq!(result, Value::Str("a%2Bb%3Dc%26d".to_string()));

        // Unreserved characters should not be encoded
        let input = Value::Str("ABCabc123-_._~".to_string());
        let result = url_encode(&input).unwrap();
        assert_eq!(result, Value::Str("ABCabc123-_._~".to_string()));
    }

    #[test]
    fn test_url_decode() {
        let input = Value::Str("hello%20world".to_string());
        let result = url_decode(&input).unwrap();
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

        let input = Value::Str("test%40example.com".to_string());
        let result = url_decode(&input).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields[0], Value::Str("test@example.com".to_string()));
            }
            _ => panic!("Expected Some variant"),
        }

        // Invalid encoding
        let input = Value::Str("test%2".to_string());
        let result = url_decode(&input).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }

        let input = Value::Str("test%ZZ".to_string());
        let result = url_decode(&input).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_url_encode_decode_roundtrip() {
        let original = "Hello World! 你好 @#$%^&*()";
        let input = Value::Str(original.to_string());

        let encoded = url_encode(&input).unwrap();
        let decoded = url_decode(&encoded).unwrap();

        match decoded {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields[0], Value::Str(original.to_string()));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_ipv6() {
        let url = Value::Str("http://[2001:db8::1]:8080/path".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(
                        r.get("host"),
                        Some(&Value::Str("[2001:db8::1]".to_string()))
                    );

                    // port should be Some(8080)
                    if let Some(Value::Variant {
                        variant_name,
                        fields,
                        ..
                    }) = r.get("port")
                    {
                        assert_eq!(variant_name, "Some");
                        assert_eq!(fields[0], Value::Int(8080));
                    } else {
                        panic!("port field missing");
                    }
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_url_parse_various_schemes() {
        let schemes = vec!["http", "https", "ftp", "ws", "wss", "file"];

        for scheme in schemes {
            let url = Value::Str(format!("{}://example.com", scheme));
            let result = url_parse(&url).unwrap();

            match result {
                Value::Variant {
                    variant_name,
                    fields,
                    ..
                } => {
                    assert_eq!(variant_name, "Some");

                    if let Value::Record(record) = &fields[0] {
                        let r = record.lock().unwrap();
                        assert_eq!(r.get("scheme"), Some(&Value::Str(scheme.to_string())));
                    }
                }
                _ => panic!("Expected Some variant for scheme {}", scheme),
            }
        }
    }

    #[test]
    fn test_url_type_errors() {
        let not_string = Value::Int(42);
        assert!(url_parse(&not_string).is_err());
        assert!(url_is_valid(&not_string).is_err());
        assert!(url_encode(&not_string).is_err());
        assert!(url_decode(&not_string).is_err());
    }

    #[test]
    fn test_is_valid_scheme() {
        assert!(is_valid_scheme("http"));
        assert!(is_valid_scheme("https"));
        assert!(is_valid_scheme("ftp"));
        assert!(is_valid_scheme("file"));
        assert!(is_valid_scheme("ws"));
        assert!(is_valid_scheme("wss"));
        assert!(is_valid_scheme("http+tls"));
        assert!(is_valid_scheme("git+ssh"));

        assert!(!is_valid_scheme("")); // Empty
        assert!(!is_valid_scheme("123")); // Starts with digit
        assert!(!is_valid_scheme("ht@tp")); // Invalid character
        assert!(!is_valid_scheme("ht tp")); // Space
    }

    #[test]
    fn test_percent_encode_all_ascii() {
        // Test that percent encoding handles all ASCII characters correctly
        let input = Value::Str("!\"#$%&'()*+,/:;=?@[]".to_string());
        let result = url_encode(&input).unwrap();

        // All special characters should be encoded
        if let Value::Str(s) = result {
            assert!(s.contains("%21")); // !
            assert!(s.contains("%3D")); // =
            assert!(s.contains("%3F")); // ?
            assert!(s.contains("%40")); // @
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_url_parse_missing_path() {
        // URL without explicit path should get default "/"
        let url = Value::Str("https://example.com".to_string());
        let result = url_parse(&url).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");

                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert_eq!(r.get("path"), Some(&Value::Str("/".to_string())));
                }
            }
            _ => panic!("Expected Some variant"),
        }
    }
}
