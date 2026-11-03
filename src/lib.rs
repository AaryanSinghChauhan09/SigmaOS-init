//! SigmaOS Init
//! System initialization and init system

pub mod init;
pub mod systemd;

pub use init::SigmaInit as InitSystem;
pub use init::*;
pub use systemd::*;
