//! DATAGRAM: a bounded UDP socket adapter. UDP provides no delivery guarantees.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub max_payload: usize,
    pub read_timeout: Option<Duration>,
}
impl Default for Config {
    fn default() -> Self { Self { max_payload: 1200, read_timeout: Some(Duration::from_millis(250)) } }
}

#[derive(Debug)]
pub enum DatagramError {
    Io(io::Error),
    PayloadTooLarge { actual: usize, maximum: usize },
    ReceiveBufferTooSmall { minimum: usize },
}
impl From<io::Error> for DatagramError {
    fn from(value: io::Error) -> Self { Self::Io(value) }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Received { pub length: usize, pub peer: SocketAddr }

pub struct Endpoint { socket: UdpSocket, config: Config }

impl Endpoint {
    pub fn bind(address: impl std::net::ToSocketAddrs, config: Config) -> Result<Self, DatagramError> {
        let socket = UdpSocket::bind(address)?;
        socket.set_read_timeout(config.read_timeout)?;
        Ok(Self { socket, config })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, DatagramError> { Ok(self.socket.local_addr()?) }

    pub fn send_to(&self, payload: &[u8], peer: SocketAddr) -> Result<usize, DatagramError> {
        if payload.len() > self.config.max_payload {
            return Err(DatagramError::PayloadTooLarge { actual: payload.len(), maximum: self.config.max_payload });
        }
        Ok(self.socket.send_to(payload, peer)?)
    }

    /// Receive into caller-owned memory. The buffer must be at least max_payload + 1
    /// bytes so an oversized datagram cannot be mistaken for a valid truncated one.
    pub fn receive_from(&self, output: &mut [u8]) -> Result<Received, DatagramError> {
        let minimum = self.config.max_payload.saturating_add(1);
        if output.len() < minimum {
            return Err(DatagramError::ReceiveBufferTooSmall { minimum });
        }
        let (length, peer) = self.socket.recv_from(output)?;
        if length > self.config.max_payload {
            return Err(DatagramError::PayloadTooLarge { actual: length, maximum: self.config.max_payload });
        }
        Ok(Received { length, peer })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sends_and_receives_local_datagram() {
        let config = Config { max_payload: 128, read_timeout: Some(Duration::from_secs(1)) };
        let receiver = Endpoint::bind("127.0.0.1:0", config).unwrap();
        let sender = Endpoint::bind("127.0.0.1:0", config).unwrap();
        let peer = receiver.local_addr().unwrap();
        sender.send_to(b"octet-frame", peer).unwrap();
        let mut buffer = [0u8; 129];
        let received = receiver.receive_from(&mut buffer).unwrap();
        assert_eq!(&buffer[..received.length], b"octet-frame");
    }

    #[test]
    fn rejects_oversized_send() {
        let endpoint = Endpoint::bind("127.0.0.1:0", Config { max_payload: 2, read_timeout: None }).unwrap();
        let peer = endpoint.local_addr().unwrap();
        assert!(matches!(endpoint.send_to(b"abc", peer), Err(DatagramError::PayloadTooLarge { .. })));
    }
}
