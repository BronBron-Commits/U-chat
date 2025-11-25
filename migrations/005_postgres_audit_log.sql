-- PostgreSQL Immutable Audit Log
-- Optimized for append-only workloads with fillfactor=100
-- IMPORTANT: Revoke DELETE and UPDATE permissions after creating

CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Actor information
    actor_id UUID NOT NULL,
    actor_type TEXT NOT NULL DEFAULT 'user' CHECK (actor_type IN ('user', 'device', 'system', 'service', 'anonymous')),
    actor_ip INET,

    -- Action details
    action TEXT NOT NULL,
    action_result TEXT NOT NULL DEFAULT 'success' CHECK (action_result IN ('success', 'failure', 'denied', 'error')),

    -- Resource information
    resource_type TEXT,
    resource_id UUID,

    -- Context and metadata
    service_name TEXT,
    request_id UUID,
    session_id UUID,
    metadata JSONB,

    -- Integrity (optional tamper detection)
    signature BYTEA,
    previous_hash BYTEA
) WITH (fillfactor=100);

-- Prevent tampering: Create immutable table policy
-- Note: Run these commands manually after migration with appropriate privileges
-- REVOKE DELETE, UPDATE ON audit_log FROM PUBLIC;
-- REVOKE TRUNCATE ON audit_log FROM PUBLIC;

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_audit_occurred_at ON audit_log(occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_log(actor_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(action, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_log(resource_type, resource_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_result ON audit_log(action_result, occurred_at DESC) WHERE action_result != 'success';
CREATE INDEX IF NOT EXISTS idx_audit_session ON audit_log(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_service ON audit_log(service_name, occurred_at DESC);

-- GIN index for JSON metadata queries
CREATE INDEX IF NOT EXISTS idx_audit_metadata ON audit_log USING GIN (metadata jsonb_path_ops);

-- Materialized view for security events (refresh periodically)
CREATE MATERIALIZED VIEW IF NOT EXISTS security_events AS
SELECT
    id,
    occurred_at,
    actor_id,
    actor_type,
    actor_ip,
    action,
    action_result,
    resource_type,
    resource_id,
    service_name,
    metadata
FROM audit_log
WHERE action IN (
    'login',
    'login_failed',
    'logout',
    'password_change',
    'password_reset',
    'passkey_registered',
    'passkey_authenticated',
    'passkey_revoked',
    'device_registered',
    'device_connected',
    'device_revoked',
    'permission_denied',
    'rate_limit_exceeded',
    'suspicious_activity',
    'tls_handshake_failed',
    'token_refresh',
    'sso_login',
    'sso_login_failed'
)
ORDER BY occurred_at DESC;

CREATE INDEX IF NOT EXISTS idx_security_events_occurred_at ON security_events(occurred_at DESC);

-- View for compliance reporting (aggregated daily)
CREATE OR REPLACE VIEW compliance_summary AS
SELECT
    DATE(occurred_at) as log_date,
    actor_type,
    action,
    action_result,
    COUNT(*) as event_count,
    COUNT(DISTINCT actor_id) as unique_actors,
    COUNT(DISTINCT CASE WHEN action_result != 'success' THEN actor_id END) as failed_actors
FROM audit_log
WHERE occurred_at >= CURRENT_DATE - INTERVAL '90 days'
GROUP BY DATE(occurred_at), actor_type, action, action_result
ORDER BY log_date DESC, event_count DESC;

-- View for failed authentication attempts (security monitoring)
CREATE OR REPLACE VIEW failed_auth_attempts AS
SELECT
    actor_id,
    actor_ip,
    action,
    COUNT(*) as attempt_count,
    MAX(occurred_at) as last_attempt,
    array_agg(DISTINCT service_name) as services,
    jsonb_object_agg(
        DATE(occurred_at),
        COUNT(*)
    ) FILTER (WHERE occurred_at >= CURRENT_DATE - INTERVAL '7 days') as daily_counts
FROM audit_log
WHERE action IN ('login_failed', 'sso_login_failed', 'passkey_authenticated')
  AND action_result IN ('failure', 'denied')
  AND occurred_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY actor_id, actor_ip, action
HAVING COUNT(*) >= 3
ORDER BY attempt_count DESC, last_attempt DESC;

-- Function to refresh security events view
CREATE OR REPLACE FUNCTION refresh_security_events()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY security_events;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE audit_log IS 'Immutable audit log for compliance and security monitoring. Do not DELETE or UPDATE.';
COMMENT ON COLUMN audit_log.occurred_at IS 'Timestamp of the event (server time, UTC)';
COMMENT ON COLUMN audit_log.actor_id IS 'User ID, device ID, or service identifier';
COMMENT ON COLUMN audit_log.metadata IS 'Additional context as JSON (e.g., user agent, error details)';
COMMENT ON COLUMN audit_log.signature IS 'Optional HMAC for tamper detection';
COMMENT ON COLUMN audit_log.previous_hash IS 'Hash of previous entry for chain integrity';
