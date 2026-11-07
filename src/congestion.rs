//! Per-path congestion and pacing controller.
//!
//! This is a conservative Reno-style byte controller, not a complete Internet
//! congestion-control implementation. The caller supplies packet ACK/loss
//! signals and must keep one controller per network path.
use std::{collections::BTreeMap, time::{Duration, Instant}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CongestionConfig {
    pub max_datagram_size: usize,
    pub initial_window_datagrams: usize,
    pub minimum_window_datagrams: usize,
    pub maximum_window_bytes: usize,
    pub initial_rtt: Duration,
    pub minimum_rto: Duration,
    pub maximum_rto: Duration,
}

impl Default for CongestionConfig {
    fn default() -> Self {
        Self {
            max_datagram_size: 1200,
            initial_window_datagrams: 10,
            minimum_window_datagrams: 2,
            maximum_window_bytes: 16 * 1024 * 1024,
            initial_rtt: Duration::from_millis(100),
            minimum_rto: Duration::from_millis(200),
            maximum_rto: Duration::from_secs(60),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CongestionError {
    InvalidConfig,
    EmptyPacket,
    PacketTooLarge,
    DuplicatePacketId,
    UnknownPacketId,
    CongestionWindowFull,
    PacingLimited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CongestionSnapshot {
    pub congestion_window_bytes: usize,
    pub slow_start_threshold_bytes: usize,
    pub bytes_in_flight: usize,
    pub smoothed_rtt: Duration,
    pub rtt_variance: Duration,
    pub retransmission_timeout: Duration,
    pub outstanding_packets: usize,
}

#[derive(Debug)]
struct SentPacket {
    bytes: usize,
    sent_at: Instant,
}

#[derive(Debug)]
pub struct CongestionController {
    config: CongestionConfig,
    congestion_window: usize,
    slow_start_threshold: usize,
    bytes_in_flight: usize,
    smoothed_rtt: Duration,
    rtt_variance: Duration,
    retransmission_timeout: Duration,
    next_send_at: Instant,
    largest_sent: u64,
    recovery_end: u64,
    outstanding: BTreeMap<u64, SentPacket>,
}

impl CongestionController {
    pub fn new(config: CongestionConfig, now: Instant) -> Result<Self, CongestionError> {
        if config.max_datagram_size == 0
            || config.initial_window_datagrams < config.minimum_window_datagrams
            || config.minimum_window_datagrams == 0
            || config.maximum_window_bytes < config.max_datagram_size.saturating_mul(config.minimum_window_datagrams)
            || config.initial_rtt.is_zero()
            || config.minimum_rto.is_zero()
            || config.maximum_rto < config.minimum_rto
        {
            return Err(CongestionError::InvalidConfig);
        }
        let minimum = config.max_datagram_size.saturating_mul(config.minimum_window_datagrams);
        let initial = config.max_datagram_size
            .saturating_mul(config.initial_window_datagrams)
            .min(config.maximum_window_bytes);
        Ok(Self {
            config,
            congestion_window: initial.max(minimum),
            slow_start_threshold: config.maximum_window_bytes,
            bytes_in_flight: 0,
            smoothed_rtt: config.initial_rtt,
            rtt_variance: config.initial_rtt / 2,
            retransmission_timeout: config.initial_rtt
                .saturating_add(config.initial_rtt)
                .clamp(config.minimum_rto, config.maximum_rto),
            next_send_at: now,
            largest_sent: 0,
            recovery_end: 0,
            outstanding: BTreeMap::new(),
        })
    }

    pub fn can_send(&self, bytes: usize, now: Instant) -> Result<(), CongestionError> {
        if bytes == 0 { return Err(CongestionError::EmptyPacket); }
        if bytes > self.config.max_datagram_size { return Err(CongestionError::PacketTooLarge); }
        if bytes > self.congestion_window.saturating_sub(self.bytes_in_flight) {
            return Err(CongestionError::CongestionWindowFull);
        }
        if now < self.next_send_at { return Err(CongestionError::PacingLimited); }
        Ok(())
    }

    pub fn on_packet_sent(&mut self, packet_id: u64, bytes: usize, now: Instant) -> Result<(), CongestionError> {
        self.can_send(bytes, now)?;
        if self.outstanding.contains_key(&packet_id) { return Err(CongestionError::DuplicatePacketId); }
        self.outstanding.insert(packet_id, SentPacket { bytes, sent_at: now });
        self.bytes_in_flight = self.bytes_in_flight.saturating_add(bytes);
        self.largest_sent = self.largest_sent.max(packet_id);

        // Pace at approximately cwnd / SRTT. A 1ms floor avoids a tight
        // userspace send loop when RTT estimates are unusually small.
        let pacing = self.smoothed_rtt
            .mul_f64(bytes as f64 / self.congestion_window.max(1) as f64)
            .max(Duration::from_millis(1));
        self.next_send_at = now + pacing;
        Ok(())
    }

    pub fn on_ack(&mut self, packet_id: u64, now: Instant) -> Result<(), CongestionError> {
        let packet = self.outstanding.remove(&packet_id).ok_or(CongestionError::UnknownPacketId)?;
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(packet.bytes);
        let sample = now.saturating_duration_since(packet.sent_at);
        if !sample.is_zero() {
            if self.smoothed_rtt == self.config.initial_rtt && self.rtt_variance == self.config.initial_rtt / 2 {
                self.smoothed_rtt = sample;
                self.rtt_variance = sample / 2;
            } else {
                let difference = self.smoothed_rtt.abs_diff(sample);
                self.rtt_variance = self.rtt_variance.mul_f64(0.75) + difference.mul_f64(0.25);
                self.smoothed_rtt = self.smoothed_rtt.mul_f64(0.875) + sample.mul_f64(0.125);
            }
            let rto = self.smoothed_rtt + self.rtt_variance.mul_f64(4.0);
            self.retransmission_timeout = rto.clamp(self.config.minimum_rto, self.config.maximum_rto);
        }

        if self.congestion_window < self.slow_start_threshold {
            self.congestion_window = self.congestion_window
                .saturating_add(packet.bytes)
                .min(self.config.maximum_window_bytes);
        } else {
            let increase = self.config.max_datagram_size
                .saturating_mul(packet.bytes)
                / self.congestion_window.max(1);
            self.congestion_window = self.congestion_window
                .saturating_add(increase.max(1))
                .min(self.config.maximum_window_bytes);
        }
        Ok(())
    }

    pub fn on_loss(&mut self, packet_id: u64) -> Result<(), CongestionError> {
        let packet = self.outstanding.remove(&packet_id).ok_or(CongestionError::UnknownPacketId)?;
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(packet.bytes);
        // One multiplicative decrease per recovery epoch, not one per packet
        // in the same loss burst.
        if packet_id > self.recovery_end {
            let minimum = self.config.max_datagram_size
                .saturating_mul(self.config.minimum_window_datagrams);
            self.slow_start_threshold = (self.congestion_window / 2).max(minimum);
            self.congestion_window = self.slow_start_threshold;
            self.recovery_end = self.largest_sent;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> CongestionSnapshot {
        CongestionSnapshot {
            congestion_window_bytes: self.congestion_window,
            slow_start_threshold_bytes: self.slow_start_threshold,
            bytes_in_flight: self.bytes_in_flight,
            smoothed_rtt: self.smoothed_rtt,
            rtt_variance: self.rtt_variance,
            retransmission_timeout: self.retransmission_timeout,
            outstanding_packets: self.outstanding.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_bounded_initial_window_and_paces_sends() {
        let now = Instant::now();
        let mut cc = CongestionController::new(CongestionConfig::default(), now).unwrap();
        assert_eq!(cc.snapshot().congestion_window_bytes, 12_000);
        cc.on_packet_sent(1, 1200, now).unwrap();
        assert_eq!(cc.can_send(1200, now), Err(CongestionError::PacingLimited));
        assert_eq!(cc.snapshot().bytes_in_flight, 1200);
    }

    #[test]
    fn ack_releases_flight_and_updates_rtt_and_window() {
        let now = Instant::now();
        let mut cc = CongestionController::new(CongestionConfig::default(), now).unwrap();
        let before = cc.snapshot().congestion_window_bytes;
        cc.on_packet_sent(1, 1200, now).unwrap();
        cc.on_ack(1, now + Duration::from_millis(80)).unwrap();
        let after = cc.snapshot();
        assert_eq!(after.bytes_in_flight, 0);
        assert_eq!(after.smoothed_rtt, Duration::from_millis(80));
        assert!(after.congestion_window_bytes > before);
    }

    #[test]
    fn loss_halves_window_once_per_recovery_epoch() {
        let now = Instant::now();
        let mut cc = CongestionController::new(CongestionConfig::default(), now).unwrap();
        cc.on_packet_sent(1, 1200, now).unwrap();
        cc.on_packet_sent(2, 1200, now + Duration::from_millis(100)).unwrap();
        cc.on_loss(1).unwrap();
        let reduced = cc.snapshot().congestion_window_bytes;
        cc.on_loss(2).unwrap();
        assert_eq!(cc.snapshot().congestion_window_bytes, reduced);
        assert_eq!(reduced, 6000);
    }

    #[test]
    fn rejects_invalid_packet_ids_and_sizes() {
        let now = Instant::now();
        let mut cc = CongestionController::new(CongestionConfig::default(), now).unwrap();
        assert_eq!(cc.on_packet_sent(1, 0, now), Err(CongestionError::EmptyPacket));
        assert_eq!(cc.on_packet_sent(1, 1201, now), Err(CongestionError::PacketTooLarge));
        cc.on_packet_sent(1, 100, now).unwrap();
        assert_eq!(cc.on_packet_sent(1, 100, now + Duration::from_secs(1)), Err(CongestionError::DuplicatePacketId));
        assert_eq!(cc.on_ack(99, now), Err(CongestionError::UnknownPacketId));
    }
}
