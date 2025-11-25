# Unhidra - Enterprise Secure Chat Platform

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Lightweight modular Rust chat platform with enterprise-grade security features including End-to-End Encryption (E2EE), SSO, and IoT device support.

## 🎯 Key Features

- **End-to-End Encryption (E2EE)**: Double Ratchet protocol with X25519 and ChaCha20Poly1305
- **Enterprise SSO**: OpenID Connect with support for Okta, Azure AD, Google Workspace, Keycloak
- **WebAuthn/Passkeys**: Modern passwordless authentication
- **IoT Device Support**: MQTT bridge for ESP32 and other embedded devices
- **Multi-Node Scalability**: Redis Streams for horizontal scaling
- **Immutable Audit Log**: PostgreSQL-based compliance logging
- **Modern Architecture**: Async Rust with Tokio, WebSocket gateway, microservices

## 📊 Architecture

```mermaid
graph TB
    Client[Web/Desktop Client] -->|WSS + JWT| Gateway[Gateway Service]
    Device[IoT Devices] -->|MQTTS| MQTT[MQTT Bridge]
    MQTT -->|E2EE| Gateway

    Gateway -->|gRPC| Chat[Chat Service]
    Gateway -->|gRPC| Presence[Presence Service]
    Gateway -->|gRPC| History[History Service]

    User[User] -->|HTTPS| AuthAPI[Auth API]
    AuthAPI -->|OIDC| SSO[Identity Provider]
    AuthAPI -->|WebAuthn| Passkey[Authenticator]

    Chat -->|Redis Streams| Redis[(Redis)]
    History -->|SQL| Postgres[(PostgreSQL)]
    AuthAPI -->|Audit Log| Postgres

    ML[ML Bridge] -.->|IPC| Python[Python ML Models]

    style Gateway fill:#4a90e2
    style AuthAPI fill:#e27d60
    style Chat fill:#85dcb0
    style Postgres fill:#e8a87c
    style Redis fill:#c38d9e
```

## 🏗️ Service Architecture

### Core Services

| Service | Purpose | Port | Protocol |
|---------|---------|------|----------|
| **auth-api** | HTTP authentication API with Argon2id password hashing, OIDC, WebAuthn | 3000 | HTTPS |
| **auth-service** | WebSocket-based authentication service | 3001 | WSS |
| **gateway-service** | WebSocket gateway with JWT validation and rate limiting | 8080 | WSS |
| **chat-service** | Chat messaging with E2EE support | 50051 | gRPC |
| **presence-service** | User presence tracking (online/offline/away) | 50052 | gRPC |
| **history-service** | Chat history with persistent storage | 50053 | gRPC |
| **bot-service** | Automated tasks and system events | 50054 | gRPC |
| **event-hub-service** | Event distribution and pub/sub | 50055 | gRPC |

### Supporting Infrastructure

- **ml-bridge**: ML model inference isolation via IPC sidecar
- **firmware**: ESP32 firmware with secure WSS client
- **mqtt-bridge**: MQTT-over-WebSocket bridge for IoT devices

### Shared Libraries

- **e2ee**: Double Ratchet, X3DH key agreement, ChaCha20Poly1305 encryption
- **jwt-common**: JWT token validation and generation
- **core**: Shared types, audit logging, configuration
- **proto**: gRPC protocol definitions

## 🔒 Security Features

### Phase 1-4 Implementation Status

| Phase | Feature | Status |
|-------|---------|--------|
| 1 | Argon2id Password Hashing | ✅ Complete |
| 2 | ML IPC Sidecar Isolation | ✅ Complete |
| 3 | WSS Gateway Security | ✅ Complete |
| 4 | ESP32 Firmware & WSS Integration | ✅ Complete |

### End-to-End Encryption (E2EE)

- **Double Ratchet**: Signal Protocol-like forward secrecy and break-in recovery
- **X3DH Key Agreement**: Extended Triple Diffie-Hellman for initial session
- **ChaCha20Poly1305**: Authenticated encryption with associated data (AEAD)
- **X25519**: Elliptic curve Diffie-Hellman key exchange

**Recent Security Fix**: Fixed critical Double Ratchet key derivation bug (see [SECURITY_ENHANCEMENTS.md](docs/SECURITY_ENHANCEMENTS.md))

### Enterprise SSO

- **Providers**: Okta, Azure AD, Google Workspace, Keycloak, custom OIDC
- **Security**: CSRF protection via state tokens, nonce validation, PKCE
- **Session Management**: 15-minute timeout for pending authentications

### WebAuthn/Passkeys

- **Standards**: FIDO2, WebAuthn Level 2
- **Authenticators**: Hardware keys (YubiKey, etc.), platform authenticators (TouchID, Windows Hello)
- **Attestation**: Optional attestation for high-security environments

### Audit Logging

- **Immutable Log**: PostgreSQL table with revoked DELETE/UPDATE permissions
- **Coverage**: Authentication, authorization, data access, security events
- **Compliance**: GDPR, SOC 2, HIPAA-ready audit trails
- **Performance**: Optimized with `fillfactor=100` for append-only workload

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ with `cargo`
- PostgreSQL 15+
- Redis 7+
- Docker & Docker Compose (optional, for containers)

### Development Setup

```bash
# Clone repository
git clone https://github.com/Matthewtgordon/Unhidra.git
cd Unhidra

# Install dependencies and build
cargo build --release

# Run database migrations
psql -U unhidra -d unhidra -f migrations/001_argon2id_password_hash.sql
psql -U unhidra -d unhidra -f migrations/002_devices_table.sql
psql -U unhidra -d unhidra -f migrations/003_audit_log.sql
psql -U unhidra -d unhidra -f migrations/004_channels_threads.sql
psql -U unhidra -d unhidra -f migrations/005_postgres_audit_log.sql

# Start services (development)
./run-all.sh

# Or use Docker Compose
docker-compose up -d
```

### Configuration

Create `.env` file:

```env
# Database
DATABASE_URL=postgres://unhidra:password@localhost:5432/unhidra

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-min-32-chars

# OIDC (optional)
OIDC_OKTA_ISSUER_URL=https://your-domain.okta.com
OIDC_OKTA_CLIENT_ID=your-client-id
OIDC_OKTA_CLIENT_SECRET=your-client-secret
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run E2EE tests
cargo test --package e2ee

# Run integration tests
cargo test --test integration
```

## 🎮 Usage Examples

### Client Authentication

```rust
use e2ee::{SessionStore, PrekeyBundle};

// Initialize E2EE session
let mut alice = SessionStore::new();
let mut bob = SessionStore::new();

// Bob shares his prekey bundle
let bob_bundle = bob.get_identity_bundle();

// Alice initiates session
let initial_msg = alice.initiate_session("bob".to_string(), &bob_bundle)?;

// Bob accepts
bob.accept_session("alice".to_string(), &initial_msg, Some(0))?;

// Alice sends encrypted message
let encrypted = alice.encrypt("bob", b"Hello, Bob!")?;

// Bob decrypts
let decrypted = bob.decrypt("alice", &encrypted)?;
```

### Audit Logging

```rust
use core::audit::{PostgresAuditLogger, AuditEvent, AuditAction};

// Initialize logger
let logger = PostgresAuditLogger::from_url(&db_url).await?;

// Log authentication event
let event = AuditEvent::new("user123", AuditAction::Login)
    .with_ip("192.168.1.1")
    .with_service("auth-api")
    .with_result(ActionResult::Success);

logger.log(event).await?;
```

## 📦 Kubernetes Deployment

```bash
# Add Helm dependencies
cd helm/unhidra
helm dependency update

# Install with PostgreSQL and Redis
helm install unhidra ./helm/unhidra \
  --set postgresql.enabled=true \
  --set redis.enabled=true \
  --set replicaCount=3

# Or use separate managed services
helm install unhidra ./helm/unhidra \
  --set postgresql.enabled=false \
  --set redis.enabled=false \
  --set externalDatabase.host=rds.amazonaws.com \
  --set externalRedis.host=redis.cache.amazonaws.com
```

## 🔧 Development

### Project Structure

```
Unhidra/
├── auth-api/           # HTTP authentication API
├── gateway-service/    # WebSocket gateway
├── chat-service/       # Chat messaging
├── presence-service/   # User presence
├── history-service/    # Message history
├── e2ee/              # E2EE library
├── core/              # Shared types
├── firmware/          # ESP32 firmware
├── helm/              # Kubernetes charts
├── migrations/        # Database migrations
└── docs/              # Documentation
```

### Building for Production

```bash
# Build all services
cargo build --release --workspace

# Build specific service
cargo build --release -p auth-api

# Build with PostgreSQL audit logging
cargo build --release --features postgres
```

### Running Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --test '*' --features integration

# E2EE tests with logging
RUST_LOG=debug cargo test --package e2ee -- --nocapture
```

## 📚 Documentation

- [Security Enhancements](docs/SECURITY_ENHANCEMENTS.md) - Recent security fixes and improvements
- [Architecture](docs/architecture/) - Detailed architecture documentation
- [API Documentation](https://docs.rs/unhidra) - Rust API docs
- [CLAUDE.md](CLAUDE.md) - Development guidelines for AI assistants

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Signal Protocol](https://signal.org/docs/) - E2EE inspiration
- [Noise Protocol](https://noiseprotocol.org/) - Cryptographic framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Rust Community](https://www.rust-lang.org/community) - Amazing ecosystem

## 📞 Support

- GitHub Issues: [Report bugs or request features](https://github.com/Matthewtgordon/Unhidra/issues)
- Discussions: [Ask questions and share ideas](https://github.com/Matthewtgordon/Unhidra/discussions)

---

**⚠️ Security Note**: This is production-grade software with enterprise security features. For security disclosures, please email security@unhidra.io or use GitHub Security Advisories.
