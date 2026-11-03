//! SigmaOS-init - SigmaOS Component
//!
//! Critical system infrastructure component for SigmaOS,
//! inspired by Arch Linux and Debian architecture.

use anyhow::Result;
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub debug: bool,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            debug: false,
            log_level: "info".to_string(),
        }
    }
}

/// Component main trait
pub trait Component {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self) -> Result<()>;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn status(&self) -> ComponentStatus;
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus {
    Running,
    Stopped,
    Error(String),
    Initializing,
}

/// Main component implementation
pub struct init {
    config: Config,
    status: ComponentStatus,
}

impl init {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            status: ComponentStatus::Stopped,
        }
    }

    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(Self::new(config))
    }
}

impl Component for init {
    fn name(&self) -> &str {
        "SigmaOS-init"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn initialize(&mut self) -> Result<()> {
        info!("Initializing SigmaOS-init");
        self.status = ComponentStatus::Initializing;
        
        // Initialize logging
        if self.config.debug {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .init();
        } else {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&self.config.log_level))
                .init();
        }
        
        info!("Component configuration: {:?}", self.config);
        self.status = ComponentStatus::Stopped;
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        info!("Starting SigmaOS-init");
        self.status = ComponentStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Stopping SigmaOS-init");
        self.status = ComponentStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        self.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let config = Config::default();
        let component = init::new(config);
        assert_eq!(component.name(), "SigmaOS-init");
        assert_eq!(component.version(), "0.1.0");
    }

    #[test]
    fn test_initialization() {
        let mut component = init::new(Config::default());
        component.initialize().unwrap();
        assert!(matches!(component.status(), ComponentStatus::Stopped));
    }

    #[test]
    fn test_start_stop() {
        let mut component = init::new(Config::default());
        component.initialize().unwrap();
        component.start().unwrap();
        assert!(matches!(component.status(), ComponentStatus::Running));
        component.stop().unwrap();
        assert!(matches!(component.status(), ComponentStatus::Stopped));
    }
}
