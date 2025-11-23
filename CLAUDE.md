# Claude Code Instructions for Unhidra

## Documentation Maintenance

**IMPORTANT**: Keep the `docs/status/` folder updated on each significant change:

1. **Progress Tracking** (`docs/status/PROGRESS.md`) - Update with completed tasks and milestones
2. **Todo/Tasks** (`docs/status/TODO.md`) - Maintain current and future development tasks
3. **Research Findings** (`docs/status/RESEARCH.md`) - Document research, findings, and technical decisions
4. **Deployment** (`docs/status/DEPLOYMENT.md`) - Deployment guides and configuration notes

## Project Structure

- `auth-api/` - HTTP-based authentication API (Argon2id password hashing)
- `auth-service/` - WebSocket-based auth service
- `gateway-service/` - WebSocket gateway with token validation and room-based pub/sub
- `chat-service/` - Chat functionality
- `presence-service/` - User presence tracking
- `history-service/` - Chat history
- `migrations/` - Database migration scripts

## Security Guidelines

- Use Argon2id for all password hashing (see `auth-api/src/services/auth_service.rs`)
- Never commit secrets or credentials
- Follow OWASP security best practices
- Use constant-time comparisons for sensitive data
- WebSocket tokens must use Sec-WebSocket-Protocol header (not URL query params)
- Validate Origin header on WebSocket connections to prevent CSWSH attacks

## Development Notes

- Run tests before committing: `cargo test -p <package-name>`
- Apply database migrations from `migrations/` folder
- Use `PasswordService::new_dev()` for faster testing (dev parameters only)

## Gateway Service (WebSocket)

### Architecture

The gateway-service provides real-time bidirectional WebSocket communication:

- **Token Auth**: JWT validated via `Sec-WebSocket-Protocol` header
- **Room-Based**: Clients join rooms based on token claims (user ID or custom room)
- **Pub/Sub**: DashMap + tokio::broadcast for efficient fan-out messaging
- **Cleanup**: Automatic resource cleanup when rooms become empty

### Key Files

| File | Purpose |
|------|---------|
| `gateway-service/src/main.rs` | Server setup, CORS, routing |
| `gateway-service/src/state.rs` | AppState with RoomsMap |
| `gateway-service/src/ws_handler.rs` | WebSocket handler with auth |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GATEWAY_BIND_ADDR` | Server bind address | `0.0.0.0:9000` |
| `JWT_SECRET` | JWT signing secret | `supersecret` |
| `ALLOWED_ORIGINS` | Comma-separated allowed origins | `http://localhost:3000,http://127.0.0.1:3000` |

### WebSocket Client Usage

```javascript
// Browser client
const token = "your-jwt-token";
const ws = new WebSocket("wss://gateway:9000/ws", ["bearer", token]);

ws.onopen = () => console.log("Connected");
ws.onmessage = (e) => console.log("Received:", e.data);
ws.send("Hello room!");
```

### JWT Token Requirements

The gateway expects JWT tokens with these claims:

```json
{
  "sub": "username",
  "exp": 1234567890,
  "room": "optional-room-id"
}
```

- `sub` (required): User identifier, used as room ID if no `room` claim
- `exp` (required): Expiration timestamp
- `room` (optional): Custom room assignment, defaults to `user:{sub}`

## Phase Status

- **Phase 1**: Completed - Argon2id password hashing
- **Phase 2**: In Progress (background) - JWT token generation
- **Phase 3**: Completed - WebSocket fabric hardening

### Phase 2/3 Integration Items (Pending)

The following require Phase 2 completion:

1. **INT-01**: Unify JWT validation logic across services
2. **INT-02**: Align token claims between auth-api and gateway-service
3. **INT-03**: WebSocket reconnection flow with token refresh
