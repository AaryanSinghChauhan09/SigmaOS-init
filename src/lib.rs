//! SigmaOS-init - System Init & Process Supervisor for SigmaOS
//!
//! Critical system infrastructure component for SigmaOS,
//! inspired by Debian and Arch Linux architecture.

#![allow(dead_code)]

pub mod config;
pub mod error;
pub mod service;

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use config::Config as InitConfig;
pub use error::{ComponentError, Result as ComponentResult};
pub use service::Service;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentStatus {
    Running,
    Stopped,
    Error(String),
    Initializing,
}

/// Main component implementation
pub struct Init {
    config: Config,
    status: ComponentStatus,
}

pub type init = Init;

impl Init {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            status: ComponentStatus::Stopped,
        }
    }

    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(Self::new(config))
    }
}

impl Component for Init {
    fn name(&self) -> &str {
        "SigmaOS-init"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn initialize(&mut self) -> Result<()> {
        info!("Initializing SigmaOS-init");
        self.status = ComponentStatus::Initializing;
        
        let _ = env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or(&self.config.log_level),
        )
        .try_init();
        
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
    fn test_component_creation_aaa() {
        let config = Config::default();
        let component = Init::new(config);
        assert_eq!(component.name(), "SigmaOS-init");
        assert_eq!(component.version(), "0.1.0");
    }

    #[test]
    fn test_initialization_aaa() {
        let mut component = Init::new(Config::default());
        component.initialize().unwrap();
        assert_eq!(component.status(), ComponentStatus::Stopped);
    }

    #[test]
    fn test_start_stop_aaa() {
        let mut component = Init::new(Config::default());
        component.initialize().unwrap();
        component.start().unwrap();
        assert_eq!(component.status(), ComponentStatus::Running);
        component.stop().unwrap();
        assert_eq!(component.status(), ComponentStatus::Stopped);
    }
}
