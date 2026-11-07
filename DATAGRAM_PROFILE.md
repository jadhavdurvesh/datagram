# DATAGRAM transport profile — draft 0.3

## Base transport
The initial engine uses OS-provided UDP over IP. UDP is message-oriented but does not guarantee delivery, ordering, uniqueness, congestion safety or peer identity.

## Implemented DGM1 envelope
The current profile uses a 24-byte fixed header:
- 4-byte magic `DGM1`;
- 1-byte version;
- 1-byte flags;
- 8-byte session identifier;
- 8-byte sequence number;
- 2-byte big-endian payload length;
- payload bytes.

The Rust crate provides bounded UDP endpoint operations, envelope encode/decode and a 64-packet replay-window primitive.

## CALLGRAPH secure datagram profile
The Go CALLGRAPH transport adapter currently layers optional AES-256-GCM protection over the OCTET frame carried in DGM1:
- `SEC1` marker followed by AES-GCM ciphertext and 16-byte tag;
- 32-byte pre-shared key configured out-of-band;
- 96-bit nonce composed of the 64-bit session identifier and 32-bit sequence/direction field;
- request and response use separate direction domains;
- encrypted packets are authenticated before consuming replay-cache capacity.

This is an interoperability profile implemented by CALLGRAPH's Go adapter and CONTRACT's Rust client. The standalone Rust DATAGRAM crate exposes the same AES-256-GCM payload seal/open primitives. CONTRACT now delegates its Rust AEAD operations to that shared crate; CALLGRAPH implements the interoperable Go counterpart.

## ACK1 acknowledgement frame — draft

The crate now defines an ACK1 frame codec. The fixed header is 24 bytes and contains magic, version, reserved flags, largest acknowledged packet number, ACK delay in microseconds, and a range count. Up to eight inclusive packet-number ranges follow (16 bytes each: first and last packet number). Ranges are newest-to-oldest, non-overlapping, non-adjacent (adjacent ranges must be merged), and the first range must end at largest_acked. Reserved bytes must be zero. The congestion controller can apply a validated ACK1 frame to its outstanding-packet and RTT state. ACK1 itself is not authenticated; the UDP adapter must authenticate the enclosing DATAGRAM before calling `apply_ack`.

## Reliability
CALLGRAPH currently implements bounded request retransmission, duplicate suppression and in-memory cached responses. This is stop-and-wait per request, not a general reliable transport. The standalone DATAGRAM crate does not yet implement retransmission or congestion control.

A production reliability profile still needs acknowledgements, congestion control, loss estimation, bounded in-flight windows, path MTU discovery and fairness.

## Key management and limitations
The current PSK mode has no key exchange, forward secrecy, certificate identity, automatic rotation or downgrade negotiation. Protect and rotate keys out-of-band; cap the number of sessions per key. A 64-bit random session identifier is used with a 31-bit per-direction sequence space. This is a research profile, not a reviewed security standard.

## Operational limits
- Enforce maximum payloads and bounded queues.
- Track receive truncation, drops, timeouts and malformed messages.
- Expose link health as uncertain and time-bounded.
- Never rely on DATAGRAM for local emergency-stop or real-time actuator control.
