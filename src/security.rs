//! Shared authenticated-encryption profile for DGM1 payloads.
//! AES-256-GCM uses a 32-byte out-of-band PSK. The 96-bit nonce is the
//! 64-bit session ID followed by a 31-bit sequence and one direction bit.
use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm, Nonce};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityError { InvalidSequence, AuthenticationFailed, InvalidKey }

fn nonce(session: u64, sequence: u64, response: bool) -> Result<[u8; 12], SecurityError> {
    if sequence > 0x7fff_ffff { return Err(SecurityError::InvalidSequence); }
    let mut out = [0u8; 12];
    out[..8].copy_from_slice(&session.to_be_bytes());
    let seq = sequence as u32 | if response { 0x8000_0000 } else { 0 };
    out[8..].copy_from_slice(&seq.to_be_bytes());
    Ok(out)
}

pub fn seal_payload(key: &[u8; 32], session: u64, sequence: u64, response: bool, plaintext: &[u8]) -> Result<Vec<u8>, SecurityError> {
    let nonce_bytes = nonce(session, sequence, response)?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| SecurityError::InvalidKey)?;
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: plaintext, aad: &nonce_bytes })
        .map_err(|_| SecurityError::AuthenticationFailed)?;
    let mut out = Vec::with_capacity(4 + ciphertext.len());
    out.extend_from_slice(b"SEC1");
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn open_payload(key: &[u8; 32], session: u64, sequence: u64, response: bool, wire: &[u8]) -> Result<Vec<u8>, SecurityError> {
    if wire.len() < 20 || &wire[..4] != b"SEC1" { return Err(SecurityError::AuthenticationFailed); }
    let nonce_bytes = nonce(session, sequence, response)?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| SecurityError::InvalidKey)?;
    cipher.decrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: &wire[4..], aad: &nonce_bytes })
        .map_err(|_| SecurityError::AuthenticationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_and_authentication() {
        let key = [0x42; 32];
        let sealed = seal_payload(&key, 7, 1, false, b"octet").unwrap();
        assert_eq!(open_payload(&key, 7, 1, false, &sealed).unwrap(), b"octet");
        assert!(open_payload(&[0x24; 32], 7, 1, false, &sealed).is_err());
        assert!(open_payload(&key, 7, 1, true, &sealed).is_err());
        let mut tampered = sealed;
        *tampered.last_mut().unwrap() ^= 1;
        assert!(open_payload(&key, 7, 1, false, &tampered).is_err());
    }
    #[test]
    fn rejects_sequence_outside_nonce_domain() {
        assert_eq!(seal_payload(&[0;32], 1, 0x8000_0000, false, b"x"), Err(SecurityError::InvalidSequence));
    }
}
