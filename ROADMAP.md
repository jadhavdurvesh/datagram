# DATAGRAM roadmap

- [x] Bounded UDP endpoint wrapper.
- [x] Explicit payload maximum and receive timeout.
- [x] Local loopback integration test.
- [ ] Correctly detect and report datagram truncation on supported platforms.
- [ ] Add non-blocking and cancellation-aware receive APIs.
- [ ] Add packet metadata and sequence primitives.
- [ ] Add authenticated peer/session layer (separate specification).
- [ ] Add optional reliability profile with congestion controls.
- [ ] Add network fault-injection and interoperability tests.
- [ ] Benchmark on target ARM and desktop hardware.

The crate is an experimental transport component, not a secure or reliable protocol by itself.
