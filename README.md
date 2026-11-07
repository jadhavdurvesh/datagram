# DATAGRAM

**A research-oriented datagram networking engine.**

DATAGRAM provides a controlled networking layer for message exchange between processes and devices. Its first implementation uses operating-system UDP sockets; it does not replace IP or the operating system's TCP/IP stack.

## Design boundaries

- Own endpoint lifecycle, bounded datagram I/O, peer identity hooks, delivery metadata, and transport-level fault reporting.
- Treat UDP as a substrate, not as a guarantee of delivery, ordering, or authenticity.
- Keep payload encoding in OCTET.
- Keep remote method semantics in CALLGRAPH.
- Keep application routing and HTTP semantics in CONTRACT.

## Initial implementation

- Language: Rust.
- Initial scope: safe UDP endpoint wrapper, explicit payload limits, receive metadata, and deterministic local tests.
- Later work: optional reliability layer, peer discovery, congestion behavior, authentication integration, and observability—each separately specified and tested.

## Safety

A datagram link is not a safety controller. FireBot emergency-stop, motor interlocks, and local protection must operate independently of DATAGRAM and network availability.

## Status

Early development; no production-readiness claim.

## Related technologies

- [CONTRACT](https://github.com/jadhavdurvesh/contract) — API framework.
- [OCTET](https://github.com/jadhavdurvesh/octet) — binary data protocol.
- [CALLGRAPH](https://github.com/jadhavdurvesh/callgraph) — remote procedure execution.
