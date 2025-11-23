# Security Architecture

> **Document Version**: 1.0
> **Last Updated**: 2024-11-23
> **Classification**: Internal

This document describes the security architecture of Unhidra, covering both the current implementation and planned enterprise enhancements.

---

## Table of Contents

1. [Security Principles](#security-principles)
2. [Current Security Implementation](#current-security-implementation)
3. [Threat Model](#threat-model)
4. [Authentication Architecture](#authentication-architecture)
5. [Authorization Architecture](#authorization-architecture)
6. [Encryption Architecture](#encryption-architecture)
7. [IoT Device Security](#iot-device-security)
8. [Infrastructure Security](#infrastructure-security)
9. [Security Controls Matrix](#security-controls-matrix)
10. [Future: E2E Encryption Design](#future-e2e-encryption-design)

---

## Security Principles

Unhidra follows these core security principles:

1. **Defense in Depth**: Multiple layers of security controls
2. **Least Privilege**: Minimal permissions for all components
3. **Zero Trust**: Verify explicitly, never trust implicitly
4. **Secure by Default**: Security enabled without configuration
5. **Fail Secure**: Errors default to deny access
6. **Memory Safety**: Rust ownership model prevents memory vulnerabilities

---

## Current Security Implementation

### Security Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                        Security Layers                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Layer 1: Transport Security                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  TLS 1.2/1.3 (WSS) • Certificate Verification • HSTS     │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Layer 2: Authentication                                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  JWT (HS256) • Argon2id Passwords • Device API Keys      │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Layer 3: Rate Limiting & Anti-Abuse                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Per-IP • Per-User • Per-Connection • Account Lockout    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Layer 4: Process Isolation                                     │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  ML Sidecar (UDS) • Container Boundaries • Namespaces    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  Layer 5: Data Protection                                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Encryption at Rest • Secure Key Storage • Audit Logs    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Implemented Controls

| Control | Implementation | Status |
|---------|---------------|--------|
| Transport Encryption | TLS 1.2/1.3 via WSS | ✅ Implemented |
| Password Hashing | Argon2id (48MiB, t=3, p=1) | ✅ Implemented |
| Token Authentication | JWT with HS256, Sec-WebSocket-Protocol | ✅ Implemented |
| Device Authentication | API keys (Argon2id hashed) | ✅ Implemented |
| Rate Limiting | Governor crate, token bucket | ✅ Implemented |
| CSRF Protection | Origin header validation | ✅ Implemented |
| Process Isolation | UDS IPC for ML sidecar | ✅ Implemented |
| Input Validation | Parameterized SQL queries | ✅ Implemented |
| Certificate Verification | ESP32 CA bundle | ✅ Implemented |

---

## Threat Model

### Assets

1. **User Credentials**: Passwords, API keys, session tokens
2. **Messages**: Chat content (currently plaintext over TLS)
3. **Device Keys**: IoT device authentication secrets
4. **System Configuration**: JWT secrets, database credentials

### Threat Actors

| Actor | Capability | Motivation |
|-------|-----------|------------|
| External Attacker | Network access | Data theft, disruption |
| Malicious User | Valid account | Privilege escalation, abuse |
| Compromised Device | Network + device secrets | Lateral movement |
| Insider Threat | System access | Data exfiltration |
| State Actor | Advanced persistent | Surveillance, sabotage |

### Attack Vectors

| Vector | Threat | Current Mitigation |
|--------|--------|-------------------|
| Credential Stuffing | Account takeover | Rate limiting, Argon2id |
| Brute Force | Password cracking | Account lockout (planned), rate limiting |
| MITM | Traffic interception | TLS, certificate verification |
| Session Hijacking | Token theft | Short expiry, Sec-WebSocket-Protocol |
| SQL Injection | Database compromise | Parameterized queries |
| XSS | Client compromise | Not applicable (no web UI) |
| DoS/DDoS | Service disruption | Rate limiting, connection limits |
| Replay Attack | Message replay | Timestamps in JWT, message IDs |

### STRIDE Analysis

| Threat | Description | Mitigation |
|--------|-------------|------------|
| **S**poofing | Impersonation | JWT authentication, device certificates |
| **T**ampering | Message modification | TLS integrity, E2EE (planned) |
| **R**epudiation | Deny actions | Audit logging (planned) |
| **I**nformation Disclosure | Data leakage | TLS, E2EE (planned) |
| **D**enial of Service | Availability attack | Rate limiting, connection limits |
| **E**levation of Privilege | Unauthorized access | RBAC (planned), input validation |

---

## Authentication Architecture

### Current: JWT + Password Authentication

```
┌─────────────────────────────────────────────────────────────────┐
│                  Authentication Flow (Current)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Client                  Auth API                 Database     │
│     │                        │                        │         │
│     │ ─── POST /login ─────► │                        │         │
│     │     {user, password}   │                        │         │
│     │                        │ ─── SELECT hash ─────► │         │
│     │                        │ ◄─── Argon2id hash ─── │         │
│     │                        │                        │         │
│     │                        │ ◄── argon2.verify() ─► │         │
│     │                        │                        │         │
│     │ ◄─── JWT Token ─────── │                        │         │
│     │     (HS256 signed)     │                        │         │
│     │                        │                        │         │
│     │ ─── WSS Upgrade ─────► Gateway                  │         │
│     │  Sec-WebSocket-Protocol: <JWT>                  │         │
│     │                        │                        │         │
│     │ ◄─── 101 Switching ─── │                        │         │
│     │                        │                        │         │
└─────────────────────────────────────────────────────────────────┘
```

### Password Storage

```rust
// Argon2id parameters (exceeds OWASP recommendations)
Params {
    m_cost: 49152,     // 48 MiB memory
    t_cost: 3,         // 3 iterations
    p_cost: 1,         // 1 thread (async-safe)
    output_len: 32,    // 256-bit hash
}

// PHC string format (self-documenting)
$argon2id$v=19$m=49152,t=3,p=1$<base64-salt>$<base64-hash>
```

### JWT Token Structure

```json
{
  "sub": "username",           // Subject (user identifier)
  "exp": 1700000000,           // Expiration (Unix timestamp)
  "iat": 1699900000,           // Issued at
  "room": "default",           // Optional room claim
  "display_name": "User Name"  // Optional display name
}
```

**Security Considerations**:
- HS256 is symmetric (shared secret) - suitable for single-service
- RS256 (asymmetric) planned for multi-service deployment
- 60-second clock skew tolerance for distributed systems

### Device Authentication

```
┌─────────────────────────────────────────────────────────────────┐
│                  Device Authentication Flow                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ESP32 Device          Gateway                   Database      │
│       │                    │                          │         │
│       │ ─── WSS Upgrade ──►│                          │         │
│       │  Sec-WebSocket-Protocol: device:<device_id>:<api_key>   │
│       │                    │                          │         │
│       │                    │ ── Lookup device ──────► │         │
│       │                    │ ◄── Device record ────── │         │
│       │                    │                          │         │
│       │                    │ ◄── argon2.verify(api_key, hash)   │
│       │                    │                          │         │
│       │ ◄── 101 Switching ─│                          │         │
│       │    (Authenticated) │                          │         │
│       │                    │                          │         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Authorization Architecture

### Current: User/Device Differentiation

Currently, authorization is basic:
- **Users**: Can send/receive messages in rooms they connect to
- **Devices**: Can send sensor data, receive commands

### Planned: RBAC/ABAC (Phase 8)

```
┌─────────────────────────────────────────────────────────────────┐
│                  Authorization Architecture (Planned)            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │   Request   │───►│    RBAC     │───►│    ABAC     │        │
│   │   Context   │    │   Engine    │    │   Engine    │        │
│   └─────────────┘    └──────┬──────┘    └──────┬──────┘        │
│                             │                  │                │
│                      ┌──────▼──────────────────▼──────┐        │
│                      │        Policy Decision         │        │
│                      │         ALLOW / DENY           │        │
│                      └────────────────────────────────┘        │
│                                                                 │
│   RBAC Roles:                                                   │
│   ┌────────────────────────────────────────────────────────┐   │
│   │ Super Admin > Org Admin > Space Admin > Member > Guest │   │
│   └────────────────────────────────────────────────────────┘   │
│                                                                 │
│   ABAC Attributes:                                              │
│   ┌────────────────────────────────────────────────────────┐   │
│   │ Time of day, IP location, Device type, Resource owner  │   │
│   └────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Encryption Architecture

### Current: Transport Encryption Only

```
┌─────────────────────────────────────────────────────────────────┐
│                  Current Encryption (TLS Only)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Client A              Gateway               Client B          │
│      │                     │                     │              │
│      │ ══TLS══► Plaintext ══TLS══►               │              │
│      │          (on server)                      │              │
│                                                                 │
│   Pros: Simple, no key management                               │
│   Cons: Server can read all messages                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Planned: End-to-End Encryption (Phase 7)

```
┌─────────────────────────────────────────────────────────────────┐
│              End-to-End Encryption Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Client A              Gateway               Client B          │
│      │                     │                     │              │
│      │ ═══════════════════════════════════════► │              │
│      │     E2EE (Double Ratchet + X3DH)         │              │
│      │                     │                     │              │
│      │ ─TLS─► Encrypted ─TLS─►                   │              │
│      │        Blob (opaque)                      │              │
│                                                                 │
│   Key Hierarchy:                                                │
│   ┌────────────────────────────────────────────────────────┐   │
│   │  Identity Key (long-term, per device)                  │   │
│   │       │                                                │   │
│   │       ▼                                                │   │
│   │  Signed PreKey (medium-term, rotates monthly)          │   │
│   │       │                                                │   │
│   │       ▼                                                │   │
│   │  One-Time PreKey (single use, consumed on contact)     │   │
│   │       │                                                │   │
│   │       ▼                                                │   │
│   │  Root Key (per conversation, from X3DH)                │   │
│   │       │                                                │   │
│   │       ▼                                                │   │
│   │  Chain Key (per message direction, ratchets)           │   │
│   │       │                                                │   │
│   │       ▼                                                │   │
│   │  Message Key (per message, derived + discarded)        │   │
│   └────────────────────────────────────────────────────────┘   │
│                                                                 │
│   Algorithms:                                                   │
│   - Key Exchange: X25519 (Curve25519 ECDH)                     │
│   - Encryption: ChaCha20-Poly1305 (AEAD)                       │
│   - KDF: HKDF-SHA256                                           │
│   - Signing: Ed25519 (for identity verification)               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## IoT Device Security

### ESP32 Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   ESP32 Security Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    ESP32 Device                         │   │
│   │  ┌───────────────────────────────────────────────────┐  │   │
│   │  │  Secure Boot (planned)                            │  │   │
│   │  │  - Signed firmware verification                   │  │   │
│   │  │  - Anti-rollback protection                       │  │   │
│   │  └───────────────────────────────────────────────────┘  │   │
│   │  ┌───────────────────────────────────────────────────┐  │   │
│   │  │  Flash Encryption (planned)                       │  │   │
│   │  │  - AES-256-XTS encryption                         │  │   │
│   │  │  - Encrypted firmware storage                     │  │   │
│   │  └───────────────────────────────────────────────────┘  │   │
│   │  ┌───────────────────────────────────────────────────┐  │   │
│   │  │  Credential Storage (current)                     │  │   │
│   │  │  - API keys in .env (compile-time)                │  │   │
│   │  │  - NVS storage (encrypted, planned)               │  │   │
│   │  └───────────────────────────────────────────────────┘  │   │
│   │  ┌───────────────────────────────────────────────────┐  │   │
│   │  │  TLS Client (current)                             │  │   │
│   │  │  - Mozilla CA bundle                              │  │   │
│   │  │  - Server certificate verification                │  │   │
│   │  │  - Optional certificate pinning                   │  │   │
│   │  └───────────────────────────────────────────────────┘  │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│   Communication Security:                                       │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  Device ──WSS+TLS──► Gateway                            │   │
│   │         API key in Sec-WebSocket-Protocol               │   │
│   │         (not in URL query params)                       │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│   Resilience:                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  - Exponential backoff (5s → 60s max)                   │   │
│   │  - Jitter (±30%) to prevent thundering herd             │   │
│   │  - Keep-alive pings (30s) + heartbeat (60s)             │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Device Lifecycle Security

| Phase | Security Control |
|-------|-----------------|
| Manufacturing | Unique device ID, initial API key |
| Provisioning | Secure credential delivery |
| Operation | TLS, API key auth, rate limiting |
| Update | OTA with signed firmware (planned) |
| Decommission | Key revocation, audit trail |

---

## Infrastructure Security

### Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Network Security Zones                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Internet                                                      │
│       │                                                         │
│       │ ┌─────────────────────────────────────┐                │
│       │ │         DMZ (TLS Termination)       │                │
│       │ │  ┌────────────┐  ┌────────────┐    │                │
│       └─┼─►│   nginx    │  │  traefik   │    │                │
│         │  │  (reverse) │  │  (optional)│    │                │
│         │  └─────┬──────┘  └─────┬──────┘    │                │
│         └────────┼───────────────┼───────────┘                │
│                  │               │                              │
│   ┌──────────────┼───────────────┼──────────────────┐          │
│   │              │  Application Zone                │          │
│   │   ┌──────────▼──────────┐  ┌─────────────┐     │          │
│   │   │    Gateway (9000)   │  │  Auth API   │     │          │
│   │   │    (internal only)  │  │   (9200)    │     │          │
│   │   └──────────┬──────────┘  └──────┬──────┘     │          │
│   │              │                    │            │          │
│   │   ┌──────────┴────────────────────┴──────┐     │          │
│   │   │           Internal Network           │     │          │
│   │   └──────────────────────────────────────┘     │          │
│   └────────────────────────────────────────────────┘          │
│                                                                 │
│   ┌─────────────────────────────────────────────────┐          │
│   │              Data Zone (isolated)               │          │
│   │   ┌────────────┐  ┌────────────┐               │          │
│   │   │   SQLite   │  │   Redis    │               │          │
│   │   │  (local)   │  │  (future)  │               │          │
│   │   └────────────┘  └────────────┘               │          │
│   └─────────────────────────────────────────────────┘          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Container Security

| Control | Implementation |
|---------|---------------|
| Base Image | `rust:slim-bookworm` (minimal) |
| Non-root User | Application runs as non-root |
| Read-only FS | Where possible |
| Resource Limits | CPU/memory limits in compose |
| Health Checks | Configured per container |
| Secrets | Environment variables (Vault planned) |

---

## Security Controls Matrix

### OWASP ASVS Mapping

| ASVS Control | Category | Implementation | Status |
|--------------|----------|---------------|--------|
| V2.1.1 | Authentication | Argon2id with OWASP params | ✅ |
| V2.1.2 | Authentication | 128-bit random salt | ✅ |
| V2.1.4 | Authentication | Constant-time comparison | ✅ |
| V2.2.1 | Authentication | Rate limiting on login | ✅ |
| V2.2.3 | Authentication | Account lockout | 📋 Planned |
| V2.8.1 | Authentication | JWT with expiration | ✅ |
| V3.2.1 | Session | Session binding | 📋 Planned |
| V3.5.1 | Session | Secure cookie flags | N/A |
| V4.1.1 | Access Control | RBAC | 📋 Planned |
| V5.1.3 | Validation | Input validation | ✅ |
| V5.3.4 | Validation | Parameterized queries | ✅ |
| V9.1.1 | Communications | TLS 1.2+ | ✅ |
| V9.2.1 | Communications | Certificate validation | ✅ |
| V13.1.1 | API | Rate limiting | ✅ |
| V14.4.1 | Configuration | Security headers | 📋 Planned |

### CIS Controls Mapping

| CIS Control | Description | Implementation | Status |
|-------------|-------------|---------------|--------|
| 3.4 | Encrypt data at rest | SQLite + volume encryption | 📋 Planned |
| 3.10 | Encrypt sensitive data in transit | TLS 1.2/1.3 | ✅ |
| 5.2 | Use unique passwords | Argon2id with salt | ✅ |
| 5.3 | Disable default accounts | No default accounts | ✅ |
| 6.1 | Establish audit log management | Basic logging | 🟡 Partial |
| 6.2 | Activate audit logging | File-based logs | 🟡 Partial |
| 7.1 | Establish malware defense | N/A (no file uploads) | ✅ |
| 12.6 | Use encrypted communication | TLS everywhere | ✅ |

---

## Future: E2E Encryption Design

### Signal Protocol Implementation

The planned E2E encryption follows the Signal Protocol with these components:

1. **X3DH (Extended Triple Diffie-Hellman)**
   - Initial key agreement between two parties
   - Uses identity keys, signed prekeys, and one-time prekeys

2. **Double Ratchet**
   - Continuous key rotation for forward secrecy
   - Each message uses a unique key

3. **Sesame (Multi-Device)**
   - Distribute messages to multiple user devices
   - Device-specific encryption

### Cryptographic Choices

| Component | Algorithm | Crate | Rationale |
|-----------|----------|-------|-----------|
| Key Exchange | X25519 | `x25519-dalek` | Fast, constant-time, widely audited |
| Signing | Ed25519 | `ed25519-dalek` | Fast signature verification |
| Encryption | ChaCha20-Poly1305 | `chacha20poly1305` | AEAD, no AES-NI dependency |
| KDF | HKDF-SHA256 | `hkdf` | Standard key derivation |
| Random | OS CSPRNG | `rand` | Cryptographically secure |

### Key Storage

| Platform | Storage | Protection |
|----------|---------|------------|
| iOS | Keychain | Secure Enclave |
| Android | Keystore | TEE/StrongBox |
| Desktop | OS keychain | User password |
| ESP32 | NVS | Flash encryption (planned) |
| Browser | IndexedDB | In-memory (session only) |

---

## References

- [OWASP ASVS v4.0](https://owasp.org/www-project-application-security-verification-standard/)
- [CIS Controls v8](https://www.cisecurity.org/controls)
- [Signal Protocol Specifications](https://signal.org/docs/)
- [NIST SP 800-63B](https://pages.nist.gov/800-63-3/sp800-63b.html)
- [RFC 9106 - Argon2](https://datatracker.ietf.org/doc/html/rfc9106)
