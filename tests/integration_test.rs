use anyhow::Result;
use sigmaos_component::{Component, ComponentStatus, Config, Init, InitConfig, Service};

#[test]
fn test_init_lifecycle_aaa() -> Result<()> {
    // Arrange
    let config = Config::default();
    let mut init = Init::new(config);
    assert_eq!(init.status(), ComponentStatus::Stopped);

    // Act
    init.initialize()?;
    init.start()?;

    // Assert
    assert_eq!(init.status(), ComponentStatus::Running);

    // Act
    init.stop()?;

    // Assert
    assert_eq!(init.status(), ComponentStatus::Stopped);
    Ok(())
}

#[test]
fn test_init_service_lifecycle_aaa() -> Result<()> {
    // Arrange
    let config = InitConfig::default();
    let mut service = Service::new(config);
    assert!(!service.is_running());

    // Act
    service.start()?;
    // Assert
    assert!(service.is_running());

    // Act
    service.stop()?;
    // Assert
    assert!(!service.is_running());
    Ok(())
}
