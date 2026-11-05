#[derive(Debug, Clone)]
pub struct RcService {
    pub name: String,
    pub script_path: String,
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct InitSupervisor {
    services: Vec<RcService>,
}

impl InitSupervisor {
    pub fn new() -> Self { Self::default() }
    pub fn register_rc(&mut self, name: &str, path: &str, enabled: bool) {
        self.services.push(RcService {
            name: name.to_string(),
            script_path: path.to_string(),
            enabled,
        });
    }
    pub fn enabled_service_count(&self) -> usize {
        self.services.iter().filter(|s| s.enabled).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init_supervisor_rc_services() {
        let mut init = InitSupervisor::new();
        init.register_rc("rc.network", "/etc/rc.d/rc.network", true);
        init.register_rc("rc.sshd", "/etc/rc.d/rc.sshd", false);
        assert_eq!(init.enabled_service_count(), 1);
    }
}
