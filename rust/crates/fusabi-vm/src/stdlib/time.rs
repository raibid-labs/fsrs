// Fusabi Time Standard Library
// Provides time-related functions for working with timestamps and time formatting

use crate::value::Value;
use crate::vm::VmError;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Time Functions
// ============================================================================

/// Time.now : unit -> int
/// Returns the current Unix timestamp in milliseconds since the epoch
pub fn time_now(_unit: &Value) -> Result<Value, VmError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| Value::Int(duration.as_millis() as i64))
        .map_err(|e| VmError::Runtime(format!("Failed to get current time: {}", e)))
}

/// Time.nowSeconds : unit -> int
/// Returns the current Unix timestamp in seconds since the epoch
pub fn time_now_seconds(_unit: &Value) -> Result<Value, VmError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| Value::Int(duration.as_secs() as i64))
        .map_err(|e| VmError::Runtime(format!("Failed to get current time: {}", e)))
}

/// Time.format : string -> int -> string
/// Formats a Unix timestamp (in milliseconds) according to a format string
///
/// Supported format specifiers:
/// - %Y - Year (4 digits)
/// - %m - Month (01-12)
/// - %d - Day of month (01-31)
/// - %H - Hour (00-23)
/// - %M - Minute (00-59)
/// - %S - Second (00-59)
/// - %% - Literal '%'
///
/// Example: Time.format "%Y-%m-%d %H:%M:%S" timestamp
pub fn time_format(format_str: &Value, timestamp: &Value) -> Result<Value, VmError> {
    let fmt = match format_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: format_str.type_name(),
            })
        }
    };

    let timestamp_ms = match timestamp {
        Value::Int(ts) => *ts,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: timestamp.type_name(),
            })
        }
    };

    // Convert milliseconds to seconds for SystemTime
    let timestamp_secs = timestamp_ms / 1000;
    let _timestamp_millis = (timestamp_ms % 1000) as u32; // Reserved for future millisecond formatting

    // Create SystemTime from timestamp
    let system_time = if timestamp_secs >= 0 {
        UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(timestamp_secs as u64))
            .ok_or_else(|| VmError::Runtime("Timestamp overflow".to_string()))?
    } else {
        let abs_secs = timestamp_secs.unsigned_abs();
        UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(abs_secs))
            .ok_or_else(|| VmError::Runtime("Timestamp underflow".to_string()))?
    };

    // Convert to calendar time (simplified UTC calculation)
    let duration = system_time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| VmError::Runtime("Invalid timestamp".to_string()))?;

    let total_seconds = duration.as_secs();
    let days_since_epoch = total_seconds / 86400;
    let seconds_today = total_seconds % 86400;

    let hours = (seconds_today / 3600) as u32;
    let minutes = ((seconds_today % 3600) / 60) as u32;
    let seconds = (seconds_today % 60) as u32;

    // Simple calendar calculation (approximation for demonstration)
    // This is a simplified algorithm that works for most modern dates
    let mut year = 1970;
    let mut days_remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining < days_in_year {
            break;
        }
        days_remaining -= days_in_year;
        year += 1;
    }

    let (month, day) = days_to_month_day(days_remaining as u32, is_leap_year(year));

    // Format the string
    let mut result = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    '%' => {
                        result.push('%');
                        chars.next();
                    }
                    'Y' => {
                        result.push_str(&format!("{:04}", year));
                        chars.next();
                    }
                    'm' => {
                        result.push_str(&format!("{:02}", month));
                        chars.next();
                    }
                    'd' => {
                        result.push_str(&format!("{:02}", day));
                        chars.next();
                    }
                    'H' => {
                        result.push_str(&format!("{:02}", hours));
                        chars.next();
                    }
                    'M' => {
                        result.push_str(&format!("{:02}", minutes));
                        chars.next();
                    }
                    'S' => {
                        result.push_str(&format!("{:02}", seconds));
                        chars.next();
                    }
                    _ => {
                        return Err(VmError::Runtime(format!(
                            "Unknown time format specifier: %{}",
                            next_ch
                        )))
                    }
                }
            } else {
                return Err(VmError::Runtime(
                    "Incomplete format specifier at end of string".to_string(),
                ));
            }
        } else {
            result.push(ch);
        }
    }

    Ok(Value::Str(result))
}

/// Time.parse : string -> string -> int option
/// Parses a time string according to a format string, returning Some timestamp or None
///
/// Supported format specifiers:
/// - %Y - Year (4 digits)
/// - %m - Month (01-12)
/// - %d - Day of month (01-31)
/// - %H - Hour (00-23)
/// - %M - Minute (00-59)
/// - %S - Second (00-59)
///
/// Example: Time.parse "%Y-%m-%d" "2024-03-15"
pub fn time_parse(format_str: &Value, time_str: &Value) -> Result<Value, VmError> {
    let fmt = match format_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: format_str.type_name(),
            })
        }
    };

    let input = match time_str {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: time_str.type_name(),
            })
        }
    };

    // Parse the format string and extract values
    let parse_result = parse_time_string(fmt, input);

    match parse_result {
        Ok((year, month, day, hour, minute, second)) => {
            // Validate parsed values
            if month < 1 || month > 12 {
                return Ok(create_none());
            }
            if day < 1 || day > days_in_month(month, is_leap_year(year as i64)) {
                return Ok(create_none());
            }
            if hour > 23 || minute > 59 || second > 59 {
                return Ok(create_none());
            }

            // Convert to Unix timestamp
            match calculate_timestamp(year, month, day, hour, minute, second) {
                Ok(timestamp) => Ok(create_some(Value::Int(timestamp))),
                Err(_) => Ok(create_none()),
            }
        }
        Err(_) => Ok(create_none()),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u32, leap_year: bool) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap_year {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn days_to_month_day(mut days: u32, leap_year: bool) -> (u32, u32) {
    for month in 1..=12 {
        let days_in_this_month = days_in_month(month, leap_year);
        if days < days_in_this_month {
            return (month, days + 1);
        }
        days -= days_in_this_month;
    }
    (12, 31) // Fallback
}

fn calculate_timestamp(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<i64, String> {
    // Calculate days since epoch (1970-01-01)
    if year < 1970 {
        return Err("Year before 1970 not supported".to_string());
    }

    let mut days = 0i64;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y as i64) { 366 } else { 365 };
    }

    // Add days for complete months in current year
    for m in 1..month {
        days += days_in_month(m, is_leap_year(year as i64)) as i64;
    }

    // Add remaining days
    days += (day - 1) as i64;

    // Convert to milliseconds
    let total_seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    Ok(total_seconds * 1000)
}

fn parse_time_string(format: &str, input: &str) -> Result<(i32, u32, u32, u32, u32, u32), String> {
    let mut year = 1970i32;
    let mut month = 1u32;
    let mut day = 1u32;
    let mut hour = 0u32;
    let mut minute = 0u32;
    let mut second = 0u32;

    let mut fmt_chars = format.chars().peekable();
    let mut input_chars = input.chars().peekable();

    while let Some(fmt_ch) = fmt_chars.next() {
        if fmt_ch == '%' {
            if let Some(&specifier) = fmt_chars.peek() {
                fmt_chars.next(); // consume specifier

                match specifier {
                    'Y' => {
                        // Read 4 digits
                        let mut year_str = String::new();
                        for _ in 0..4 {
                            if let Some(&ch) = input_chars.peek() {
                                if ch.is_ascii_digit() {
                                    year_str.push(ch);
                                    input_chars.next();
                                } else {
                                    return Err("Expected digit for year".to_string());
                                }
                            } else {
                                return Err("Unexpected end of input".to_string());
                            }
                        }
                        year = year_str.parse().map_err(|_| "Invalid year".to_string())?;
                    }
                    'm' => {
                        month = parse_two_digits(&mut input_chars)?;
                    }
                    'd' => {
                        day = parse_two_digits(&mut input_chars)?;
                    }
                    'H' => {
                        hour = parse_two_digits(&mut input_chars)?;
                    }
                    'M' => {
                        minute = parse_two_digits(&mut input_chars)?;
                    }
                    'S' => {
                        second = parse_two_digits(&mut input_chars)?;
                    }
                    _ => {
                        return Err(format!("Unknown format specifier: %{}", specifier));
                    }
                }
            } else {
                return Err("Incomplete format specifier".to_string());
            }
        } else {
            // Match literal character
            if let Some(&input_ch) = input_chars.peek() {
                if input_ch == fmt_ch {
                    input_chars.next();
                } else {
                    return Err(format!("Expected '{}', got '{}'", fmt_ch, input_ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
    }

    Ok((year, month, day, hour, minute, second))
}

fn parse_two_digits(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<u32, String> {
    let mut num_str = String::new();
    for _ in 0..2 {
        if let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                chars.next();
            } else {
                return Err("Expected digit".to_string());
            }
        } else {
            return Err("Unexpected end of input".to_string());
        }
    }
    num_str.parse().map_err(|_| "Invalid number".to_string())
}

fn create_some(value: Value) -> Value {
    Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![value],
    }
}

fn create_none() -> Value {
    Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        fields: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_now() {
        let unit = Value::Variant {
            type_name: "Unit".to_string(),
            variant_name: "()".to_string(),
            fields: vec![],
        };

        let result = time_now(&unit).unwrap();
        match result {
            Value::Int(ts) => {
                // Timestamp should be a reasonable value (after 2020-01-01 and before 2100-01-01)
                assert!(ts > 1577836800000); // 2020-01-01 in ms
                assert!(ts < 4102444800000); // 2100-01-01 in ms
            }
            _ => panic!("Expected Int, got {:?}", result),
        }
    }

    #[test]
    fn test_time_now_seconds() {
        let unit = Value::Variant {
            type_name: "Unit".to_string(),
            variant_name: "()".to_string(),
            fields: vec![],
        };

        let result = time_now_seconds(&unit).unwrap();
        match result {
            Value::Int(ts) => {
                // Timestamp should be a reasonable value (after 2020-01-01 and before 2100-01-01)
                assert!(ts > 1577836800); // 2020-01-01 in seconds
                assert!(ts < 4102444800); // 2100-01-01 in seconds
            }
            _ => panic!("Expected Int, got {:?}", result),
        }
    }

    #[test]
    fn test_time_now_vs_now_seconds() {
        let unit = Value::Variant {
            type_name: "Unit".to_string(),
            variant_name: "()".to_string(),
            fields: vec![],
        };

        let ms_result = time_now(&unit).unwrap();
        let s_result = time_now_seconds(&unit).unwrap();

        if let (Value::Int(ms), Value::Int(s)) = (ms_result, s_result) {
            // The millisecond timestamp divided by 1000 should be close to the second timestamp
            let ms_in_seconds = ms / 1000;
            assert!((ms_in_seconds - s).abs() <= 1); // Allow 1 second difference
        } else {
            panic!("Expected Int values");
        }
    }

    #[test]
    fn test_time_format_basic() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        // 2024-03-15 00:00:00 UTC = 1710460800000 ms
        let timestamp = Value::Int(1710460800000);

        let result = time_format(&fmt, &timestamp).unwrap();
        assert_eq!(result, Value::Str("2024-03-15".to_string()));
    }

    #[test]
    fn test_time_format_with_time() {
        let fmt = Value::Str("%Y-%m-%d %H:%M:%S".to_string());
        // 2024-03-15 14:30:45 UTC = 1710513045000 ms
        let timestamp = Value::Int(1710513045000);

        let result = time_format(&fmt, &timestamp).unwrap();
        assert_eq!(result, Value::Str("2024-03-15 14:30:45".to_string()));
    }

    #[test]
    fn test_time_format_epoch() {
        let fmt = Value::Str("%Y-%m-%d %H:%M:%S".to_string());
        let timestamp = Value::Int(0); // Unix epoch

        let result = time_format(&fmt, &timestamp).unwrap();
        assert_eq!(result, Value::Str("1970-01-01 00:00:00".to_string()));
    }

    #[test]
    fn test_time_format_literal_percent() {
        let fmt = Value::Str("Date: %Y-%m-%d %%".to_string());
        let timestamp = Value::Int(1710460800000);

        let result = time_format(&fmt, &timestamp).unwrap();
        assert_eq!(result, Value::Str("Date: 2024-03-15 %".to_string()));
    }

    #[test]
    fn test_time_format_invalid_format_string() {
        let fmt = Value::Int(42);
        let timestamp = Value::Int(1710460800000);

        let result = time_format(&fmt, &timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_format_invalid_timestamp() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let timestamp = Value::Str("not a number".to_string());

        let result = time_format(&fmt, &timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_format_unknown_specifier() {
        let fmt = Value::Str("%Y-%m-%d %X".to_string());
        let timestamp = Value::Int(1710460800000);

        let result = time_format(&fmt, &timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parse_basic() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Str("2024-03-15".to_string());

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                // 2024-03-15 00:00:00 UTC = 1710460800000 ms
                assert_eq!(fields[0], Value::Int(1710460800000));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_time_parse_with_time() {
        let fmt = Value::Str("%Y-%m-%d %H:%M:%S".to_string());
        let input = Value::Str("2024-03-15 14:30:45".to_string());

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                // 2024-03-15 14:30:45 UTC = 1710513045000 ms
                assert_eq!(fields[0], Value::Int(1710513045000));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_time_parse_epoch() {
        let fmt = Value::Str("%Y-%m-%d %H:%M:%S".to_string());
        let input = Value::Str("1970-01-01 00:00:00".to_string());

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(0));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_time_parse_invalid_month() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Str("2024-13-15".to_string());

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_time_parse_invalid_day() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Str("2024-02-30".to_string()); // February doesn't have 30 days

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_time_parse_malformed_input() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Str("not-a-date".to_string());

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_time_parse_format_mismatch() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Str("2024/03/15".to_string()); // Wrong separator

        let result = time_parse(&fmt, &input).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_time_parse_invalid_format_string() {
        let fmt = Value::Int(42);
        let input = Value::Str("2024-03-15".to_string());

        let result = time_parse(&fmt, &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parse_invalid_input_type() {
        let fmt = Value::Str("%Y-%m-%d".to_string());
        let input = Value::Int(42);

        let result = time_parse(&fmt, &input);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(is_leap_year(2004)); // Divisible by 4, not by 100
        assert!(!is_leap_year(1900)); // Divisible by 100, not by 400
        assert!(!is_leap_year(2001)); // Not divisible by 4
        assert!(is_leap_year(2024));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(1, false), 31);
        assert_eq!(days_in_month(2, false), 28);
        assert_eq!(days_in_month(2, true), 29);
        assert_eq!(days_in_month(4, false), 30);
        assert_eq!(days_in_month(12, false), 31);
    }

    #[test]
    fn test_time_roundtrip() {
        let fmt = Value::Str("%Y-%m-%d %H:%M:%S".to_string());
        let timestamp = Value::Int(1710512645000);

        // Format timestamp to string
        let formatted = time_format(&fmt, &timestamp).unwrap();

        // Parse string back to timestamp
        let parsed = time_parse(&fmt, &formatted).unwrap();

        match parsed {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields[0], timestamp);
            }
            _ => panic!("Expected Some variant"),
        }
    }
}
