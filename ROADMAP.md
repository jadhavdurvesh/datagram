# DATAGRAM roadmap

- [x] Bounded UDP endpoint wrapper.
- [x] Explicit payload maximum and receive timeout.
- [x] Local loopback integration test.
- [x] DGM1 message envelope encode/decode.
- [x] 64-packet replay-window primitive.
- [x] CALLGRAPH interoperability profile documented, including its optional AES-256-GCM PSK wrapper.
- [x] Shared AES-256-GCM seal/open primitives with direction-separated nonce domain.
- [ ] Correctly detect/report datagram truncation on supported platforms.
- [x] Non-blocking receive API and explicit socket mode control.
- [ ] Cancellation-aware asynchronous receive integration.
- [ ] General authenticated peer/session API and key lifecycle.
- [x] Define ACK1 wire frame codec with bounded ACK ranges.
- [ ] Integrate ACK generation, authentication and state application into the UDP adapter.
- [x] Reusable per-path congestion controller with pacing, ACK/loss feedback, RTT/RTO estimation and bounded in-flight metadata.
- [x] Explicit path MTU discovery state machine with authenticated-probe contract.
- [ ] Integrate controller and MTU state into a complete transport adapter.
- [ ] Network fault injection, cross-language conformance tests and fuzzing.
- [ ] Benchmarks on target ARM and desktop hardware.

The crate is an experimental transport component. The current encryption and retransmission profile lives in CALLGRAPH/CONTRACT adapters, not yet in the standalone Rust DATAGRAM crate.
