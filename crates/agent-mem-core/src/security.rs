//! Security validation utilities for SQL injection prevention
//!
//! This module provides whitelist-based validation to prevent SQL injection attacks.
//!
//! # Security
//!
//! All table names and column names are validated against:
//! 1. Whitelist of allowed tables
//! 2. Regex pattern (only alphanumeric + underscore)
//! 3. Length limits

use crate::{CoreError, CoreResult};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

/// Maximum table name length (PostgreSQL limit is 63, we use 64 for safety)
const MAX_TABLE_NAME_LENGTH: usize = 64;

/// Maximum column name length
const MAX_COLUMN_NAME_LENGTH: usize = 64;

lazy_static! {
    /// Whitelist of allowed table names
    static ref ALLOWED_TABLES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        // Core tables
        set.insert("memories");
        set.insert("agents");
        set.insert("messages");
        set.insert("users");
        set.insert("organizations");
        set.insert("api_keys");
        set.insert("blocks");
        set.insert("associations");
        // Add more tables as needed
        set
    };

    /// Table name validation regex (only letters, numbers, underscores)
    static ref TABLE_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,63}$").unwrap();

    /// Column name validation regex
    static ref COLUMN_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,63}$").unwrap();
}

/// Validates a table name against whitelist and pattern rules
///
/// # Arguments
///
/// * `table_name` - The table name to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, `Err(CoreError::InvalidInput)` if invalid
///
/// # Errors
///
/// Returns `CoreError::InvalidInput` if:
/// - Table name is not in the whitelist
/// - Table name contains invalid characters
/// - Table name exceeds maximum length
pub fn validate_table_name(table_name: &str) -> CoreResult<()> {
    // Check length first
    if table_name.len() > MAX_TABLE_NAME_LENGTH {
        return Err(CoreError::InvalidInput(format!(
            "Table name '{}' exceeds maximum length of {}",
            table_name, MAX_TABLE_NAME_LENGTH
        )));
    }

    // Check against whitelist
    if !ALLOWED_TABLES.contains(table_name) {
        return Err(CoreError::InvalidInput(format!(
            "Table '{}' is not in the allowed list. Allowed tables: {}",
            table_name,
            ALLOWED_TABLES.iter().copied().collect::<Vec<_>>().join(", ")
        )));
    }

    // Check pattern (defensive in case whitelist is bypassed)
    if !TABLE_NAME_REGEX.is_match(table_name) {
        return Err(CoreError::InvalidInput(format!(
            "Invalid table name '{}': must start with a letter or underscore and contain only letters, numbers, and underscores",
            table_name
        )));
    }

    Ok(())
}

/// Validates a list of column names against pattern rules
///
/// # Arguments
///
/// * `columns` - Slice of column names to validate
///
/// # Returns
///
/// Returns `Ok(())` if all columns are valid, `Err` otherwise
pub fn validate_column_names(columns: &[&str]) -> CoreResult<()> {
    for column in columns {
        validate_column_name(column)?;
    }
    Ok(())
}

/// Validates a single column name
pub fn validate_column_name(column_name: &str) -> CoreResult<()> {
    if column_name.len() > MAX_COLUMN_NAME_LENGTH {
        return Err(CoreError::InvalidInput(format!(
            "Column name '{}' exceeds maximum length of {}",
            column_name, MAX_COLUMN_NAME_LENGTH
        )));
    }

    if !COLUMN_NAME_REGEX.is_match(column_name) {
        return Err(CoreError::InvalidInput(format!(
            "Invalid column name '{}': must start with a letter or underscore and contain only letters, numbers, and underscores",
            column_name
        )));
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_validate_table_name_valid() {
        assert!(validate_table_name("memories").is_ok());
        assert!(validate_table_name("agents").is_ok());
    }

    #[test]
    fn test_validate_table_name_sql_injection() {
        assert!(validate_table_name("memories; DROP TABLE memories; --").is_err());
        assert!(validate_table_name("memories' OR '1'='1").is_err());
    }

    #[test]
    fn test_validate_table_name_not_in_whitelist() {
        assert!(validate_table_name("sensitive_data").is_err());
    }

    #[test]
    fn test_validate_column_names_valid() {
        assert!(validate_column_names(&["id", "content", "created_at"]).is_ok());
    }

    #[test]
    fn test_validate_column_names_sql_injection() {
        assert!(validate_column_names(&["id; DROP TABLE users; --"]).is_err());
    }
}
