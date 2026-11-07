# DATAGRAM transport profile — draft 0.2

## Base transport
The initial engine uses OS-provided UDP over IP. UDP is message-oriented but does not guarantee delivery, ordering, uniqueness, congestion safety or peer identity.

## Message metadata profile
An application-level DATAGRAM envelope may carry:
- protocol version;
- sender node identifier;
- session identifier;
- sequence number;
- message type;
- expiry/deadline;
- payload length and payload.

This envelope is a future profile, not yet implemented by the current UDP wrapper.

## Reliability
Reliability is opt-in and must be designed as a distinct profile. It requires acknowledgements, bounded retransmission, duplicate suppression, expiry, congestion control and a clear maximum in-flight window. Retransmission must not be implemented without congestion behavior.

## Security
Use authenticated session establishment and message integrity before accepting control-relevant peer data. Source IP/port alone is not identity. Define key rotation, replay windows and downgrade handling before production use.

## Operational limits
- Enforce maximum payloads and bounded queues.
- Track receive truncation, drops, timeouts and malformed messages.
- Expose link health as uncertain and time-bounded.
- Never rely on DATAGRAM for local emergency-stop or real-time actuator control.
