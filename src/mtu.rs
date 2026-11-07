//! Explicit path datagram-size discovery state.
//!
//! The module does not set platform-specific IP "don't fragment" flags or
//! interpret ICMP itself. A transport adapter sends a padded probe and reports
//! whether it arrived intact. The default 1200-byte UDP payload is conservative
//! for common IPv6 minimum-MTU paths; larger sizes must be probed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MtuConfig {
    pub initial_datagram_size: usize,
    pub minimum_datagram_size: usize,
    pub maximum_datagram_size: usize,
}

impl Default for MtuConfig {
    fn default() -> Self {
        Self {
            initial_datagram_size: 1200,
            minimum_datagram_size: 576,
            maximum_datagram_size: 1472,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MtuError {
    InvalidConfig,
    ProbeInProgress,
    StaleProbe,
    ProbeTooSmall,
    ProbeTooLarge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MtuProbe {
    pub id: u64,
    pub datagram_size: usize,
}

#[derive(Debug)]
pub struct MtuDiscovery {
    config: MtuConfig,
    current: usize,
    upper_bound: usize,
    next_id: u64,
    active: Option<MtuProbe>,
}

impl MtuDiscovery {
    pub fn new(config: MtuConfig) -> Result<Self, MtuError> {
        if config.minimum_datagram_size == 0
            || config.minimum_datagram_size > config.initial_datagram_size
            || config.initial_datagram_size > config.maximum_datagram_size
            || config.maximum_datagram_size > 65_507
        {
            return Err(MtuError::InvalidConfig);
        }
        Ok(Self { config, current: config.initial_datagram_size, upper_bound: config.maximum_datagram_size, next_id: 1, active: None })
    }

    pub fn current_datagram_size(&self) -> usize { self.current }

    /// Return the next binary-search probe size. Only one probe may be active.
    pub fn begin_probe(&mut self) -> Result<Option<MtuProbe>, MtuError> {
        if self.active.is_some() { return Err(MtuError::ProbeInProgress); }
        if self.current >= self.upper_bound { return Ok(None); }
        let gap = self.upper_bound - self.current;
        let size = self.current + (gap + 1) / 2;
        let probe = MtuProbe { id: self.next_id, datagram_size: size };
        self.next_id = self.next_id.checked_add(1).unwrap_or(1);
        self.active = Some(probe);
        Ok(Some(probe))
    }

    /// Report success only after the peer confirms receipt of the exact probe.
    pub fn complete_probe(&mut self, probe: MtuProbe, succeeded: bool) -> Result<usize, MtuError> {
        if self.active != Some(probe) { return Err(MtuError::StaleProbe); }
        self.active = None;
        if probe.datagram_size <= self.current { return Err(MtuError::ProbeTooSmall); }
        if probe.datagram_size > self.upper_bound { return Err(MtuError::ProbeTooLarge); }
        if succeeded {
            self.current = probe.datagram_size;
        } else {
            self.upper_bound = probe.datagram_size.saturating_sub(1).max(self.current);
        }
        Ok(self.current)
    }

    /// Maximum application payload after accounting for all wire headers/tags.
    pub fn payload_budget(&self, overhead_bytes: usize) -> usize {
        self.current.saturating_sub(overhead_bytes)
    }

    pub fn is_converged(&self) -> bool { self.current >= self.upper_bound }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes_larger_than_conservative_baseline_and_converges() {
        let mut mtu = MtuDiscovery::new(MtuConfig::default()).unwrap();
        assert_eq!(mtu.current_datagram_size(), 1200);
        let p = mtu.begin_probe().unwrap().unwrap();
        assert_eq!(p.datagram_size, 1336);
        mtu.complete_probe(p, true).unwrap();
        let p = mtu.begin_probe().unwrap().unwrap();
        assert_eq!(p.datagram_size, 1404);
        mtu.complete_probe(p, false).unwrap();
        while let Some(p) = mtu.begin_probe().unwrap() {
            mtu.complete_probe(p, true).unwrap();
        }
        assert!(mtu.is_converged());
        assert!(mtu.current_datagram_size() <= 1403);
    }

    #[test]
    fn allows_only_one_probe_and_rejects_stale_completion() {
        let mut mtu = MtuDiscovery::new(MtuConfig::default()).unwrap();
        let p = mtu.begin_probe().unwrap().unwrap();
        assert_eq!(mtu.begin_probe(), Err(MtuError::ProbeInProgress));
        let forged = MtuProbe { id: p.id + 1, datagram_size: p.datagram_size };
        assert_eq!(mtu.complete_probe(forged, true), Err(MtuError::StaleProbe));
        mtu.complete_probe(p, false).unwrap();
    }

    #[test]
    fn computes_payload_budget_and_validates_bounds() {
        let mtu = MtuDiscovery::new(MtuConfig::default()).unwrap();
        assert_eq!(mtu.payload_budget(48), 1152);
        assert_eq!(MtuDiscovery::new(MtuConfig { initial_datagram_size: 1500, ..MtuConfig::default() }), Err(MtuError::InvalidConfig));
    }
}
