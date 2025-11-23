# Todo / Development Tasks

## Current Sprint

### Phase 2/3 Integration (Blocked - Awaiting Phase 2 Commit)

- [ ] **INT-01**: Unify JWT validation logic
  - Share token validation between auth-api and gateway-service
  - Create shared `jwt-common` crate or module
  - Ensure consistent secret handling across services

- [ ] **INT-02**: Align token claims structure
  - auth-api must generate tokens with `sub` (username) and `exp` claims
  - Add optional `room` claim for custom room assignment
  - Document expected JWT payload structure

- [ ] **INT-03**: WebSocket reconnection flow
  - Handle token expiry during active WebSocket sessions
  - Client-side token refresh before reconnection
  - Graceful disconnect with retry logic

### Optional Fast-Track Enhancements

- [ ] **EF-SEC-01**: Rate limiting on login endpoint
  - Add Tower rate limiter or governor crate
  - Limit login attempts per IP/account per minute
  - Prevents DoS via expensive hash computations

- [ ] **EF-SEC-02**: Structured token validation for WebSockets
  - Replace stub `validate_token` with full JWT verification
  - Verify signature, expiration, and required claims
  - Log validation failures for security monitoring

- [ ] **EF-DEVX-01**: Environment-based password cost selection
  - Implement config-based parameter selection
  - Use reduced params in development, full params in production
  - Environment variable or feature flag driven

- [ ] **EF-OBS-01**: Structured logging for auth and WS events
  - JSON-formatted log output for production
  - Include user context, room, IP in WebSocket logs
  - Avoid logging sensitive data (tokens, passwords)

- [ ] **EF-OBS-02**: Basic metrics for WebSockets
  - Active connection count gauge
  - Message rate histogram
  - Room count and subscriber distribution
  - Prometheus-compatible endpoint

## Backlog

### Security Enhancements

- [ ] Implement password change endpoint
- [ ] Add password reset flow with secure tokens
- [ ] Implement account lockout after failed attempts
- [ ] Add audit logging for authentication events

### Migration Tasks

- [ ] Create legacy hash migration flow
  - Detect old SHA256 hash format
  - Re-hash on successful login
  - Gradual migration without forced resets

### Infrastructure

- [ ] Add health check endpoints (Done for gateway-service)
- [ ] Implement proper error handling
- [ ] Add structured logging (tracing crate) - Done for gateway-service
- [ ] Set up CI/CD pipeline with security scanning

### Phase 3 Optional Enhancements (From Spec)

- [ ] **EF-CHAT-01**: Room message history endpoint
  - REST API: `GET /rooms/{id}/messages?limit=N`
  - Store messages in database on broadcast
  - Index on (room_id, timestamp)
  - Provides chat history / audit trail

- [ ] **EF-CHAT-02**: Typing indicator broadcast
  - Ephemeral "typing" notifications via WebSocket
  - JSON message type: `{type: "typing", user: X, state: "start|stop"}`
  - Not persisted to database
  - Nice-to-have for real-time collaboration UX

## Completed

### Phase 1

- [x] Argon2id password hashing implementation
- [x] Database migration for PHC-format hashes
- [x] Test suite for password service

### Phase 3

- [x] WebSocket endpoint (`GET /ws`) in gateway-service
- [x] Token authentication via Sec-WebSocket-Protocol header
- [x] Room-based pub/sub with DashMap
- [x] Tokio broadcast channels for message fan-out
- [x] CORS and Origin validation
- [x] Resource cleanup on disconnect
- [x] Structured logging with tracing
- [x] Health check endpoint (`GET /health`)

## Notes

### Phase 2 Dependency

Phase 2 (Token-Gated HTTP Route Foundation) is completing in the background but not yet committed. The following Phase 3 items are **blocked** until Phase 2 integration:

1. Real JWT tokens from auth-api (currently gateway-service validates tokens independently)
2. Shared JWT secret configuration
3. Token refresh/renewal flow

### WebSocket Client Usage

To connect to the WebSocket endpoint:

```javascript
// Browser client
const token = "your-jwt-token";
const ws = new WebSocket("wss://gateway/ws", ["bearer", token]);

ws.onopen = () => console.log("Connected");
ws.onmessage = (e) => console.log("Received:", e.data);
ws.send("Hello room!");
```

```rust
// Rust client (e.g., IoT device)
// Use tungstenite with Sec-WebSocket-Protocol header
```
