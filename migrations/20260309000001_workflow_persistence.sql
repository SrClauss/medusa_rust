-- ─── Workflow/Saga Persistence ───────────────────────────────────────────────
-- Based on MedusaJS TransactionState and DistributedTransactionStorage.
-- All statements are idempotent (safe to re-run).

-- Workflow execution states (mirrors MedusaJS TransactionState)
DO $$ BEGIN
    CREATE TYPE workflow_state AS ENUM (
        'not_started',
        'invoking',
        'waiting_to_compensate',
        'compensating',
        'done',
        'reverted',
        'failed'
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Individual step states (mirrors MedusaJS StepState)
DO $$ BEGIN
    CREATE TYPE step_state AS ENUM (
        'idle',
        'invoking',
        'waiting_to_compensate',
        'compensating',
        'done',
        'reverted',
        'failed',
        'dormant',
        'skipped',
        'timeout'
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Workflow executions (equivalent to DistributedTransaction in MedusaJS)
CREATE TABLE IF NOT EXISTS workflow_executions (
    id               UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id   TEXT          NOT NULL UNIQUE,  -- idempotency key
    workflow_name    TEXT          NOT NULL,
    state            workflow_state NOT NULL DEFAULT 'not_started',
    context          JSONB         NOT NULL DEFAULT '{}',
    current_step     TEXT,
    error            TEXT,
    retry_count      INTEGER       NOT NULL DEFAULT 0,
    max_retries      INTEGER       NOT NULL DEFAULT 3,
    created_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_executions_name           ON workflow_executions(workflow_name);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_state          ON workflow_executions(state);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_transaction_id ON workflow_executions(transaction_id);

-- Individual steps (equivalent to TransactionStep in MedusaJS)
CREATE TABLE IF NOT EXISTS workflow_steps (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id      UUID        NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    step_name         TEXT        NOT NULL,
    step_depth        INTEGER     NOT NULL DEFAULT 0,
    state             step_state  NOT NULL DEFAULT 'idle',
    started_at        TIMESTAMPTZ,
    completed_at      TIMESTAMPTZ,
    invoke_result     JSONB,
    compensate_result JSONB,
    error             TEXT,
    attempts          INTEGER     NOT NULL DEFAULT 0,
    UNIQUE(execution_id, step_name)
);

CREATE INDEX IF NOT EXISTS idx_workflow_steps_execution ON workflow_steps(execution_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_state     ON workflow_steps(state);

-- Event audit table (used by AuditInterceptor in src/events.rs)
CREATE TABLE IF NOT EXISTS event_audit (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type   TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_audit_type    ON event_audit(event_type);
CREATE INDEX IF NOT EXISTS idx_event_audit_created ON event_audit(created_at);
