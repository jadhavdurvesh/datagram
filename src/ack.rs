//! ACK1 acknowledgement frame codec for the DATAGRAM reliability profile.
//! Ranges are inclusive and ordered from newest to oldest. Adjacent ranges
//! must be coalesced; overlapping or out-of-order ranges are rejected.

pub const ACK_HEADER_LEN: usize = 24;
pub const MAX_ACK_RANGES: usize = 8;
pub const ACK_RANGE_LEN: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AckRange { pub first: u64, pub last: u64 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AckFrame { pub largest_acked: u64, pub ack_delay_micros: u32, pub ranges: Vec<AckRange> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckError { Truncated, InvalidMagic, UnsupportedVersion, InvalidLength, TooManyRanges, InvalidRanges }

impl AckFrame {
    pub fn validate(&self) -> Result<(), AckError> {
        if self.ranges.is_empty() { return Err(AckError::InvalidRanges); }
        if self.ranges.len() > MAX_ACK_RANGES { return Err(AckError::TooManyRanges); }
        if self.ranges[0].first > self.ranges[0].last || self.ranges[0].last != self.largest_acked { return Err(AckError::InvalidRanges); }
        for pair in self.ranges.windows(2) {
            let newer = pair[0]; let older = pair[1];
            if older.first > older.last || older.last.saturating_add(1) >= newer.first { return Err(AckError::InvalidRanges); }
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, AckError> {
        self.validate()?;
        let mut out = vec![0u8; ACK_HEADER_LEN + self.ranges.len() * ACK_RANGE_LEN];
        out[..4].copy_from_slice(b"ACK1"); out[4] = 1;
        out[8..16].copy_from_slice(&self.largest_acked.to_be_bytes());
        out[16..20].copy_from_slice(&self.ack_delay_micros.to_be_bytes());
        out[20] = self.ranges.len() as u8;
        for (index, range) in self.ranges.iter().enumerate() {
            let offset = ACK_HEADER_LEN + index * ACK_RANGE_LEN;
            out[offset..offset + 8].copy_from_slice(&range.first.to_be_bytes());
            out[offset + 8..offset + 16].copy_from_slice(&range.last.to_be_bytes());
        }
        Ok(out)
    }

    pub fn decode(input: &[u8]) -> Result<Self, AckError> {
        if input.len() < ACK_HEADER_LEN { return Err(AckError::Truncated); }
        if &input[..4] != b"ACK1" { return Err(AckError::InvalidMagic); }
        if input[4] != 1 { return Err(AckError::UnsupportedVersion); }
        if input[5..8] != [0, 0, 0] || input[21..24] != [0, 0, 0] { return Err(AckError::InvalidLength); }
        let count = input[20] as usize;
        if count == 0 { return Err(AckError::InvalidRanges); }
        if count > MAX_ACK_RANGES { return Err(AckError::TooManyRanges); }
        if input.len() != ACK_HEADER_LEN + count * ACK_RANGE_LEN { return Err(AckError::InvalidLength); }
        let largest_acked = u64::from_be_bytes(input[8..16].try_into().unwrap());
        let ack_delay_micros = u32::from_be_bytes(input[16..20].try_into().unwrap());
        let mut ranges = Vec::with_capacity(count);
        for index in 0..count {
            let offset = ACK_HEADER_LEN + index * ACK_RANGE_LEN;
            ranges.push(AckRange {
                first: u64::from_be_bytes(input[offset..offset + 8].try_into().unwrap()),
                last: u64::from_be_bytes(input[offset + 8..offset + 16].try_into().unwrap()),
            });
        }
        let frame = Self { largest_acked, ack_delay_micros, ranges };
        frame.validate()?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trips_ack_ranges() {
        let frame = AckFrame { largest_acked: 100, ack_delay_micros: 250, ranges: vec![AckRange { first: 98, last: 100 }, AckRange { first: 90, last: 94 }] };
        let wire = frame.encode().unwrap();
        assert_eq!(AckFrame::decode(&wire).unwrap(), frame);
    }
    #[test]
    fn rejects_truncated_and_bad_length_frames() {
        assert_eq!(AckFrame::decode(b"ACK1"), Err(AckError::Truncated));
        let frame = AckFrame { largest_acked: 1, ack_delay_micros: 0, ranges: vec![AckRange { first: 1, last: 1 }] };
        let mut wire = frame.encode().unwrap(); wire.push(0);
        assert_eq!(AckFrame::decode(&wire), Err(AckError::InvalidLength));
    }
    #[test]
    fn rejects_nonzero_reserved_bytes() {
        let frame = AckFrame { largest_acked: 1, ack_delay_micros: 0, ranges: vec![AckRange { first: 1, last: 1 }] };
        let mut wire = frame.encode().unwrap(); wire[6] = 1;
        assert_eq!(AckFrame::decode(&wire), Err(AckError::InvalidLength));
    }

    #[test]
    fn rejects_overlapping_or_unordered_ranges() {
        let frame = AckFrame { largest_acked: 10, ack_delay_micros: 0, ranges: vec![AckRange { first: 8, last: 10 }, AckRange { first: 9, last: 9 }] };
        assert_eq!(frame.encode(), Err(AckError::InvalidRanges));
    }
    #[test]
    fn rejects_more_than_eight_ranges() {
        let ranges = (0..9).map(|n| AckRange { first: 100 - n * 3, last: 100 - n * 3 }).collect();
        let frame = AckFrame { largest_acked: 100, ack_delay_micros: 0, ranges };
        assert_eq!(frame.encode(), Err(AckError::TooManyRanges));
    }
}