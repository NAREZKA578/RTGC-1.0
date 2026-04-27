//! Logger utilities for RTGC-0.9
//! Provides centralized logging configuration

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging system with configurable log level
pub fn init_logger() {
    // Try to initialize tracing subscriber with env filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_timer(fmt::time::SystemTime::default())
        .with_env_filter(filter)
        .finish()
        .init();
    
    tracing::info!("Logger initialized");
}

/// Initialize logger with custom default level
pub fn init_logger_with_level(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .finish()
        .init();
    
    tracing::info!("Logger initialized with level: {}", level);
}

/// Set log level at runtime (requires dynamic filtering)
pub fn set_log_level(level: &str) {
    tracing::info!("Setting log level to: {}", level);
    // Note: This would require a reload handle for dynamic changes
    // For now, this is a placeholder for future implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        // Just verify it doesn't panic
        // Note: Can only init once, so this test may fail if run multiple times
        let result = std::panic::catch_unwind(|| {
            init_logger();
        });
        // Test passes if no panic or if already initialized
        assert!(result.is_ok());
    }
}
