# DATAGRAM roadmap

- [x] Bounded UDP endpoint wrapper.
- [x] Explicit payload maximum and receive timeout.
- [x] Local loopback integration test.
- [x] DGM1 message envelope encode/decode.
- [x] 64-packet replay-window primitive.
- [x] CALLGRAPH interoperability profile documented, including its optional AES-256-GCM PSK wrapper.
- [x] Shared AES-256-GCM seal/open primitives with direction-separated nonce domain.
- [ ] Correctly detect/report datagram truncation on supported platforms.
- [ ] Non-blocking and cancellation-aware receive APIs.
- [ ] General authenticated peer/session API and key lifecycle.
- [ ] General reliability profile with acknowledgements, congestion control and MTU discovery.
- [ ] Network fault injection, cross-language conformance tests and fuzzing.
- [ ] Benchmarks on target ARM and desktop hardware.

The crate is an experimental transport component. The current encryption and retransmission profile lives in CALLGRAPH/CONTRACT adapters, not yet in the standalone Rust DATAGRAM crate.
