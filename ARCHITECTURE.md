# DATAGRAM architecture

## Initial transport model
The first implementation is a bounded wrapper around operating-system UDP sockets. IP routing and the kernel's UDP/IP implementation remain responsibilities of the host operating system. DATAGRAM is not a replacement TCP/IP stack.

## Initial responsibilities
- Endpoint binding and lifecycle.
- Caller-owned receive buffers.
- Configurable maximum payload and read timeout.
- Peer address metadata.
- Explicit reporting of I/O errors and oversized sends.

## Future layers
- Peer identity and authenticated session establishment.
- Sequence numbers, duplicate detection and optional reliability profiles.
- Congestion and pacing behavior.
- Discovery and observability.
- Fault injection for loss, delay, duplication and reordering.

## Safety boundary
Network availability must never be a prerequisite for local emergency stop, motor interlocks, thermal trips, or other local protective functions.
