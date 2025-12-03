/// Error handling utilities for WASM plugins
/// This module provides safe error handling and reporting for no_std plugins
/// Error codes for plugin execution
pub const ERROR_INVALID_UTF8: i32 = -1;
pub const ERROR_INVALID_INPUT: i32 = -2;
pub const ERROR_BUFFER_OVERFLOW: i32 = -3;
pub const ERROR_MEMORY_ALLOCATION: i32 = -4;
pub const ERROR_PARSE_ERROR: i32 = -5;
pub const ERROR_ENV_PARSING: i32 = -6;
pub const ERROR_UNKNOWN: i32 = -99;

/// Success code
pub const SUCCESS: i32 = 0;

/// Error result wrapper for plugin operations
pub type PluginResult<T> = Result<T, i32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ERROR_INVALID_UTF8, -1);
        assert_eq!(SUCCESS, 0);
    }
}
