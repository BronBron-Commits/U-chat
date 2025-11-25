# Security Enhancements & Bug Fixes

## Overview
This document details the security fixes and enhancements implemented in this update.

## Phase A: E2EE Double-Ratchet Fix (CRITICAL)

### Issue
The Double Ratchet implementation had two critical bugs in key derivation:

1. **Asymmetric Key Derivation**: The `DerivedKeys::derive()` function incorrectly derived separate `sending_key` and `receiving_key` from the same DH output. In the Double Ratchet protocol, each DH ratchet step should produce a single chain key.

2. **Message Counter Reset Bug**: The `skip_message_keys()` function incorrectly reset `recv_count` to 0 after skipping keys, breaking sequential message decryption.

### Fix
- Modified `DerivedKeys` to return only `chain_key` and `next_root_key`
- Alice uses the chain key for sending, Bob uses the same chain key for receiving
- Removed the incorrect `recv_count = 0` line in `skip_message_keys()`

### Impact
- ✅ All E2EE tests now pass
- ✅ Multiple sequential messages work correctly
- ✅ Proper forward secrecy and break-in recovery

### Files Modified
- `e2ee/src/cipher.rs` - Fixed DerivedKeys structure
- `e2ee/src/ratchet.rs` - Fixed init_alice, dh_ratchet, and skip_message_keys

## Phase B: OIDC Security (Already Secure)

### Verification
The existing OIDC implementation already includes proper security:

- ✅ CSRF protection via state token (line 244-256 in auth-api/src/oidc.rs)
- ✅ State validation with 15-minute expiry (line 266-281)
- ✅ Nonce validation in ID token (line 299-301)
- ✅ PKCE for authorization code flow

### No Changes Required
The implementation follows OIDC security best practices.

## Phase C: Redis Streams (Already Implemented)

### Verification
The Redis Streams backend is fully implemented:

- ✅ Consumer groups for multi-instance deployment
- ✅ Message persistence with XREADGROUP
- ✅ Automatic reconnection with ConnectionManager
- ✅ Room-based pub/sub

### Files
- `chat-service/src/redis_streams.rs` - Complete implementation

## Phase D: PostgreSQL Audit Log (NEW)

### Implementation
Added production-ready PostgreSQL audit logging:

**Features:**
- Immutable audit table with `fillfactor=100`
- Comprehensive indexing for query performance
- Materialized view for security events
- Compliance reporting views
- Failed authentication monitoring

**Security:**
- Table designed to prevent tampering (revoke DELETE/UPDATE)
- Optional HMAC signatures for tamper detection
- Hash chain for integrity verification
- GIN index for JSON metadata queries

### Files Added
- `migrations/005_postgres_audit_log.sql` - PostgreSQL schema
- `core/src/audit.rs` - PostgresAuditLogger implementation

### Usage
```rust
use core::audit::{PostgresAuditLogger, AuditEvent, AuditAction};

// Initialize logger
let logger = PostgresAuditLogger::from_url("postgres://...").await?;

// Log event
let event = AuditEvent::new("user123", AuditAction::Login)
    .with_ip("192.168.1.1")
    .with_service("auth-api");
logger.log(event).await?;
```

## Testing
All changes have been tested:

```bash
# E2EE tests
cargo test --package e2ee --lib

# Full workspace tests
cargo test --workspace

# Integration tests
docker-compose up -d
cargo test --test integration
```

## Migration Guide

### Enabling PostgreSQL Audit Logging

1. Run the migration:
```bash
psql -U unhidra -d unhidra -f migrations/005_postgres_audit_log.sql
```

2. Revoke tampering permissions:
```sql
REVOKE DELETE, UPDATE, TRUNCATE ON audit_log FROM PUBLIC;
```

3. Enable in your service:
```rust
// In your service main.rs
use core::audit::{PostgresAuditLogger, init_audit_logger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let audit_logger = PostgresAuditLogger::from_url(&env::var("DATABASE_URL")?).await?;
    init_audit_logger(Arc::new(audit_logger));

    // ... rest of service initialization
}
```

### E2EE Migration

No migration required. The fix is backward compatible with existing sessions. New sessions will automatically use the corrected implementation.

## Security Recommendations

1. **Audit Log Retention**: Set up automated archival after 90 days
2. **Security Event Monitoring**: Refresh materialized view hourly
   ```sql
   SELECT refresh_security_events();
   ```
3. **Failed Auth Alerts**: Monitor `failed_auth_attempts` view for brute force attacks
4. **E2EE Key Rotation**: Implement periodic session refresh (every 30 days)
5. **Redis Security**: Enable TLS and authentication in production
6. **OIDC Providers**: Use only trusted identity providers with MFA

## Future Enhancements

- [ ] Noise Protocol Framework integration (Phase A alternative)
- [ ] Hardware Security Module (HSM) support for key storage
- [ ] Audit log signing with external timestamp authority
- [ ] Real-time anomaly detection for security events
- [ ] WebAuthn/Passkey support (already implemented, needs testing)

## References

- Signal Protocol: https://signal.org/docs/
- OIDC Security Best Practices: https://openid.net/specs/
- PostgreSQL Security: https://www.postgresql.org/docs/current/security.html
- Noise Protocol: https://noiseprotocol.org/
