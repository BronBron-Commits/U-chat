# Research Findings

## Argon2id Selection Rationale

### Why Argon2id?

1. **Password Hashing Competition Winner** (2015)
   - Designed specifically for password hashing
   - Peer-reviewed and extensively analyzed

2. **Memory-Hard Algorithm**
   - Requires significant memory per hash computation
   - Dramatically increases cost for GPU/ASIC attackers
   - Time-memory tradeoff resistance

3. **Argon2id Variant**
   - Hybrid of Argon2i (side-channel resistant) and Argon2d (GPU resistant)
   - Best of both worlds for password hashing
   - Recommended by OWASP and IETF

### Parameter Selection

| Parameter | Our Value | OWASP Minimum | Justification |
|-----------|-----------|---------------|---------------|
| Memory    | 48 MiB    | ~19 MiB       | Future-proofing against hardware advances |
| Iterations| 3         | 2             | Additional security margin |
| Parallelism| 1        | 1             | Prevents async runtime thread starvation |

### Parallelism = 1 Decision

In async web servers (Axum/Tokio), setting parallelism > 1 would:
- Spawn multiple threads per login request
- Potentially starve the async runtime
- Create unfair scheduling under load

Single-threaded hashing allows Tokio to schedule other requests fairly.

### PHC String Format

Format: `$argon2id$v=19$m=49152,t=3,p=1$<salt>$<hash>`

Benefits:
- Self-documenting (includes all parameters)
- Forward-compatible (new params auto-parsed)
- Standard format (interoperable)
- Salt embedded (no separate column needed)

---

## WebSocket Security Rationale (Phase 3)

### Token Transmission via Sec-WebSocket-Protocol Header

**Problem**: Browser WebSocket API does not support custom Authorization headers.

**Rejected Alternatives**:

1. **URL Query Parameter** (`wss://server/ws?token=XYZ`)
   - Exposes tokens in server logs, browser history, referrer headers
   - Violates security best practices
   - Potential credential leakage

2. **Cookies**
   - Subject to CSRF attacks
   - Complex cross-origin handling
   - Not suitable for non-browser clients (IoT)

**Chosen Solution**: `Sec-WebSocket-Protocol` header

- Browser clients: `new WebSocket(url, ["bearer", token])`
- Server extracts token from header before upgrade
- Token encrypted in transit (wss/TLS)
- Not typically logged by intermediaries
- Works for both browser and non-browser clients

### Room-Based Pub/Sub Architecture

**Problem**: Scaling message broadcast to many clients.

**Naive Approach** (Rejected):
- Global list of WebSocket connections
- Loop through all connections per message
- O(N) broadcast, blocking under lock
- Doesn't scale

**Chosen Solution**: DashMap + Tokio Broadcast Channels

| Component | Purpose |
|-----------|---------|
| DashMap | Lock-free concurrent map, sharded storage |
| broadcast::channel | Efficient fan-out, internal buffering |
| Room isolation | Different rooms broadcast independently |

Benefits:
- Concurrent broadcasts without contention
- Bounded buffer (100 messages) for backpressure
- Memory freed when rooms empty
- No explicit locking per message

### Origin Validation (CSRF Protection)

**Threat**: Cross-Site WebSocket Hijacking (CSWSH)
- Attacker's web page initiates WebSocket to our server
- Browser sends victim's cookies automatically
- Attacker can send/receive messages as victim

**Mitigation**:
- Server validates `Origin` header on handshake
- Only allow known frontend origins
- Reject unexpected origins with HTTP 403

Configuration: `ALLOWED_ORIGINS` environment variable

### Resource Cleanup Strategy

**Problem**: Memory leaks from abandoned rooms.

**Solution**:
1. Track subscriber count per room (`sender.receiver_count()`)
2. On disconnect, check if room is empty
3. Remove empty rooms from DashMap
4. Dropping Sender closes all Receivers

This ensures:
- No unbounded memory growth
- Clean disconnection handling
- Proper channel cleanup

### Bounded Channel Capacity

Channel capacity: 100 messages

**Rationale**:
- Prevents memory exhaustion from slow consumers
- Oldest messages dropped if buffer full
- Acceptable for transient real-time data
- Clients should handle message loss gracefully

---

## References

- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Argon2 RFC (RFC 9106)](https://datatracker.ietf.org/doc/html/rfc9106)
- [RustCrypto argon2 crate](https://docs.rs/argon2)
- [OWASP WebSocket Security](https://cheatsheetseries.owasp.org/cheatsheets/HTML5_Security_Cheat_Sheet.html#websockets)
- [Cross-Site WebSocket Hijacking](https://christian-schneider.net/CrossSiteWebSocketHijacking.html)
- [DashMap Documentation](https://docs.rs/dashmap)
- [Tokio Broadcast Channel](https://docs.rs/tokio/latest/tokio/sync/broadcast/)
