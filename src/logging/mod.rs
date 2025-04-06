// Logging utilities for the Runar system
//
// This module provides a comprehensive logging system with:
// - Compile-time efficient macros
// - Component-based structured logging
// - Context-aware logging for services
// - Node ID tracking through logger inheritance

use log::{debug, error, info, warn};

// Include macros submodule
pub mod macros;

/// Predefined components for logging categorization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Component {
    Node,
    Registry,
    Service,
    Database,
    Network,
    System,
    Custom(&'static str),
}

impl Component {
    /// Get the string representation of the component
    pub fn as_str(&self) -> &str {
        match self {
            Component::Node => "Node",
            Component::Registry => "Registry",
            Component::Service => "Service",
            Component::Database => "DB",
            Component::Network => "Network",
            Component::System => "System",
            Component::Custom(name) => name,
        }
    }
}

/// A helper for creating component-specific loggers with node ID tracking
#[derive(Clone)]
pub struct Logger {
    /// Component this logger is for
    component: Component,
    /// Node ID for distributed tracing
    node_id: String,
    /// Parent component for hierarchical logging (if any)
    parent_component: Option<Component>,
}

impl Logger {
    /// Create a new root logger for a specific component and node ID
    /// This should only be called by the Node root component
    pub fn new_root(component: Component, node_id: &str) -> Self {
        Self { 
            component,
            node_id: node_id.to_string(),
            parent_component: None,
        }
    }
    
    /// Create a child logger with the same node ID but different component
    /// This is the preferred way to create loggers in services and other components
    pub fn with_component(&self, component: Component) -> Self {
        Self {
            component,
            node_id: self.node_id.clone(),
            parent_component: Some(self.component),
        }
    }
    
    /// Clone this logger with the same settings
    /// This is useful when you need to pass a logger to a component that might modify it
    pub fn clone_logger(&self) -> Self {
        self.clone()
    }
    
    /// Get a reference to the node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    
    /// Get the component prefix for logging, including parent if available
    fn component_prefix(&self) -> String {
        match self.parent_component {
            Some(parent) if parent != Component::Node => 
                format!("{}.{}", parent.as_str(), self.component.as_str()),
            _ => self.component.as_str().to_string(),
        }
    }
    
    /// Log a debug message
    pub fn debug(&self, message: impl Into<String>) {
        if log::log_enabled!(log::Level::Debug) {
            debug!("[{}][{}] {}", self.node_id, self.component_prefix(), message.into());
        }
    }
    
    /// Log an info message
    pub fn info(&self, message: impl Into<String>) {
        if log::log_enabled!(log::Level::Info) {
            info!("[{}][{}] {}", self.node_id, self.component_prefix(), message.into());
        }
    }
    
    /// Log a warning message
    pub fn warn(&self, message: impl Into<String>) {
        if log::log_enabled!(log::Level::Warn) {
            warn!("[{}][{}] {}", self.node_id, self.component_prefix(), message.into());
        }
    }
    
    /// Log an error message
    pub fn error(&self, message: impl Into<String>) {
        if log::log_enabled!(log::Level::Error) {
            error!("[{}][{}] {}", self.node_id, self.component_prefix(), message.into());
        }
    }
}

/// Logging context for structured logging with additional context
pub trait LoggingContext {
    /// Get the component
    fn component(&self) -> Component;
    
    /// Get the service path or identifier
    fn service_path(&self) -> Option<&str>;
    
    /// Get the logger
    fn logger(&self) -> &Logger;
    
    /// Log at debug level
    fn log_debug(&self, message: String) {
        if log::log_enabled!(log::Level::Debug) {
            let prefix = self.log_prefix();
            let logger = self.logger();
            debug!("[{}][{}] {}", logger.node_id(), prefix, message);
        }
    }
    
    /// Log at info level
    fn log_info(&self, message: String) {
        if log::log_enabled!(log::Level::Info) {
            let prefix = self.log_prefix();
            let logger = self.logger();
            info!("[{}][{}] {}", logger.node_id(), prefix, message);
        }
    }
    
    /// Log at warning level
    fn log_warn(&self, message: String) {
        if log::log_enabled!(log::Level::Warn) {
            let prefix = self.log_prefix();
            let logger = self.logger();
            warn!("[{}][{}] {}", logger.node_id(), prefix, message);
        }
    }
    
    /// Log at error level
    fn log_error(&self, message: String) {
        if log::log_enabled!(log::Level::Error) {
            let prefix = self.log_prefix();
            let logger = self.logger();
            error!("[{}][{}] {}", logger.node_id(), prefix, message);
        }
    }
    
    /// Get the logging prefix
    fn log_prefix(&self) -> String {
        match self.service_path() {
            Some(path) => format!("{}:{}", self.component().as_str(), path),
            None => self.component().as_str().to_string(),
        }
    }
} 