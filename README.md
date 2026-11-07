# DATAGRAM

**A research-oriented datagram networking engine.**

DATAGRAM provides a controlled networking layer for message exchange between processes and devices. Its first implementation uses operating-system UDP sockets; it does not replace IP or the operating system's TCP/IP stack.

## Design boundaries

- Own endpoint lifecycle, bounded datagram I/O, peer identity hooks, delivery metadata, and transport-level fault reporting.
- Treat UDP as a substrate, not as a guarantee of delivery, ordering, or authenticity.
- Keep payload encoding in OCTET.
- Keep remote method semantics in CALLGRAPH.
- Keep application routing and HTTP semantics in CONTRACT.

## Implemented primitives

- Bounded UDP endpoint wrapper and DGM1 envelope codec, including non-blocking receive and explicit socket mode control.
- 64-packet replay-window primitive.
- Shared AES-256-GCM payload protection with request/response nonce separation.
- Per-path congestion controller with byte-based congestion window, pacing, RTT/RTO estimation, ACK/loss feedback, and bounded outstanding-packet metadata.
- Explicit path datagram-size discovery state with single-probe validation and binary-search convergence.
- Conservative default probe ceiling of 1452 bytes; the transport adapter must confirm successful receipt of a padded probe before increasing the usable size.

## Congestion controller contract

The congestion controller is a reusable control-plane primitive, not a complete congestion-controlled transport by itself. The adapter must:
1. Create one controller per network path.
2. Call `can_send` before sending a datagram and `on_packet_sent` after a successful send.
3. Feed authenticated acknowledgements to `on_ack`.
4. Call `detect_losses` on its timer and process the returned packet IDs according to the reliability policy.
5. Use the controller's RTO and congestion window; do not treat UDP delivery as guaranteed.

It currently provides a conservative Reno-style byte controller. It does not yet implement a wire-level ACK format, retransmission scheduler, ECN, fairness testing, or a complete congestion-control standard.

## Path MTU discovery contract

`MtuDiscovery` does not manipulate platform-specific DF flags or interpret ICMP. The adapter sends a uniquely identified padded probe and calls `complete_probe` only after authenticated peer confirmation. Failed probes reduce the search ceiling. Until a larger probe succeeds, use the conservative configured baseline.

## Safety

A datagram link is not a safety controller. FireBot emergency-stop, motor interlocks, and local protection must operate independently of DATAGRAM and network availability.

## Status

Early development; no production-readiness claim. The new congestion and MTU modules are primitives awaiting integration into a complete reliable transport adapter.

## Related technologies

- [CONTRACT](https://github.com/jadhavdurvesh/contract) — API framework.
- [OCTET](https://github.com/jadhavdurvesh/octet) — binary data protocol.
- [CALLGRAPH](https://github.com/jadhavdurvesh/callgraph) — remote procedure execution.
