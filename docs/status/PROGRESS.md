# Development Progress

## Phase 1: Cryptographic Hardening (Argon2id Password Hashing)

**Status**: Completed

### Completed Tasks

- [x] Created `PasswordService` with Argon2id implementation
  - File: `auth-api/src/services/auth_service.rs`
  - Parameters: 48 MiB memory, 3 iterations, parallelism = 1
  - Exceeds OWASP 2024+ minimums

- [x] Added argon2 crate dependency
  - Replaced legacy sha2 crate with argon2 v0.5
  - Added rand_core for secure salt generation

- [x] Updated handlers to use Argon2id verification
  - File: `auth-api/src/handlers.rs`
  - Constant-time password verification
  - PHC-formatted hash storage

- [x] Created database migration
  - File: `migrations/001_argon2id_password_hash.sql`
  - Expands password_hash to VARCHAR(255)
  - Removes legacy salt column (embedded in PHC format)

- [x] Comprehensive test suite
  - 7 tests covering roundtrip, unique salts, unicode, edge cases
  - Development profile for faster testing

### Security Improvements

- Memory-hard password hashing (resists GPU/ASIC attacks)
- 128-bit random salt per password (CSPRNG)
- Constant-time verification (timing attack protection)
- PHC-formatted strings (self-documenting hash format)

---

## Phase 2: Token-Gated HTTP Route Foundation with JWT

**Status**: In Progress (Background)

### Expected Components (Not Yet Committed)

- [ ] JWT token generation on successful login
- [ ] JWT validation middleware for protected routes
- [ ] Token expiration and refresh logic
- [ ] Integration with auth-api login flow

### Notes

Phase 2 is completing in the background. Phase 3 WebSocket implementation includes
JWT validation that will need to be unified with Phase 2 once committed.

---

## Phase 3: Real-Time WebSocket Fabric Hardening

**Status**: Completed

### Completed Tasks

- [x] WebSocket endpoint implementation (`GET /ws`)
  - File: `gateway-service/src/ws_handler.rs`
  - Axum WebSocketUpgrade extractor for handshake
  - Health check endpoint at `/health`

- [x] Token authentication via Sec-WebSocket-Protocol header
  - Browsers cannot set Authorization headers in WebSocket JS API
  - Token passed as subprotocol: `new WebSocket(url, ["bearer", token])`
  - Server validates JWT before completing upgrade
  - HTTP 403 returned for invalid/missing tokens

- [x] Room-based pub/sub with DashMap
  - File: `gateway-service/src/state.rs`
  - Lock-free concurrent room management
  - Room ID derived from JWT claims (user ID or custom room)
  - Automatic room creation on first client join

- [x] Tokio broadcast channels for fan-out messaging
  - Efficient message distribution to all room subscribers
  - Bounded capacity (100 messages) for backpressure
  - No explicit locking for message broadcast

- [x] Resource cleanup on disconnect
  - Forward task aborted when client disconnects
  - Empty rooms removed from DashMap
  - Memory freed when last subscriber leaves

- [x] CORS and Origin validation
  - Origin header checked against allowed origins list
  - Configurable via `ALLOWED_ORIGINS` environment variable
  - Prevents Cross-Site WebSocket Hijacking (CSWSH)

- [x] Structured logging with tracing
  - Connection/disconnection events logged
  - User and room context in log messages
  - Environment-configurable log levels

### Architecture

```
Client                    Gateway Service                    Room
  |                             |                              |
  |-- GET /ws (token in header) |                              |
  |                             |-- Validate JWT               |
  |                             |-- Check Origin               |
  |                             |-- Join/Create Room --------->|
  |<-- WebSocket Upgrade -------|                              |
  |                             |                              |
  |-- Text Message ------------>|-- Broadcast ---------------->|
  |<-- Broadcast Messages ------|<-----------------------------|
  |                             |                              |
  |-- Close ------------------->|-- Cleanup (if last) -------->|
```

### Security Improvements

- Token in header (not URL query) - avoids logging sensitive data
- JWT signature and expiration validation
- Origin checking prevents CSWSH attacks
- Room isolation - users only receive messages for their room
- TLS required in production (wss://)

### Files Added/Modified

| File | Change |
|------|--------|
| `gateway-service/Cargo.toml` | Added dashmap, tracing, tracing-subscriber |
| `gateway-service/src/main.rs` | Modular architecture, CORS, health check |
| `gateway-service/src/state.rs` | New - AppState with RoomsMap |
| `gateway-service/src/ws_handler.rs` | New - WebSocket handler with auth |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GATEWAY_BIND_ADDR` | Server bind address | `0.0.0.0:9000` |
| `JWT_SECRET` | JWT signing secret | `supersecret` |
| `ALLOWED_ORIGINS` | Comma-separated allowed origins | `http://localhost:3000,http://127.0.0.1:3000` |

---

## Pending Phase 2/3 Integration Items

The following items require Phase 2 completion before integration:

1. **EF-SEC-02**: Unified token validation - Share JWT validation logic between auth-api and gateway-service
2. **Token generation alignment** - Ensure auth-api generates tokens with claims expected by gateway-service
3. **Refresh token support** - WebSocket reconnection with new tokens on expiry
