# Enterprise Readiness Roadmap

> **Document Version**: 1.0
> **Last Updated**: 2024-11-23
> **Status**: Active Development

This document outlines the enterprise readiness roadmap for Unhidra, targeting two primary markets:

1. **Primary**: Ultra-secure, self-hosted enterprise chat/collaboration platform (Signal/Element/Mattermost competitor)
2. **Secondary**: Secure command-and-control backbone for home/industrial automation (Home Assistant on steroids, MQTT-grade but E2E-encrypted and policy-driven)

---

## Table of Contents

1. [Current State Assessment](#current-state-assessment)
2. [Enterprise Gap Analysis](#enterprise-gap-analysis)
3. [Phase 7: E2E Encryption Layer](#phase-7-e2e-encryption-layer)
4. [Phase 8: Enterprise Identity & Access](#phase-8-enterprise-identity--access)
5. [Phase 9: Compliance & Audit](#phase-9-compliance--audit)
6. [Phase 10: Scalability & High Availability](#phase-10-scalability--high-availability)
7. [Phase 11: Enterprise Chat Features](#phase-11-enterprise-chat-features)
8. [Phase 12: Secure Automation Mode](#phase-12-secure-automation-mode)
9. [Phase 13: Go-to-Market Packaging](#phase-13-go-to-market-packaging)
10. [Implementation Priority](#implementation-priority)
11. [Crate & Dependency Reference](#crate--dependency-reference)

---

## Current State Assessment

### Completed Phases (v0.2.0)

| Phase | Component | Status | Enterprise Ready |
|-------|-----------|--------|------------------|
| 1 | Argon2id Password Hashing | ✅ Complete | ✅ Yes |
| 2 | ML IPC Sidecar Isolation | ✅ Complete | ✅ Yes |
| 3 | WSS Gateway Security | ✅ Complete | ✅ Yes |
| 4 | ESP32 Firmware & WSS | ✅ Complete | ✅ Yes |
| 5 | Rate Limiting & Device Mgmt | ✅ Complete | ✅ Yes |
| 6 | Docker/Prometheus/Grafana | ✅ Complete | 🟡 Partial |

### Current Security Posture

| Component | Implementation | Enterprise Standard |
|-----------|---------------|---------------------|
| Password Hashing | Argon2id (48MiB, t=3) | ✅ Exceeds OWASP |
| JWT Algorithm | HS256 (symmetric) | 🟡 RS256 recommended |
| Transport Encryption | TLS 1.2/1.3 via WSS | ✅ Meets standard |
| Message Encryption | None (TLS only) | ❌ E2EE required |
| Key Management | Environment variables | ❌ HSM/KMS required |
| SSO/Federation | Not implemented | ❌ OIDC/SAML required |
| Audit Logging | Basic file logs | ❌ Immutable logs required |
| Multi-tenancy | Not implemented | ❌ Required |

### Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Load Balancer (TLS)                      │
└───────────────────────┬─────────────────────────────────────┘
                        │
         ┌──────────────┼──────────────┐
         │              │              │
         ▼              ▼              ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  auth-api   │  │   gateway   │  │  ml-bridge  │
│  (Axum)     │  │   (WSS)     │  │  (IPC)      │
│  Port 9200  │  │  Port 9000  │  │  UDS        │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       │         ┌──────┴──────┐         │
       │         │ DashMap +   │         │
       │         │ broadcast   │         │
       │         └─────────────┘         │
       │                                 │
       ▼                                 ▼
┌─────────────┐                   ┌─────────────┐
│   SQLite    │                   │   Python    │
│  (Auth DB)  │                   │  Worker     │
└─────────────┘                   └─────────────┘
```

---

## Enterprise Gap Analysis

### Immediate Gaps to Close (3-6 Weeks)

| Category | Current State | Enterprise Requirement | Priority |
|----------|---------------|----------------------|----------|
| **Encryption & Crypto** | HS256 symmetric only | E2EE + forward secrecy | 🔴 Critical |
| **Key Management** | None | Centralized/BYOK enterprise KMS | 🔴 Critical |
| **Identity & Access** | Simple username + JWT | SSO (SAML/OIDC), SCIM, RBAC/ABAC | 🔴 Critical |
| **Auditing & Compliance** | Basic file logs | Immutable audit log, GDPR/SOC2/HIPAA | 🟠 High |
| **Data Sovereignty** | Runs anywhere | Air-gapped/on-prem certified, FIPS | 🟠 High |
| **Scalability** | Single node | 10k-100k concurrent users | 🟠 High |
| **High Availability** | None | Zero-downtime blue-green deploys | 🟡 Medium |

### Feature Parity with Enterprise Platforms (6-12 Weeks)

| Feature | Why Enterprises Pay | Gap Level |
|---------|---------------------|-----------|
| Channels/Spaces/Threads | Core collaboration | 🟠 High |
| Message Reactions/Editing | Expected UX | 🟡 Medium |
| Read Receipts/Typing Indicators | User experience | 🟡 Medium |
| E2EE File Sharing | Replaces Slack/Dropbox | 🟠 High |
| Voice/Video (WebRTC) | Unified comms | 🟡 Medium |
| Bots & App Platform | Automation | 🟡 Medium |
| Bridge to Legacy Systems | Migration path | 🟢 Low |
| Mobile/Desktop Clients | User access | 🟠 High |

---

## Phase 7: E2E Encryption Layer

**Timeline**: Immediate priority (3-4 weeks)
**Status**: 📋 Planned

### Overview

Implement end-to-end encryption with forward secrecy using the Double Ratchet algorithm, with a path to MLS (Message Layer Security) for group messaging.

### Technical Approach

```
┌────────────────────────────────────────────────────────────┐
│                    Message Encryption Flow                  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Client A                Gateway                Client B   │
│     │                       │                       │      │
│     │ ─── X3DH Key Exchange ────────────────────►  │      │
│     │                       │                       │      │
│     │ ◄─── PreKey Bundle ──────────────────────── │      │
│     │                       │                       │      │
│     │ ─── Encrypted Msg ───►│─── Encrypted Msg ──► │      │
│     │   (Double Ratchet)    │   (Opaque to server) │      │
│     │                       │                       │      │
└────────────────────────────────────────────────────────────┘
```

### Implementation Tasks

- [ ] **E2EE-01**: X25519 key pair generation for each device
  - Crate: `x25519-dalek` v2.0
  - Generate identity keys on device registration
  - Store private keys in secure enclave (mobile) or OS keychain

- [ ] **E2EE-02**: X3DH (Extended Triple Diffie-Hellman) key exchange
  - Crate: Custom implementation using `x25519-dalek`
  - PreKey bundles stored on server (encrypted)
  - One-time prekeys for forward secrecy

- [ ] **E2EE-03**: Double Ratchet implementation
  - Crate: `double-ratchet` or custom with `chacha20poly1305`
  - Symmetric ratchet with HKDF
  - DH ratchet on each message exchange

- [ ] **E2EE-04**: ChaCha20-Poly1305 message encryption
  - Crate: `chacha20poly1305` v0.10
  - AEAD encryption for each message
  - Associated data: sender_id, timestamp, message_id

- [ ] **E2EE-05**: MLS (Message Layer Security) stub for groups
  - Crate: `openmls` v0.5
  - Group key agreement protocol
  - Efficient for large groups (O(log n) vs O(n))

### Database Schema Extensions

```sql
-- Device cryptographic keys
CREATE TABLE device_keys (
    device_id TEXT PRIMARY KEY,
    identity_public_key BLOB NOT NULL,        -- X25519 public key
    signed_prekey BLOB NOT NULL,              -- Signed prekey bundle
    signed_prekey_signature BLOB NOT NULL,
    prekey_timestamp TIMESTAMP NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);

-- One-time prekeys (consumed on use)
CREATE TABLE one_time_prekeys (
    id INTEGER PRIMARY KEY,
    device_id TEXT NOT NULL,
    prekey_id INTEGER NOT NULL,
    public_key BLOB NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);

-- Encrypted message metadata (content is opaque)
CREATE TABLE messages (
    message_id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_device_id TEXT NOT NULL,
    encrypted_payload BLOB NOT NULL,          -- E2EE encrypted
    timestamp TIMESTAMP NOT NULL,
    INDEX idx_conversation (conversation_id, timestamp)
);
```

### Crate Dependencies

```toml
[dependencies]
x25519-dalek = { version = "2.0", features = ["static_secrets"] }
chacha20poly1305 = "0.10"
hkdf = "0.12"
sha2 = "0.10"
rand = "0.8"
# Optional for MLS
openmls = { version = "0.5", optional = true }
```

---

## Phase 8: Enterprise Identity & Access

**Timeline**: 2-3 weeks
**Status**: 📋 Planned

### Overview

Implement enterprise SSO integration, fine-grained RBAC/ABAC, and modern passwordless authentication.

### Technical Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Identity Architecture                      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│   │   Okta      │    │  Azure AD   │    │  Keycloak   │     │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│          │                  │                  │             │
│          └──────────────────┼──────────────────┘             │
│                             │                                │
│                    ┌────────▼────────┐                       │
│                    │   OIDC/SAML     │                       │
│                    │   Adapter       │                       │
│                    └────────┬────────┘                       │
│                             │                                │
│                    ┌────────▼────────┐                       │
│                    │   Auth Service  │                       │
│                    │   (RS256 JWT)   │                       │
│                    └────────┬────────┘                       │
│                             │                                │
│          ┌──────────────────┼──────────────────┐            │
│          │                  │                  │            │
│   ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐      │
│   │    RBAC     │   │    ABAC     │   │   WebAuthn  │      │
│   │   Engine    │   │   Engine    │   │   Adapter   │      │
│   └─────────────┘   └─────────────┘   └─────────────┘      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Implementation Tasks

- [ ] **IAM-01**: Upgrade JWT to RS256 asymmetric signing
  - Crate: `jsonwebtoken` v9 (already in use)
  - RSA-2048 or Ed25519 signing keys
  - Key rotation support (jwks endpoint)

- [ ] **IAM-02**: OpenID Connect provider integration
  - Crate: `openidconnect` v3.4
  - Authorization code flow with PKCE
  - ID token validation and user provisioning

- [ ] **IAM-03**: SAML 2.0 SP implementation
  - Crate: `samael` v0.0.14
  - SP metadata generation
  - Assertion parsing and validation

- [ ] **IAM-04**: SCIM 2.0 user provisioning
  - REST API for user/group sync
  - Just-in-time (JIT) provisioning
  - Automatic deprovisioning on IdP removal

- [ ] **IAM-05**: Role-Based Access Control (RBAC)
  - Roles: Super Admin → Org Admin → Space Admin → Member → Guest
  - Permission matrix for all operations
  - Role inheritance and delegation

- [ ] **IAM-06**: Attribute-Based Access Control (ABAC)
  - Policy engine (OPA/Rego compatible)
  - Context-aware permissions (time, location, device)
  - Dynamic policy evaluation

- [ ] **IAM-07**: WebAuthn/Passkey authentication
  - Crate: `webauthn-rs` v0.4
  - Device-bound credentials (no passwords)
  - Platform authenticator support (TouchID, FaceID, Windows Hello)

### Database Schema Extensions

```sql
-- Organizations
CREATE TABLE organizations (
    org_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sso_provider TEXT,                -- 'oidc', 'saml', 'local'
    sso_config JSONB,                 -- IdP-specific configuration
    created_at TIMESTAMP NOT NULL
);

-- Roles
CREATE TABLE roles (
    role_id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    permissions JSONB NOT NULL,        -- {"channels.create": true, ...}
    parent_role_id TEXT,               -- For inheritance
    FOREIGN KEY (org_id) REFERENCES organizations(org_id)
);

-- User-Role assignments
CREATE TABLE user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    scope TEXT NOT NULL,               -- 'org', 'space', 'channel'
    scope_id TEXT NOT NULL,
    granted_by TEXT NOT NULL,
    granted_at TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id, role_id, scope_id)
);

-- WebAuthn credentials
CREATE TABLE webauthn_credentials (
    credential_id BLOB PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key BLOB NOT NULL,
    counter INTEGER NOT NULL,
    transports JSONB,
    created_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);
```

### Crate Dependencies

```toml
[dependencies]
openidconnect = "3.4"
samael = "0.0.14"
webauthn-rs = { version = "0.4", features = ["danger-allow-state-serialisation"] }
```

---

## Phase 9: Compliance & Audit

**Timeline**: 2-3 weeks
**Status**: 📋 Planned

### Overview

Implement immutable audit logging, data retention policies, and compliance controls for GDPR, SOC2, and HIPAA.

### Implementation Tasks

- [ ] **AUDIT-01**: Immutable audit log table
  - Append-only Postgres table with hash chaining
  - No UPDATE/DELETE permissions
  - Cryptographic integrity verification

- [ ] **AUDIT-02**: Security event logging
  - Authentication attempts (success/failure)
  - Permission changes
  - Device registrations/revocations
  - Configuration changes

- [ ] **AUDIT-03**: OpenSearch/ELK integration
  - Structured JSON log export
  - Real-time log streaming
  - Retention policy enforcement

- [ ] **AUDIT-04**: WORM storage hook
  - AWS S3 Object Lock integration
  - Azure Immutable Blob Storage
  - Compliance hold support

- [ ] **AUDIT-05**: GDPR data subject rights
  - Data export (right to portability)
  - Data deletion (right to erasure)
  - Consent management

- [ ] **AUDIT-06**: Data retention policies
  - Configurable retention periods
  - Automatic data purging
  - Legal hold override

### Audit Log Schema

```sql
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    actor_id TEXT,
    actor_type TEXT,                   -- 'user', 'device', 'system'
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,             -- 'success', 'failure', 'denied'
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    prev_hash BYTEA,                   -- Hash chain for integrity
    event_hash BYTEA NOT NULL          -- SHA-256 of this record
);

-- Prevent modifications
REVOKE UPDATE, DELETE ON audit_log FROM PUBLIC;

-- Append-only trigger
CREATE OR REPLACE FUNCTION audit_log_immutable()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit log is immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_audit_modification
BEFORE UPDATE OR DELETE ON audit_log
FOR EACH ROW EXECUTE FUNCTION audit_log_immutable();
```

---

## Phase 10: Scalability & High Availability

**Timeline**: 3-4 weeks
**Status**: 📋 Planned

### Overview

Scale from single-node to multi-region active-active deployment supporting 10k-100k concurrent users.

### Architecture Evolution

```
┌───────────────────────────────────────────────────────────────┐
│                    Multi-Region Architecture                   │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Region A (US-East)              Region B (EU-West)           │
│  ┌─────────────────┐            ┌─────────────────┐          │
│  │  Load Balancer  │◄──────────►│  Load Balancer  │          │
│  └────────┬────────┘            └────────┬────────┘          │
│           │                              │                    │
│  ┌────────▼────────┐            ┌────────▼────────┐          │
│  │  Gateway Pool   │◄──NATS────►│  Gateway Pool   │          │
│  │  (3 replicas)   │   Cluster  │  (3 replicas)   │          │
│  └────────┬────────┘            └────────┬────────┘          │
│           │                              │                    │
│  ┌────────▼────────┐            ┌────────▼────────┐          │
│  │ Redis Cluster   │◄──────────►│ Redis Cluster   │          │
│  │ (Presence/Cache)│  Replication│ (Presence/Cache)│          │
│  └────────┬────────┘            └────────┬────────┘          │
│           │                              │                    │
│  ┌────────▼────────┐            ┌────────▼────────┐          │
│  │ CockroachDB     │◄──────────►│ CockroachDB     │          │
│  │ (Global Tables) │  Multi-region│ (Global Tables) │          │
│  └─────────────────┘            └─────────────────┘          │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### Implementation Tasks

- [ ] **SCALE-01**: Replace broadcast channels with NATS JetStream
  - Crate: `async-nats` v0.33
  - Cross-service message routing
  - Persistent message streams
  - At-least-once delivery guarantees

- [ ] **SCALE-02**: Redis Cluster for presence and caching
  - Crate: `redis` v0.24 with cluster feature
  - Session storage
  - Rate limit counters (distributed)
  - Pub/sub for presence updates

- [ ] **SCALE-03**: Database migration to CockroachDB/Citus
  - SQLx compatible (Postgres protocol)
  - Horizontal sharding
  - Multi-region replication
  - Automatic failover

- [ ] **SCALE-04**: Kubernetes operator and Helm chart
  - Deployment, Service, ConfigMap, Secret resources
  - Horizontal Pod Autoscaler
  - Pod Disruption Budget
  - Network Policies

- [ ] **SCALE-05**: Health checks and graceful shutdown
  - Liveness and readiness probes
  - Connection draining
  - SIGTERM handling

- [ ] **SCALE-06**: Service mesh integration
  - Linkerd or Istio sidecar support
  - mTLS between services
  - Traffic management (canary, A/B)

### Kubernetes Resources

```yaml
# helm/unhidra/templates/gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Release.Name }}-gateway
spec:
  replicas: {{ .Values.gateway.replicas }}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  template:
    spec:
      containers:
        - name: gateway
          image: {{ .Values.gateway.image }}
          ports:
            - containerPort: 9000
          livenessProbe:
            httpGet:
              path: /health
              port: 9000
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: 9000
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            requests:
              memory: "256Mi"
              cpu: "100m"
            limits:
              memory: "1Gi"
              cpu: "1000m"
```

### Crate Dependencies

```toml
[dependencies]
async-nats = "0.33"
redis = { version = "0.24", features = ["cluster-async", "tokio-comp"] }
```

---

## Phase 11: Enterprise Chat Features

**Timeline**: 4-6 weeks
**Status**: 📋 Planned

### Overview

Implement feature parity with enterprise chat platforms (Slack, Teams, Element).

### Feature Matrix

| Feature | Implementation Path | Crates |
|---------|---------------------|--------|
| Channels/Spaces | `Channel { id, name, org_id, private }` | - |
| Threaded Replies | `parent_message_id` foreign key | - |
| Reactions | `reactions` table with emoji support | - |
| Message Editing | Edit window (5 min), version history | - |
| Read Receipts | Delivery/read events via presence | - |
| Typing Indicators | Presence service extension | - |
| E2EE File Sharing | Client encrypt → object store → share link | `chacha20poly1305` |
| Voice/Video | WebRTC signaling service | `webrtc` |
| Bots Platform | Webhooks + OAuth2 app framework | `oauth2` |

### Implementation Tasks

- [ ] **CHAT-01**: Channel and Space management
  - Create/archive channels
  - Public/private visibility
  - Channel permissions

- [ ] **CHAT-02**: Threaded conversations
  - Reply to message (parent_id)
  - Thread summary in channel view
  - Thread notifications

- [ ] **CHAT-03**: Message reactions and editing
  - Emoji reactions with user list
  - Edit window (configurable)
  - Edit history (compliance)

- [ ] **CHAT-04**: Read receipts and typing indicators
  - Per-message delivery status
  - Read status (privacy-respecting)
  - Typing indicator events

- [ ] **CHAT-05**: E2EE file sharing
  - Client-side encryption before upload
  - MinIO/S3 backend
  - Encrypted thumbnail generation
  - Virus scanning (pre-encryption on client)

- [ ] **CHAT-06**: WebRTC voice/video signaling
  - Separate `webrtc-signaling` microservice
  - STUN/TURN server integration
  - SFU for group calls (LiveKit compatible)

- [ ] **CHAT-07**: Bot and app platform
  - Webhook integrations (incoming/outgoing)
  - OAuth2 app authorization
  - Bot user type with API access

---

## Phase 12: Secure Automation Mode

**Timeline**: 4-6 weeks (parallel track)
**Status**: 📋 Planned

### Overview

Position Unhidra as a secure backbone for home/industrial automation, replacing insecure MQTT with E2E-encrypted policy-driven communication.

### Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                 Secure Automation Architecture                  │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ ESP32 Sensor │  │ Smart Light  │  │ Thermostat   │         │
│  │ (Device Cert)│  │ (Device Cert)│  │ (Device Cert)│         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                  │
│         └─────────────────┼─────────────────┘                  │
│                           │                                    │
│              ┌────────────▼────────────┐                       │
│              │   Gateway (MQTT-WS)     │                       │
│              │   E2EE Message Routing  │                       │
│              └────────────┬────────────┘                       │
│                           │                                    │
│              ┌────────────▼────────────┐                       │
│              │   Policy Engine (OPA)   │                       │
│              │   "thermostat can only  │                       │
│              │    write to hvac-topic" │                       │
│              └────────────┬────────────┘                       │
│                           │                                    │
│              ┌────────────▼────────────┐                       │
│              │   Action Dispatcher     │                       │
│              │   Voice → Intent → Cmd  │                       │
│              └─────────────────────────┘                       │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### Implementation Tasks

- [ ] **AUTO-01**: Device-bound X.509 certificates
  - Auto-enrollment via EST (RFC 7030) or CMPv2
  - Crate: `rcgen`, `x509-parser`
  - No shared secrets on devices

- [ ] **AUTO-02**: MQTT over WebSocket bridge
  - Gateway accepts MQTT-WS protocol
  - Topic-based routing to channels
  - QoS levels (0, 1, 2) support

- [ ] **AUTO-03**: OPA policy engine integration
  - Crate: `opa` (WASM embeddings)
  - Rego policy language
  - Per-topic write permissions

- [ ] **AUTO-04**: Offline message queue
  - Durable message storage
  - Offline device delivery on reconnect
  - Message expiration policies

- [ ] **AUTO-05**: Voice command integration
  - Whisper.cpp Rust bindings for STT
  - Intent classification
  - Action dispatcher to devices

### Policy Example (Rego)

```rego
package unhidra.automation

default allow = false

# Thermostat can only publish to hvac topics
allow {
    input.device.type == "thermostat"
    input.action == "publish"
    startswith(input.topic, "hvac/")
}

# Sensors can publish to their own topics
allow {
    input.device.type == "sensor"
    input.action == "publish"
    input.topic == concat("/", ["sensors", input.device.id, input.reading_type])
}

# Admins can do anything
allow {
    input.user.role == "admin"
}
```

---

## Phase 13: Go-to-Market Packaging

**Timeline**: 2-4 weeks
**Status**: 📋 Planned

### Overview

Package Unhidra for enterprise deployment and commercial support.

### Deliverables

- [ ] **GTM-01**: One-click enterprise installer
  - Terraform modules (AWS/Azure/GCP)
  - Ansible playbooks for on-prem
  - Official Helm chart on ArtifactHub
  - Pre-built OVA appliance (air-gapped)

- [ ] **GTM-02**: FIPS mode profile
  - Crate: `ring` with FIPS feature or `aws-lc-rs`
  - FIPS-validated TLS (Rustls + aws-lc)
  - Zero-telemetry build flag

- [ ] **GTM-03**: Commercial support tier
  - SLA response times
  - Long-term support (LTS) branches
  - Certified FIPS builds

- [ ] **GTM-04**: Documentation portal
  - API reference (OpenAPI)
  - Integration guides
  - Security whitepaper
  - Compliance attestations

---

## Implementation Priority

### Sprint 1 (Weeks 1-3): Foundation

| Priority | Task | Phase | Owner |
|----------|------|-------|-------|
| 🔴 P0 | X25519 key generation | E2EE-01 | - |
| 🔴 P0 | Double Ratchet implementation | E2EE-03 | - |
| 🔴 P0 | RS256 JWT upgrade | IAM-01 | - |
| 🟠 P1 | OIDC integration | IAM-02 | - |

### Sprint 2 (Weeks 4-6): Enterprise Identity

| Priority | Task | Phase | Owner |
|----------|------|-------|-------|
| 🔴 P0 | Complete E2EE message flow | E2EE-04 | - |
| 🔴 P0 | RBAC engine | IAM-05 | - |
| 🟠 P1 | Audit logging | AUDIT-01/02 | - |
| 🟠 P1 | WebAuthn support | IAM-07 | - |

### Sprint 3 (Weeks 7-9): Scalability

| Priority | Task | Phase | Owner |
|----------|------|-------|-------|
| 🟠 P1 | NATS JetStream | SCALE-01 | - |
| 🟠 P1 | Redis Cluster | SCALE-02 | - |
| 🟠 P1 | Kubernetes Helm chart | SCALE-04 | - |
| 🟡 P2 | Health checks | SCALE-05 | - |

### Sprint 4 (Weeks 10-12): Features & Polish

| Priority | Task | Phase | Owner |
|----------|------|-------|-------|
| 🟠 P1 | Channels/Threads | CHAT-01/02 | - |
| 🟡 P2 | Message reactions | CHAT-03 | - |
| 🟡 P2 | File sharing | CHAT-05 | - |
| 🟢 P3 | Voice/Video signaling | CHAT-06 | - |

---

## Crate & Dependency Reference

### Cryptography

| Crate | Version | Purpose |
|-------|---------|---------|
| `x25519-dalek` | 2.0 | Key exchange |
| `chacha20poly1305` | 0.10 | AEAD encryption |
| `hkdf` | 0.12 | Key derivation |
| `sha2` | 0.10 | Hashing |
| `argon2` | 0.5 | Password hashing (existing) |
| `ring` | 0.17 | FIPS crypto (optional) |
| `openmls` | 0.5 | MLS group encryption |

### Identity & Auth

| Crate | Version | Purpose |
|-------|---------|---------|
| `openidconnect` | 3.4 | OIDC client |
| `samael` | 0.0.14 | SAML 2.0 |
| `webauthn-rs` | 0.4 | Passkey/FIDO2 |
| `jsonwebtoken` | 9.3 | JWT (existing) |

### Infrastructure

| Crate | Version | Purpose |
|-------|---------|---------|
| `async-nats` | 0.33 | Message queue |
| `redis` | 0.24 | Caching/presence |
| `sqlx` | 0.7 | Database (Postgres) |

### Automation

| Crate | Version | Purpose |
|-------|---------|---------|
| `rcgen` | 0.12 | X.509 certificates |
| `x509-parser` | 0.15 | Certificate parsing |
| `rumqttc` | 0.23 | MQTT client |

---

## Success Metrics

### Technical KPIs

| Metric | Target | Current |
|--------|--------|---------|
| Concurrent connections | 100,000 | ~1,000 |
| Message latency (p99) | <100ms | ~50ms |
| E2EE coverage | 100% | 0% |
| Uptime SLA | 99.99% | N/A |

### Business KPIs

| Metric | Target |
|--------|--------|
| Enterprise pilot customers | 5 |
| SOC2 Type II certification | Q2 2025 |
| HIPAA BAA availability | Q3 2025 |
| FIPS certification | Q4 2025 |

---

## References

- [Signal Protocol Specifications](https://signal.org/docs/)
- [MLS RFC (RFC 9420)](https://datatracker.ietf.org/doc/html/rfc9420)
- [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/)
- [SOC2 Compliance](https://www.aicpa.org/soc2)
- [NIST FIPS 140-3](https://csrc.nist.gov/publications/detail/fips/140/3/final)
