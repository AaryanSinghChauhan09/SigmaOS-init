//! SigmaOS-init — PID 1 service orchestration.
//!
//! Two layers, mirroring systemd + sysvinit:
//! - [`InitSupervisor`]: rc-script service registry (kept API).
//! - [`UnitGraph`]: systemd-style dependency graph — units declare
//!   `After=`, `Requires=`, `Wants=`; startup order is a topological
//!   sort; missing hard deps and dependency cycles fail fast.

use std::collections::{BTreeMap, BTreeSet};

/// Original rc-service model (kept for API stability).
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
    pub fn new() -> Self {
        Self::default()
    }
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

// ---------------------------------------------------------------------------
// systemd-style unit dependency graph
// ---------------------------------------------------------------------------

/// Dependency kinds — systemd directive analogues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepKind {
    /// Requires=: hard dependency — target fails if dep fails.
    Requires,
    /// Wants=: soft dependency — order only, failure tolerated.
    Wants,
    /// After=:/Before=: ordering constraint without requirement.
    After,
    Before,
}

/// A systemd unit analogue.
#[derive(Debug, Clone, Default)]
pub struct Unit {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    /// Unit names this unit depends on, with kind.
    pub deps: Vec<(String, DepKind)>,
}

impl Unit {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }
    pub fn describe(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }
    /// Builder-style dep declaration.
    pub fn dep(mut self, target: impl Into<String>, kind: DepKind) -> Self {
        self.deps.push((target.into(), kind));
        self
    }
}

/// Startup plan error — fail-fast on bad graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitError {
    DuplicateUnit(String),
    UnknownUnit(String),
    Cycle(Vec<String>),
    DisabledDependency { unit: String, dep: String },
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitError::DuplicateUnit(u) => write!(f, "duplicate unit: {u}"),
            UnitError::UnknownUnit(u) => write!(f, "unknown unit referenced: {u}"),
            UnitError::Cycle(path) => write!(f, "dependency cycle: {}", path.join(" -> ")),
            UnitError::DisabledDependency { unit, dep } => {
                write!(f, "unit {unit} requires disabled unit {dep}")
            }
        }
    }
}
impl std::error::Error for UnitError {}

/// systemd.unit analogue: registered units + computed start order.
#[derive(Default)]
pub struct UnitGraph {
    units: BTreeMap<String, Unit>,
}

impl UnitGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, unit: Unit) -> Result<(), UnitError> {
        if self.units.contains_key(&unit.name) {
            return Err(UnitError::DuplicateUnit(unit.name));
        }
        self.units.insert(unit.name.clone(), unit);
        Ok(())
    }

    /// Compute the ordered startup list (topological sort).
    ///
    /// Edge direction: dep -> unit (dep starts first). `Requires` and
    /// `Wants` pull the dep earlier; `After`/`Before` impose order only.
    /// Deterministic: ties broken by unit name.
    pub fn startup_order(&self) -> Result<Vec<String>, UnitError> {
        // Build edges: for unit U depending on D, add edge D -> U.
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut indegree: BTreeMap<String, usize> = BTreeMap::new();
        for name in self.units.keys() {
            edges.entry(name.clone()).or_default();
            indegree.insert(name.clone(), 0);
        }
        for (name, unit) in &self.units {
            for (dep, kind) in &unit.deps {
                if !self.units.contains_key(dep) {
                    return Err(UnitError::UnknownUnit(dep.clone()));
                }
                match kind {
                    // dep must start before this unit: edge dep -> unit.
                    DepKind::Requires | DepKind::Wants | DepKind::After => {
                        edges.entry(dep.clone()).or_default().insert(name.clone());
                        *indegree.get_mut(name).unwrap() += 1;
                    }
                    // Before=: this unit must start before dep: edge unit -> dep.
                    DepKind::Before => {
                        edges.entry(name.clone()).or_default().insert(dep.clone());
                        *indegree.get_mut(dep).unwrap() += 1;
                    }
                }
            }
        }
        // Kahn's algorithm with name-ordered tie-break (deterministic).
        let mut ready: BTreeSet<String> = indegree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(n) = ready.iter().next().cloned() {
            ready.remove(&n);
            order.push(n.clone());
            if let Some(nexts) = edges.get(&n) {
                for m in nexts {
                    let d = indegree.get_mut(m).unwrap();
                    *d -= 1;
                    if *d == 0 {
                        ready.insert(m.clone());
                    }
                }
            }
        }
        if order.len() != self.units.len() {
            // Find a cycle member for the error.
            let stuck: Vec<String> = indegree
                .iter()
                .filter(|(_, &d)| d > 0)
                .map(|(n, _)| n.clone())
                .collect();
            return Err(UnitError::Cycle(stuck));
        }
        Ok(order)
    }

    /// startup_order restricted to enabled units; hard-required disabled
    /// deps are rejected.
    pub fn enabled_startup_order(&self) -> Result<Vec<String>, UnitError> {
        let order = self.startup_order()?;
        let enabled: BTreeSet<&str> = self
            .units
            .values()
            .filter(|u| u.enabled)
            .map(|u| u.name.as_str())
            .collect();
        for name in &order {
            if !enabled.contains(name.as_str()) {
                continue;
            }
            let unit = &self.units[name];
            for (dep, kind) in &unit.deps {
                if *kind == DepKind::Requires && !enabled.contains(dep.as_str()) {
                    return Err(UnitError::DisabledDependency {
                        unit: name.clone(),
                        dep: dep.clone(),
                    });
                }
            }
        }
        Ok(order.into_iter().filter(|n| enabled.contains(n.as_str())).collect())
    }

    pub fn get(&self, name: &str) -> Option<&Unit> {
        self.units.get(name)
    }
    pub fn len(&self) -> usize {
        self.units.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_rc_services() {
        let mut init = InitSupervisor::new();
        init.register_rc("rc.network", "/etc/rc.d/rc.network", true);
        init.register_rc("rc.sshd", "/etc/rc.d/rc.sshd", false);
        assert_eq!(init.enabled_service_count(), 1);
    }

    #[test]
    fn dependency_order_respected() {
        // network.target wants network-pre.target; sshd requires network.target.
        let mut g = UnitGraph::new();
        g.add(Unit::new("sshd").enable().dep("network.target", DepKind::Requires))
            .unwrap();
        g.add(Unit::new("network.target").enable().dep("network-pre.target", DepKind::Wants))
            .unwrap();
        g.add(Unit::new("network-pre.target").enable()).unwrap();
        let order = g.startup_order().unwrap();
        let pos = |n: &str| order.iter().position(|x| x == n).unwrap();
        assert!(pos("network-pre.target") < pos("network.target"));
        assert!(pos("network.target") < pos("sshd"));
    }

    #[test]
    fn deterministic_tie_break() {
        let mut g = UnitGraph::new();
        g.add(Unit::new("b")).unwrap();
        g.add(Unit::new("a")).unwrap();
        g.add(Unit::new("c")).unwrap();
        assert_eq!(g.startup_order().unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn cycle_fails_fast() {
        let mut g = UnitGraph::new();
        g.add(Unit::new("a").dep("b", DepKind::Requires)).unwrap();
        g.add(Unit::new("b").dep("a", DepKind::Requires)).unwrap();
        assert!(matches!(g.startup_order(), Err(UnitError::Cycle(_))));
    }

    #[test]
    fn unknown_dep_fails() {
        let mut g = UnitGraph::new();
        g.add(Unit::new("a").dep("ghost", DepKind::Wants)).unwrap();
        assert!(matches!(g.startup_order(), Err(UnitError::UnknownUnit(_))));
    }

    #[test]
    fn disabled_hard_dep_rejected() {
        let mut g = UnitGraph::new();
        g.add(Unit::new("a").enable().dep("b", DepKind::Requires)).unwrap();
        g.add(Unit::new("b")).unwrap(); // disabled by default
        assert!(matches!(
            g.enabled_startup_order(),
            Err(UnitError::DisabledDependency { .. })
        ));
    }

    #[test]
    fn duplicate_unit_rejected() {
        let mut g = UnitGraph::new();
        g.add(Unit::new("a")).unwrap();
        assert!(matches!(g.add(Unit::new("a")), Err(UnitError::DuplicateUnit(_))));
    }
}
